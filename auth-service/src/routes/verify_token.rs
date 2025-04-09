use axum::{http::StatusCode, response::IntoResponse};

pub async fn post_verify_token() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
