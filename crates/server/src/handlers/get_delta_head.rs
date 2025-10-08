use crate::state::AppState;
use crate::storage::DeltaObject;
use axum::{extract::Query, extract::State, http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct StateQuery {
    pub account_id: String,
}

pub async fn get_delta_head(
    State(state): State<AppState>,
    Query(query): Query<StateQuery>,
) -> (StatusCode, Json<DeltaObject>) {
    match state.storage.list_deltas(&query.account_id).await {
        Ok(deltas) => {
            if let Some(last_delta_file) = deltas.last() {
                // Extract nonce from filename (e.g., "123.json" -> 123)
                if let Some(nonce_str) = last_delta_file.strip_suffix(".json") {
                    if let Ok(nonce) = nonce_str.parse::<u64>() {
                        match state.storage.pull_delta(&query.account_id, nonce).await {
                            Ok(delta) => return (StatusCode::OK, Json(delta)),
                            Err(e) => eprintln!("Failed to pull head delta: {}", e),
                        }
                    }
                }
            }
            (StatusCode::NOT_FOUND, Json(DeltaObject::default()))
        }
        Err(e) => {
            eprintln!("Failed to list deltas: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeltaObject::default()),
            )
        }
    }
}
