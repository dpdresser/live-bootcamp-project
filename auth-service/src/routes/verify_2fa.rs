use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
    utils::auth::generate_auth_cookie,
};

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let email =
        Email::parse(request.email.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let login_attempt_id = LoginAttemptId::parse(request.login_attempt_id.clone())
        .map_err(|_| AuthAPIError::InvalidCredentials)?;
    let two_fa_code = TwoFACode::parse(request.two_fa_code.clone())
        .map_err(|_| AuthAPIError::InvalidCredentials)?;

    let code_tuple = state
        .two_fa_code_store
        .write()
        .await
        .get_code(&email)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    if login_attempt_id == code_tuple.0 && two_fa_code == code_tuple.1 {
        state
            .two_fa_code_store
            .write()
            .await
            .remove_code(&email)
            .await
            .map_err(|_| AuthAPIError::UnexpectedError)?;

        let auth_cookie =
            generate_auth_cookie(&email).map_err(|_| AuthAPIError::UnexpectedError)?;
        let updated_jar = jar.add(auth_cookie);

        Ok((updated_jar, StatusCode::OK))
    } else {
        Err(AuthAPIError::IncorrectCredentials)
    }
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}
