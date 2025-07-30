use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, User},
    services::UserStoreError,
};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    if request.email.is_empty() || !request.email.contains("@") || request.password.len() < 8 {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let user = User::new(request.email, request.password, request.requires_2fa);

    match state.user_store.write().await.add_user(user) {
        Ok(_) => (),
        Err(UserStoreError::UserAlreadyExists) => {
            return Err(AuthAPIError::UserAlreadyExists);
        }
        _ => return Err(AuthAPIError::UnexpectedError),
    }

    let response = Json(SignupResponse {
        message: "User created successfully".to_string(),
    });

    Ok((StatusCode::CREATED, response))
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
