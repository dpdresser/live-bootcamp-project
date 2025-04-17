use auth_service::{routes::SignupResponse, ErrorResponse};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": &random_email,
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": &random_email,
            "password": "password123",
        }),
        serde_json::json!({}),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case,
        );
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;

    let request = serde_json::json!({
        "email": "test@example.com",
        "password": "password123",
        "requires2FA": true,
    });

    let response = app.post_signup(&request).await;

    assert_eq!(response.status().as_u16(), 201);

    let expected_response = SignupResponse {
        message: "User created successfully!".to_string(),
    };

    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialized response body to UserBody"),
        expected_response
    );
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "email": "testexample.com",
            "password": "password123",
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": &random_email,
            "password": "123",
            "requires2FA": true,
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;

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
                .expect("Could not deserialize body into ErrorResponse")
                .error,
            "Invalid credentials".to_string(),
        )
    }
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let app = TestApp::new().await;

    let request = serde_json::json!({
        "email": "test@example.com",
        "password": "password123",
        "requires2FA": true,
    });

    let response = app.post_signup(&request).await;

    assert_eq!(response.status().as_u16(), 201);

    let response = app.post_signup(&request).await;

    assert_eq!(response.status().as_u16(), 409);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize body into ErrorResponse")
            .error,
        "User already exists".to_string(),
    )
}
