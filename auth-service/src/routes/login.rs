use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, UserStoreError},
};

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Validate email format
    let email = Email::parse(&request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;

    // Basic password validation - at least not empty
    if request.password.is_empty() {
        return Err(AuthAPIError::InvalidCredentials);
    }

    match state
        .user_store
        .read()
        .await
        .validate_user(email, &request.password)
        .await
    {
        Ok(_) => Ok(StatusCode::OK.into_response()),
        Err(UserStoreError::InvalidCredentials) => Err(AuthAPIError::IncorrectCredentials),
        _ => Err(AuthAPIError::UnexpectedError),
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct LoginResponse {
    pub message: String,
}
