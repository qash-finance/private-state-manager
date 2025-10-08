use crate::state::AppState;
use crate::storage::AccountState;
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfigureRequest {
    pub account_id: String,
    pub initial_state: serde_json::Value,
    pub storage_type: String, // "local" or "S3"
}

pub async fn configure(
    State(state): State<AppState>,
    Json(payload): Json<ConfigureRequest>,
) -> StatusCode {
    // Create initial account state
    let account_state = AccountState {
        account_id: payload.account_id.clone(),
        state_json: payload.initial_state,
        commitment: String::new(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    match state.storage.submit_state(&account_state).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("Failed to configure account: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
