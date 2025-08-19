use auth_service::utils::JWT_COOKIE_NAME;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

    let response = app.verify_token(&serde_json::json!({})).await;

    assert_eq!(response.status().as_u16(), 422);

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_200_if_valid_token() {
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

    let response = app
        .verify_token(&serde_json::json!({
            "token": auth_cookie.value()
        }))
        .await;

    assert_eq!(response.status().as_u16(), 200);

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
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

    let response = app
        .verify_token(&serde_json::json!({
            "token": "invalid_token"
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
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

    // Simulate banning the token
    let _ = app
        .app_state
        .banned_token_store
        .write()
        .await
        .add_token(auth_cookie.value())
        .await;

    let response = app
        .verify_token(&serde_json::json!({
            "token": auth_cookie.value()
        }))
        .await;

    assert_eq!(response.status().as_u16(), 401);

    app.cleanup().await;
}
