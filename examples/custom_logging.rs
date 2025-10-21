use server::builder::ServerBuilder;
use server::canonicalization::{CanonicalizationConfig, CanonicalizationMode};
use server::logging::LoggingConfig;
use server::network::NetworkType;
use server::storage::filesystem::FilesystemMetadataStore;
use server::storage::StorageRegistry;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::Level;

#[tokio::main]
async fn main() {
    let storage_path = PathBuf::from("/tmp/psm-example/storage");
    let metadata_path = PathBuf::from("/tmp/psm-example/metadata");

    let storage_registry = StorageRegistry::with_filesystem(storage_path)
        .await
        .expect("Failed to initialize storage registry");

    let metadata = FilesystemMetadataStore::new(metadata_path)
        .await
        .expect("Failed to initialize metadata store");

    let canonicalization_mode = CanonicalizationMode::Enabled(CanonicalizationConfig::default());

    // Example 1: Default logging (INFO level with env filter)
    ServerBuilder::new()
        .with_logging(LoggingConfig::default())
        .network(NetworkType::MidenTestnet)
        .with_canonicalization(canonicalization_mode.clone())
        .storage(storage_registry.clone())
        .metadata(Arc::new(metadata.clone()))
        .http(true, 3000)
        .grpc(true, 50051)
        .build()
        .await
        .expect("Failed to build server");

    // Example 2: Debug logging
    ServerBuilder::new()
        .with_logging(LoggingConfig::new(Level::DEBUG))
        .network(NetworkType::MidenTestnet)
        .with_canonicalization(canonicalization_mode.clone())
        .storage(storage_registry.clone())
        .metadata(Arc::new(metadata.clone()))
        .http(true, 3000)
        .grpc(true, 50051)
        .build()
        .await
        .expect("Failed to build server");

    // Example 3: Trace logging without env filter override
    ServerBuilder::new()
        .with_logging(LoggingConfig::new(Level::TRACE).with_env_filter(false))
        .network(NetworkType::MidenTestnet)
        .with_canonicalization(canonicalization_mode)
        .storage(storage_registry)
        .metadata(Arc::new(metadata))
        .http(true, 3000)
        .grpc(true, 50051)
        .build()
        .await
        .expect("Failed to build server");
}
