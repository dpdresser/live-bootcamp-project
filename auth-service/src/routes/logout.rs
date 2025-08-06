use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{app_state::AppState, domain::AuthAPIError, utils::validate_token};

pub async fn logout(
    State(app_state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    // Retrieve JWT cookie
    // If no cookie exists, user is already logged out - return error
    let cookie = jar
        .get(crate::utils::JWT_COOKIE_NAME)
        .ok_or(AuthAPIError::MissingToken)?;

    let token = cookie.value().to_string();

    // Validate the token
    // If token is invalid, user is already logged out - return error
    let _ = validate_token(&token)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    // Add token to banned token store
    app_state
        .banned_token_store
        .write()
        .await
        .add_token(&token)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    // Remove the cookie and return success
    let updated_jar = jar.remove(crate::utils::JWT_COOKIE_NAME);

    Ok((updated_jar, StatusCode::OK.into_response()))
}
