use crate::auth::Credentials;
use crate::state::AppState;

use super::common::{verify_request_auth, ServiceError, ServiceResult};

/// Get the latest nonce for an account (returns None if no deltas exist)
pub async fn get_latest_nonce(
    state: &AppState,
    account_id: &str,
    credentials: Credentials,
) -> ServiceResult<Option<u64>> {
    // Verify account exists
    let account_metadata = state
        .metadata
        .get(account_id)
        .await
        .map_err(|e| ServiceError::new(format!("Failed to check account: {e}")))?
        .ok_or_else(|| ServiceError::new(format!("Account '{account_id}' not found")))?;

    // Verify authentication and authorization
    verify_request_auth(
        &account_metadata.auth,
        &account_metadata,
        account_id,
        &credentials,
    )?;

    let delta_files = state
        .storage
        .list_deltas(account_id)
        .await
        .map_err(|e| ServiceError::new(format!("Failed to list deltas: {e}")))?;

    if delta_files.is_empty() {
        return Ok(None);
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

    Ok(max_nonce)
}
