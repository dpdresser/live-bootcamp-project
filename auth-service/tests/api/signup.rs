use auth_service::{routes::SignupResponse, ErrorResponse};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": random_email,
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password123",
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();

    let response = app
        .signup(&serde_json::json!({
            "email": random_email,
            "password": "validPass123!",
            "requires2FA": true,
        }))
        .await;

    assert_eq!(
        response.status().as_u16(),
        201,
        "Failed to sign up with valid input"
    );

    let expected_response = SignupResponse {
        message: "User created successfully".to_string(),
    };

    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to UserBody"),
        expected_response,
    );

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();
    let test_cases = [
        // Password too short
        serde_json::json!({
            "email": random_email,
            "password": "short1!",
            "requires2FA": true,
        }),
        // Password missing number
        serde_json::json!({
            "email": random_email,
            "password": "password!",
            "requires2FA": true,
        }),
        // Password missing special character
        serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true,
        }),
        // Invalid email format
        serde_json::json!({
            "email": "invalid-email",
            "password": "validPass123!",
            "requires2FA": true,
        }),
        // Invalid email - missing @
        serde_json::json!({
            "email": "invalid.email.com",
            "password": "validPass123!",
            "requires2FA": true,
        }),
        // Invalid email - missing domain
        serde_json::json!({
            "email": "user@",
            "password": "validPass123!",
            "requires2FA": true,
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_string(),
        );
    }

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let response = app
        .signup(&serde_json::json!({
            "email": random_email,
            "password": "validPass123!",
            "requires2FA": true,
        }))
        .await;

    assert_eq!(
        response.status().as_u16(),
        201,
        "Failed to sign up with valid input"
    );

    let response = app
        .signup(&serde_json::json!({
            "email": random_email,
            "password": "validPass123!",
            "requires2FA": true,
        }))
        .await;

    assert_eq!(
        response.status().as_u16(),
        409,
        "Failed to return 409 for duplicate email"
    );

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_string(),
    );

    app.cleanup().await;
}
