use crate::storage::{MetadataStore, StorageBackend};
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<dyn StorageBackend>,
    pub metadata: Arc<dyn MetadataStore>,
}
