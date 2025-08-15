use auth_service::{
    domain::{Email, LoginAttemptId},
    routes::TwoFactorAuthResponse,
    utils::JWT_COOKIE_NAME,
};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let email = get_random_email();

    // verify_2fa route expects email, loginAttemptId, 2FACode fields
    let test_cases = [
        serde_json::json!({
            "email": email,
            "loginAttemptId": LoginAttemptId::default().as_ref(),
        }),
        serde_json::json!({
            "email": email,
            "2FACode": "123456",
        }),
        serde_json::json!({
            "loginAttemptId": LoginAttemptId::default().as_ref(),
            "2FACode": "123456",
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.verify_2fa(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let test_cases = [
        serde_json::json!({
            "email": "invalid_email",
            "loginAttemptId": LoginAttemptId::default().as_ref(),
            "2FACode": "123456",
        }),
        serde_json::json!({
            "email": get_random_email(),
            "loginAttemptId": "invalid_id",
            "2FACode": "123456",
        }),
        serde_json::json!({
            "email": get_random_email(),
            "loginAttemptId": LoginAttemptId::default().as_ref(),
            "2FACode": "123",
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.verify_2fa(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let email = get_random_email();

    let response = app
        .signup(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
            "requires2FA": true,
        }))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let login_response = app
        .login(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
        }))
        .await;

    assert_eq!(login_response.status().as_u16(), 206);

    let login_response_id = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .unwrap()
        .login_attempt_id;
    let response = app
        .verify_2fa(&serde_json::json!({
            "email": email,
            "loginAttemptId": login_response_id,
            "2FACode": "123456",
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let app = TestApp::new().await;

    let email = get_random_email();

    let response = app
        .signup(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
            "requires2FA": true,
        }))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let login_response = app
        .login(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
        }))
        .await;

    assert_eq!(login_response.status().as_u16(), 206);

    let (login_attempt_id, code) = {
        let two_fa_code_store = app.app_state.two_fa_code_store.write().await;
        two_fa_code_store
            .get_code(&Email::parse(&email).unwrap())
            .await
            .unwrap()
    };

    let response = app
        .verify_2fa(&serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": code.as_ref(),
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);

    let login_response = app
        .login(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
        }))
        .await;

    assert_eq!(login_response.status().as_u16(), 206);

    let (login_attempt_id, _code) = {
        let two_fa_code_store = app.app_state.two_fa_code_store.read().await;
        two_fa_code_store
            .get_code(&Email::parse(&email).unwrap())
            .await
            .unwrap()
    };

    let response = app
        .verify_2fa(&serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": code.as_ref(),
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let app = TestApp::new().await;

    let email = get_random_email();

    let response = app
        .signup(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
            "requires2FA": true,
        }))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let login_response = app
        .login(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
        }))
        .await;

    assert_eq!(login_response.status().as_u16(), 206);

    let (login_attempt_id, code) = {
        let two_fa_code_store = app.app_state.two_fa_code_store.write().await;
        two_fa_code_store
            .get_code(&Email::parse(&email).unwrap())
            .await
            .unwrap()
    };

    let response = app
        .verify_2fa(&serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": code.as_ref(),
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
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let app = TestApp::new().await;

    let email = get_random_email();

    let response = app
        .signup(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
            "requires2FA": true,
        }))
        .await;

    assert_eq!(response.status().as_u16(), 201);

    let login_response = app
        .login(&serde_json::json!({
            "email": email,
            "password": "validPass123!",
        }))
        .await;

    assert_eq!(login_response.status().as_u16(), 206);

    let (login_attempt_id, code) = {
        let two_fa_code_store = app.app_state.two_fa_code_store.write().await;
        two_fa_code_store
            .get_code(&Email::parse(&email).unwrap())
            .await
            .unwrap()
    };

    let response = app
        .verify_2fa(&serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": code.as_ref(),
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

    let response = app
        .verify_2fa(&serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id.as_ref(),
            "2FACode": code.as_ref(),
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);
}
