use crate::auth::Credentials;
use crate::canonicalization::CanonicalizationMode;
use crate::state::AppState;
use crate::storage::{AccountState, DeltaObject, StorageBackend};
use std::sync::Arc;

use super::{ServiceError, ServiceResult};

#[derive(Debug, Clone)]
pub struct PushDeltaParams {
    pub delta: DeltaObject,
    pub credentials: Credentials,
}

#[derive(Debug, Clone)]
pub struct PushDeltaResult {
    pub delta: DeltaObject,
}

pub async fn push_delta(
    state: &AppState,
    params: PushDeltaParams,
) -> ServiceResult<PushDeltaResult> {
    let account_metadata = state
        .metadata
        .get(&params.delta.account_id)
        .await
        .map_err(|e| ServiceError::new(format!("Failed to check account: {e}")))?
        .ok_or_else(|| {
            ServiceError::new(format!("Account '{}' not found", params.delta.account_id))
        })?;

    account_metadata
        .auth
        .verify(&params.delta.account_id, &params.credentials)
        .map_err(|e| ServiceError::new(format!("Authentication failed: {e}")))?;

    let storage_backend = state
        .storage
        .get(&account_metadata.storage_type)
        .map_err(ServiceError::new)?;

    check_no_pending_candidates(&storage_backend, &params.delta.account_id).await?;

    let current_state = storage_backend
        .pull_state(&params.delta.account_id)
        .await
        .map_err(|e| ServiceError::new(format!("Failed to fetch account state: {e}")))?;

    verify_prev_commitment(&params.delta, &current_state)?;

    let new_commitment =
        calculate_new_commitment(state, &current_state, &params.delta.delta_payload).await?;

    let now = chrono::Utc::now().to_rfc3339();

    match &state.canonicalization_mode {
        CanonicalizationMode::Enabled(_) => {
            save_as_candidate(&storage_backend, &params.delta, &new_commitment, &now).await?;
        }
        CanonicalizationMode::Optimistic => {
            save_as_canonical(
                state,
                &storage_backend,
                &params.delta,
                &current_state,
                &new_commitment,
                &now,
            )
            .await?;
        }
    }

    Ok(PushDeltaResult {
        delta: params.delta,
    })
}

async fn check_no_pending_candidates(
    storage_backend: &Arc<dyn StorageBackend>,
    account_id: &str,
) -> ServiceResult<()> {
    let all_deltas = storage_backend
        .pull_deltas_after(account_id, 0)
        .await
        .map_err(|e| ServiceError::new(format!("Failed to check deltas: {e}")))?;

    eprintln!("DEBUG: Checking {} existing deltas", all_deltas.len());
    for delta in &all_deltas {
        eprintln!(
            "  Delta {}: candidate_at={:?}, canonical_at={:?}, discarded_at={:?}",
            delta.nonce, delta.candidate_at, delta.canonical_at, delta.discarded_at
        );
    }

    let has_pending_candidate = all_deltas
        .iter()
        .any(|d| d.candidate_at.is_some() && d.canonical_at.is_none() && d.discarded_at.is_none());

    eprintln!("DEBUG: has_pending_candidate = {has_pending_candidate}");

    if has_pending_candidate {
        return Err(ServiceError::new(
            "Cannot push new delta: there is already a non-canonical delta pending. Wait for canonicalization or discard the pending delta.".to_string()
        ));
    }

    Ok(())
}

fn verify_prev_commitment(delta: &DeltaObject, current_state: &AccountState) -> ServiceResult<()> {
    if delta.prev_commitment != current_state.commitment {
        return Err(ServiceError::new(format!(
            "Delta prev_commitment mismatch: expected {}, got {}",
            current_state.commitment, delta.prev_commitment
        )));
    }
    Ok(())
}

async fn calculate_new_commitment(
    state: &AppState,
    current_state: &AccountState,
    delta_payload: &serde_json::Value,
) -> ServiceResult<String> {
    let client = state.network_client.lock().await;

    client
        .verify_delta(
            &current_state.commitment,
            &current_state.state_json,
            delta_payload,
        )
        .map_err(|e| ServiceError::new(format!("Delta verification failed: {e}")))?;

    let (_, commitment) = client
        .apply_delta(&current_state.state_json, delta_payload)
        .map_err(|e| ServiceError::new(format!("Failed to calculate commitment: {e}")))?;

    Ok(commitment)
}

async fn save_as_candidate(
    storage_backend: &Arc<dyn StorageBackend>,
    delta: &DeltaObject,
    new_commitment: &str,
    timestamp: &str,
) -> ServiceResult<()> {
    let mut candidate_delta = delta.clone();
    candidate_delta.new_commitment = new_commitment.to_string();
    candidate_delta.candidate_at = Some(timestamp.to_string());

    storage_backend
        .submit_delta(&candidate_delta)
        .await
        .map_err(|e| ServiceError::new(format!("Failed to submit delta: {e}")))
}

async fn save_as_canonical(
    state: &AppState,
    storage_backend: &Arc<dyn StorageBackend>,
    delta: &DeltaObject,
    current_state: &AccountState,
    new_commitment: &str,
    timestamp: &str,
) -> ServiceResult<()> {
    let mut canonical_delta = delta.clone();
    canonical_delta.new_commitment = new_commitment.to_string();
    canonical_delta.candidate_at = Some(timestamp.to_string());
    canonical_delta.canonical_at = Some(timestamp.to_string());

    let (new_state_json, _) = {
        let client = state.network_client.lock().await;
        client
            .apply_delta(&current_state.state_json, &canonical_delta.delta_payload)
            .map_err(|e| ServiceError::new(format!("Failed to apply delta: {e}")))?
    };

    let new_state = AccountState {
        account_id: canonical_delta.account_id.clone(),
        commitment: new_commitment.to_string(),
        state_json: new_state_json,
        created_at: current_state.created_at.clone(),
        updated_at: timestamp.to_string(),
    };

    storage_backend
        .submit_state(&new_state)
        .await
        .map_err(|e| ServiceError::new(format!("Failed to update state: {e}")))?;

    storage_backend
        .submit_delta(&canonical_delta)
        .await
        .map_err(|e| ServiceError::new(format!("Failed to submit delta: {e}")))?;

    Ok(())
}
