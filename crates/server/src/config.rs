use crate::storage::filesystem::{FilesystemConfig, FilesystemService};
use crate::storage::StorageBackend;
use std::sync::Arc;

/// Initialize storage backend based on configuration
pub async fn initialize_storage() -> Result<Arc<dyn StorageBackend>, String> {
    // For now, only filesystem storage is supported
    // In the future, we can read from env var to determine storage type
    println!("Initializing filesystem storage...");
    let fs_config = FilesystemConfig::from_env()?;
    let fs_service = FilesystemService::new(fs_config).await?;

    Ok(Arc::new(fs_service))
}
