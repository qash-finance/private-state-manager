use crate::auth::Credentials;
use crate::state::AppState;
use crate::storage::DeltaObject;

use super::common::{verify_request_auth, ServiceError, ServiceResult};

#[derive(Debug, Clone)]
pub struct GetDeltaHeadParams {
    pub account_id: String,
    pub credentials: Credentials,
}

#[derive(Debug, Clone)]
pub struct GetDeltaHeadResult {
    pub delta: DeltaObject,
}

/// Get the latest delta (head) for an account
pub async fn get_delta_head(
    state: &AppState,
    params: GetDeltaHeadParams,
) -> ServiceResult<GetDeltaHeadResult> {
    // Verify account exists
    let account_metadata = state
        .metadata
        .get(&params.account_id)
        .await
        .map_err(|e| ServiceError::new(format!("Failed to check account: {e}")))?
        .ok_or_else(|| ServiceError::new(format!("Account '{}' not found", params.account_id)))?;

    // Verify authentication and authorization
    verify_request_auth(
        &account_metadata.auth,
        &account_metadata,
        &params.account_id,
        &params.credentials,
    )?;

    let delta_files = state
        .storage
        .list_deltas(&params.account_id)
        .await
        .map_err(|e| ServiceError::new(format!("Failed to list deltas: {e}")))?;

    if delta_files.is_empty() {
        return Err(ServiceError::new(format!(
            "No deltas found for account '{}'",
            params.account_id
        )));
    }

    // Parse nonces from filenames and find the maximum
    let mut max_nonce: Option<u64> = None;
    for filename in &delta_files {
        if let Some(nonce_str) = filename.strip_suffix(".json") {
            if let Ok(nonce) = nonce_str.parse::<u64>() {
                max_nonce = Some(max_nonce.map_or(nonce, |current| current.max(nonce)));
            }
        }
    }

    let latest_nonce = max_nonce
        .ok_or_else(|| ServiceError::new("Failed to parse nonces from delta files".to_string()))?;

    // Fetch the latest delta
    let delta = state
        .storage
        .pull_delta(&params.account_id, latest_nonce)
        .await
        .map_err(|e| ServiceError::new(format!("Failed to fetch latest delta: {e}")))?;

    Ok(GetDeltaHeadResult { delta })
}
