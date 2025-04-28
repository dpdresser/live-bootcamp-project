use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, Password, TwoFACode},
    utils::auth::generate_auth_cookie,
};

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let email =
        Email::parse(request.email.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let password =
        Password::parse(request.password.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user_store = state.user_store.read().await;

    if user_store.validate_user(&email, &password).await.is_err() {
        return Err(AuthAPIError::IncorrectCredentials);
    }

    let user = user_store
        .get_user(&email)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    match user.requires_2fa {
        true => handle_2fa(&email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }

    // let auth_cookie =
    //     generate_auth_cookie(&user.email).map_err(|_| AuthAPIError::UnexpectedError)?;
    // let updated_jar = jar.add(auth_cookie);

    // Ok((updated_jar, StatusCode::OK.into_response()))
}

async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    state
        .two_fa_code_store
        .write()
        .await
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    Ok((
        jar,
        (
            StatusCode::PARTIAL_CONTENT,
            LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
                message: "2FA required".to_string(),
                login_attempt_id: login_attempt_id.as_ref().to_string(),
            })
            .into(),
        ),
    ))
}

async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let auth_cookie = generate_auth_cookie(email).map_err(|_| AuthAPIError::UnexpectedError)?;
    let updated_jar = jar.add(auth_cookie);

    Ok((
        updated_jar,
        (StatusCode::OK, LoginResponse::RegularAuth.into()),
    ))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
