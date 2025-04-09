use axum::{http::StatusCode, response::IntoResponse};

pub async fn post_verify_2fa() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
