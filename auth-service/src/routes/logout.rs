use axum::{http::StatusCode, response::IntoResponse};

pub async fn post_logout() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
