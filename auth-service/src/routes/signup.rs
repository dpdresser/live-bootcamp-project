use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password, User, UserStoreError},
};

#[tracing::instrument(name = "Signup", skip_all, err(Debug))]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(&request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(&request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let user = User::new(email, password, request.requires_2fa);

    match state.user_store.write().await.add_user(user).await {
        Ok(_) => {
            let response = Json(SignupResponse {
                message: "User created successfully".to_string(),
            });

            Ok((StatusCode::CREATED, response))
        }
        Err(UserStoreError::UserAlreadyExists) => Err(AuthAPIError::UserAlreadyExists),
        _ => Err(AuthAPIError::UnexpectedError),
    }
}

#[derive(Deserialize)]
pub struct SignupRequest {
    email: String,
    password: String,
    #[serde(rename = "requires2FA")]
    requires_2fa: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}
