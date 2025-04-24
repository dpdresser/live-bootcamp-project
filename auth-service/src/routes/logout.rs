use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let cookie = jar.get(JWT_COOKIE_NAME).ok_or(AuthAPIError::MissingToken)?;
    let token = cookie.value().to_string();

    validate_token(&token, state.banned_token_list.clone())
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    state
        .banned_token_list
        .write()
        .await
        .ban_token(token.clone())
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    let updated_jar = jar.remove(JWT_COOKIE_NAME);

    Ok((updated_jar, StatusCode::OK))
}
