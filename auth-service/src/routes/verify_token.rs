use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{app_state::AppState, domain::AuthAPIError, utils::validate_token};

#[tracing::instrument(name = "Verify Token", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    validate_token(&request.token)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    let is_banned = state
        .banned_token_store
        .read()
        .await
        .is_token_banned(&request.token)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;

    if is_banned {
        return Err(AuthAPIError::InvalidToken);
    }

    Ok(StatusCode::OK.into_response())
}

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}
