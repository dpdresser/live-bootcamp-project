use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, domain::User};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> impl IntoResponse {
    let user = User::new(request.email, request.password, request.requires_2fa);

    state.user_store.write().await.add_user(user).unwrap();

    let response = Json(SignupResponse {
        message: "User created successfully".to_string(),
    });

    (StatusCode::CREATED, response)
}

#[derive(Deserialize)]
pub struct SignupRequest {
    email: String,
    password: String,
    #[serde(rename = "requires2FA")]
    requires_2fa: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}
