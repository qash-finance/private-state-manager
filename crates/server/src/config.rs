use crate::storage::filesystem::{FilesystemConfig, FilesystemMetadataStore, FilesystemService};
use crate::storage::{MetadataStore, StorageBackend};
use std::sync::Arc;

/// Initialize storage backend based on configuration
pub async fn initialize_storage() -> Result<Arc<dyn StorageBackend>, String> {
    // For now, only filesystem storage is supported
    // In the future, we will support other storage types.
    println!("Initializing filesystem storage...");
    let fs_config = FilesystemConfig::from_env()?;
    let fs_service = FilesystemService::new(fs_config).await?;

    Ok(Arc::new(fs_service))
}

/// Initialize metadata store
pub async fn initialize_metadata() -> Result<Arc<dyn MetadataStore>, String> {
    println!("Initializing metadata store...");
    let fs_config = FilesystemConfig::from_env()?;
    let metadata_store = FilesystemMetadataStore::new(fs_config.app_path).await?;

    Ok(Arc::new(metadata_store))
}
