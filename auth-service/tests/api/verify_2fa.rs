use crate::helpers::{get_random_email, TestApp};
use auth_service::{
    domain::{Email, LoginAttemptId, TwoFACode},
    routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME,
    ErrorResponse,
};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let email = get_random_email();
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let test_cases = [
        serde_json::json!({
            "email": &email,
            "loginAttemptId": login_attempt_id.as_ref(),
        }),
        serde_json::json!({
            "email": &email,
            "2FACode": two_fa_code.as_ref(),
        }),
        serde_json::json!({
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": two_fa_code.as_ref(),
        }),
        serde_json::json!({}),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case,
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let email = get_random_email();
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let test_cases = [
        serde_json::json!({
            "email": "not valid email",
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": two_fa_code.as_ref(),
        }),
        serde_json::json!({
            "email": &email,
            "loginAttemptId": "not valid uuid",
            "2FACode": two_fa_code.as_ref(),
        }),
        serde_json::json!({
            "email": &email,
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": "123",
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case,
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body into ErrorResponse")
                .error,
            "Invalid credentials".to_string(),
        );
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let email = get_random_email();

    let signup_body = serde_json::json!({
        "email": &email,
        "password": "password123",
        "requires2FA": true,
    });

    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": &email,
        "password": "password123",
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 206);

    let json_body = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body into TwoFactorAuthResponse");

    let verify_2fa_body = serde_json::json!({
        "email": &email,
        "loginAttemptId": json_body.login_attempt_id,
        "2FACode": "999999",
    });

    let verify_2fa_response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(verify_2fa_response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let app = TestApp::new().await;

    let email = get_random_email();

    let signup_body = serde_json::json!({
        "email": &email,
        "password": "password123",
        "requires2FA": true,
    });

    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": &email,
        "password": "password123",
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 206);

    let json_body = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body into TwoFactorAuthResponse");

    let old_2fa_code = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(email.to_string()).unwrap())
        .await
        .unwrap()
        .1;

    let _ = app.post_logout().await;
    let _ = app.post_login(&login_body).await;

    let verify_2fa_body = serde_json::json!({
        "email": &email,
        "loginAttemptId": json_body.login_attempt_id,
        "2FACode": old_2fa_code.as_ref().to_string(),
    });

    let verify_2fa_response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(verify_2fa_response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let app = TestApp::new().await;

    let email = get_random_email();

    let signup_body = serde_json::json!({
        "email": &email,
        "password": "password123",
        "requires2FA": true,
    });

    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": &email,
        "password": "password123",
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 206);

    let json_body = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body into TwoFactorAuthResponse");

    let current_2fa_code = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(email.to_string()).unwrap())
        .await
        .unwrap()
        .1;

    let verify_2fa_body = serde_json::json!({
        "email": &email,
        "loginAttemptId": json_body.login_attempt_id,
        "2FACode": current_2fa_code.as_ref().to_string(),
    });

    let verify_2fa_response = app.post_verify_2fa(&verify_2fa_body).await;

    let auth_cookie = verify_2fa_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    assert_eq!(verify_2fa_response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let app = TestApp::new().await;

    let email = get_random_email();

    let signup_body = serde_json::json!({
        "email": &email,
        "password": "password123",
        "requires2FA": true,
    });

    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": &email,
        "password": "password123",
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 206);

    let json_body = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body into TwoFactorAuthResponse");

    let current_2fa_code = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(email.to_string()).unwrap())
        .await
        .unwrap()
        .1;

    let verify_2fa_body = serde_json::json!({
        "email": &email,
        "loginAttemptId": json_body.login_attempt_id,
        "2FACode": current_2fa_code.as_ref().to_string(),
    });

    let verify_2fa_response = app.post_verify_2fa(&verify_2fa_body).await;

    let auth_cookie = verify_2fa_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    assert_eq!(verify_2fa_response.status().as_u16(), 200);

    let verify_2fa_response = app.post_verify_2fa(&verify_2fa_body).await;

    assert_eq!(verify_2fa_response.status().as_u16(), 401);
}
