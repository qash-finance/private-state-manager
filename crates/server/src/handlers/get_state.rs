use crate::state::AppState;
use crate::storage::AccountState;
use axum::{extract::Query, extract::State, http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct StateQuery {
    pub account_id: String,
}

pub async fn get_state(
    State(state): State<AppState>,
    Query(query): Query<StateQuery>,
) -> (StatusCode, Json<AccountState>) {
    match state.storage.pull_state(&query.account_id).await {
        Ok(account_state) => (StatusCode::OK, Json(account_state)),
        Err(e) => {
            eprintln!("Failed to pull state: {}", e);
            (StatusCode::NOT_FOUND, Json(AccountState::default()))
        }
    }
}
