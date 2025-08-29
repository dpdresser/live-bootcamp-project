use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
    utils::generate_auth_cookie,
};

#[tracing::instrument(name = "Login", skip_all)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    // Validate email format
    let email = Email::parse(&request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;

    // Basic password validation - at least not empty
    if request.password.is_empty() {
        return Err(AuthAPIError::InvalidCredentials);
    }

    // Get user and validate password
    let user_store = state.user_store.read().await;

    let user = user_store
        .get_user(&email)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    user_store
        .validate_user(&email, &request.password)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    // Handle authentication based on 2FA requirement
    match user.requires_2fa() {
        true => {
            let (jar, response) = handle_2fa(&email, &state, jar).await?;
            Ok((jar, (StatusCode::PARTIAL_CONTENT, response)))
        }
        false => {
            let (jar, response) = handle_no_2fa(&email, jar).await?;
            Ok((jar, (StatusCode::OK, response)))
        }
    }
}

#[tracing::instrument(name = "Handle 2FA", skip_all)]
async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> Result<(CookieJar, Json<LoginResponse>), AuthAPIError> {
    // Generate new random login attempt ID and 2FA code
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    // Store ID and code in 2FA code store.
    state
        .two_fa_code_store
        .write()
        .await
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;

    // Send 2FA code via the email client. Return AuthAPIError::UnexpectedError if it fails.
    let email_client = state.email_client.write().await;
    email_client
        .send_email(email, "Your 2FA code", two_fa_code.as_ref())
        .await
        .map_err(AuthAPIError::UnexpectedError)?;

    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_string(),
        login_attempt_id: login_attempt_id.as_ref().to_string(),
    }));

    Ok((jar, response))
}

#[tracing::instrument(name = "Handle No 2FA", skip_all)]
async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> Result<(CookieJar, Json<LoginResponse>), AuthAPIError> {
    let auth_cookie = generate_auth_cookie(email).map_err(AuthAPIError::UnexpectedError)?;

    let updated_jar = jar.add(auth_cookie);

    let response = Json(LoginResponse::RegularAuth);
    Ok((updated_jar, response))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
