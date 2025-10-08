use crate::state::AppState;
use crate::storage::DeltaObject;
use axum::{extract::Query, extract::State, http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeltaQuery {
    pub account_id: String,
    pub nonce: u64,
}

pub async fn get_delta(
    State(state): State<AppState>,
    Query(query): Query<DeltaQuery>,
) -> (StatusCode, Json<DeltaObject>) {
    match state
        .storage
        .pull_delta(&query.account_id, query.nonce)
        .await
    {
        Ok(delta) => (StatusCode::OK, Json(delta)),
        Err(e) => {
            eprintln!("Failed to pull delta: {}", e);
            (StatusCode::NOT_FOUND, Json(DeltaObject::default()))
        }
    }
}
