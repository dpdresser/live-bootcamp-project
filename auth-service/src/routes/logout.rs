use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{domain::AuthAPIError, utils::validate_token};

pub async fn logout(jar: CookieJar) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
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

    // Remove the cookie and return success
    let updated_jar = jar.remove(crate::utils::JWT_COOKIE_NAME);

    Ok((updated_jar, StatusCode::OK.into_response()))
}
