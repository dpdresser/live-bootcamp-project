use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

pub async fn signup(Json(_request): Json<SignupRequest>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}

#[derive(Deserialize)]
pub struct SignupRequest {
    _email: String,
    _password: String,
    #[serde(rename = "requires2FA")]
    _requires_2fa: bool,
}
