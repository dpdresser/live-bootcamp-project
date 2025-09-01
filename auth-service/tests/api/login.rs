use crate::helpers::{get_random_email, TestApp};
use auth_service::{
    domain::Email, routes::TwoFactorAuthResponse, utils::JWT_COOKIE_NAME, ErrorResponse,
};
use secrecy::ExposeSecret;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password123",
        }),
        serde_json::json!({
            "email": random_email,
        }),
    ];

    for case in test_cases {
        let response = app.login(&case).await;
        assert_eq!(response.status().as_u16(), 422);
    }

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "email": "invalid-email",
            "password": "password123",
        }),
        serde_json::json!({
            "email": random_email,
            "password": "",
        }),
    ];

    for case in test_cases {
        let response = app.login(&case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            case
        );
    }

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;

    let email = get_random_email();

    let response = app
        .signup(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
            "requires2FA": false,
        }))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let response = app
        .login(&serde_json::json!({
            "email": email,
            "password": "wrongPassword123!",
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Incorrect credentials".to_string(),
    );

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let mut app = TestApp::new().await;

    let email = get_random_email();

    let response = app
        .signup(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
            "requires2FA": false,
        }))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let response = app
        .login(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|c| c.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(
        !auth_cookie.value().is_empty(),
        "Auth cookie should not be empty"
    );

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let mut app = TestApp::new().await;

    let email = get_random_email();

    let response = app
        .signup(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
            "requires2FA": true,
        }))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let response = app
        .login(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
        }))
        .await;

    assert_eq!(response.status().as_u16(), 206);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Failed to deserialize response body");

    assert_eq!(
        json_body.message,
        "2FA required".to_string(),
        "Unexpected message in 2FA response"
    );

    assert!(app
        .app_state
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&email).unwrap())
        .await
        .is_ok());

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials_and_2fa() {
    let mut app = TestApp::new().await;
    let email = get_random_email();

    // Sign up a user with 2FA enabled
    let signup_body = serde_json::json!({
        "email": email,
        "password": "correctPassword123!",
        "requires2FA": true,
    });

    let response = app.signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    // Try to login with incorrect password - no email should be sent
    let login_body = serde_json::json!({
        "email": email,
        "password": "wrongPassword123!",
    });

    let response = app.login(&login_body).await;
    assert_eq!(response.status().as_u16(), 401);

    let json_body = response
        .json::<ErrorResponse>()
        .await
        .expect("Failed to deserialize response body");

    assert_eq!(json_body.error, "Incorrect credentials");

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    use wiremock::{
        matchers::{method, path},
        Mock, ResponseTemplate,
    };

    let mut app = TestApp::new().await;
    let email = get_random_email();

    // Sign up a user with 2FA enabled
    let signup_body = serde_json::json!({
        "email": email,
        "password": "validPassword123!",
        "requires2FA": true,
    });

    let response = app.signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    // Set up email mock expectation for first login
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // First login to generate a 2FA code
    let login_body = serde_json::json!({
        "email": email,
        "password": "validPassword123!",
    });

    let response = app.login(&login_body).await;
    assert_eq!(response.status().as_u16(), 206);

    // Get the first 2FA code
    let email_obj = Email::parse(&email).unwrap();
    let first_code = app
        .app_state
        .two_fa_code_store
        .read()
        .await
        .get_code(&email_obj)
        .await
        .expect("Failed to get 2FA code");

    // Wait a moment to ensure time difference
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Set up email mock expectation for second login
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Login again to generate a new 2FA code
    let response = app.login(&login_body).await;
    assert_eq!(response.status().as_u16(), 206);

    // Try to verify with the old code
    let verify_body = serde_json::json!({
        "email": email,
        "loginAttemptId": first_code.0.as_ref().expose_secret(),
        "2FACode": first_code.1.as_ref().expose_secret(),
    });

    let response = app.verify_2fa(&verify_body).await;
    assert_eq!(response.status().as_u16(), 401);

    let json_body = response
        .json::<ErrorResponse>()
        .await
        .expect("Failed to deserialize response body");

    assert_eq!(json_body.error, "Incorrect credentials");

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    use wiremock::{
        matchers::{method, path},
        Mock, ResponseTemplate,
    };

    let mut app = TestApp::new().await;
    let email = get_random_email();

    // Sign up a user with 2FA enabled
    let signup_body = serde_json::json!({
        "email": email,
        "password": "validPassword123!",
        "requires2FA": true,
    });

    let response = app.signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    // Set up email mock expectation
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Login to generate a 2FA code
    let login_body = serde_json::json!({
        "email": email,
        "password": "validPassword123!",
    });

    let response = app.login(&login_body).await;
    assert_eq!(response.status().as_u16(), 206);

    // Get the 2FA code
    let email_obj = Email::parse(&email).unwrap();
    let code = app
        .app_state
        .two_fa_code_store
        .read()
        .await
        .get_code(&email_obj)
        .await
        .expect("Failed to get 2FA code");

    // First verification (should succeed)
    let verify_body = serde_json::json!({
        "email": email,
        "loginAttemptId": code.0.as_ref().expose_secret(),
        "2FACode": code.1.as_ref().expose_secret(),
    });

    let response = app.verify_2fa(&verify_body).await;
    assert_eq!(response.status().as_u16(), 200);

    // Try to use the same code again (should fail)
    let response = app.verify_2fa(&verify_body).await;
    assert_eq!(response.status().as_u16(), 401);

    let json_body = response
        .json::<ErrorResponse>()
        .await
        .expect("Failed to deserialize response body");

    assert_eq!(json_body.error, "Incorrect credentials");

    app.cleanup().await;
}
