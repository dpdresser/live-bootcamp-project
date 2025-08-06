use auth_service::utils::JWT_COOKIE_NAME;
use reqwest::Url;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let response = app.logout().await;

    assert_eq!(response.status().as_u16(), 400);

    let error_response: auth_service::ErrorResponse = response
        .json()
        .await
        .expect("Could not deserialize response body to ErrorResponse");

    assert_eq!(error_response.error, "Missing token");
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid_token; Path=/; HttpOnly; SameSite=Lax",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.logout().await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let app = TestApp::new().await;

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

    // Extract the JWT token value before logout
    let jwt_token = auth_cookie.value().to_string();

    // Verify token is not banned before logout
    let is_banned_before = app
        .app_state
        .banned_token_store
        .read()
        .await
        .is_token_banned(&jwt_token)
        .await
        .expect("Failed to check if token is banned");

    assert!(
        !is_banned_before,
        "Token should not be banned before logout"
    );

    let response = app.logout().await;

    assert_eq!(response.status().as_u16(), 200);

    let logout_cookie = response
        .cookies()
        .find(|c| c.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found after logout");

    assert!(
        logout_cookie.value().is_empty(),
        "Auth cookie should be empty after logout"
    );

    // Verify the JWT token was added to the banned token store
    let is_banned_after = app
        .app_state
        .banned_token_store
        .read()
        .await
        .is_token_banned(&jwt_token)
        .await
        .expect("Failed to check if token is banned");

    assert!(is_banned_after, "Token should be banned after logout");
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;

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

    let response = app.logout().await;

    assert_eq!(response.status().as_u16(), 200);

    let logout_cookie = response
        .cookies()
        .find(|c| c.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found after logout");

    assert!(
        logout_cookie.value().is_empty(),
        "Auth cookie should be empty after logout"
    );

    let response = app.logout().await;

    assert_eq!(response.status().as_u16(), 400);

    let error_response: auth_service::ErrorResponse = response
        .json()
        .await
        .expect("Could not deserialize response body to ErrorResponse");

    assert_eq!(error_response.error, "Missing token");
}
