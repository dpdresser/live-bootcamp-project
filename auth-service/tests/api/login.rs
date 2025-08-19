use auth_service::{
    domain::Email, routes::TwoFactorAuthResponse, utils::JWT_COOKIE_NAME, ErrorResponse,
};

use crate::helpers::{get_random_email, TestApp};

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
