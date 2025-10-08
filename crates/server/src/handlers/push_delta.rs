use crate::state::AppState;
use crate::storage::DeltaObject;
use axum::{extract::State, http::StatusCode, Json};

pub async fn push_delta(
    State(state): State<AppState>,
    Json(payload): Json<DeltaObject>,
) -> (StatusCode, Json<DeltaObject>) {
    match state.storage.submit_delta(&payload).await {
        Ok(_) => (StatusCode::OK, Json(payload)),
        Err(e) => {
            eprintln!("Failed to submit delta: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(payload))
        }
    }
}
