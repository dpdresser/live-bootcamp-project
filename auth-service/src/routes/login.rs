use axum::{http::StatusCode, response::IntoResponse};

pub async fn post_login() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
