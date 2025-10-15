mod miden_falcon_rpo;

use crate::storage::AccountMetadata;

/// Trait for extracting authentication credentials from request metadata
/// Implemented by HTTP headers and gRPC metadata
pub trait ExtractCredentials {
    type Error;

    /// Extract credentials from the metadata source
    fn extract_credentials(&self) -> Result<Credentials, Self::Error>;
}

/// Authentication credentials enum - extensible for different auth methods
#[derive(Debug, Clone)]
pub enum Credentials {
    /// Public key signature-based authentication
    /// Used for cryptographic signature verification (e.g., Falcon, ECDSA, etc.)
    Signature { pubkey: String, signature: String },
}

impl Credentials {
    pub fn signature(pubkey: String, signature: String) -> Self {
        Self::Signature { pubkey, signature }
    }

    pub fn as_signature(&self) -> Option<(&str, &str)> {
        match self {
            Self::Signature { pubkey, signature } => Some((pubkey, signature)),
        }
    }
}

/// Authentication and authorization handler
/// Defines which signature scheme to use and handles verification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum Auth {
    /// Miden Falcon RPO signature scheme
    MidenFalconRpo,
}

impl Auth {
    /// Verify credentials are authorized for account
    ///
    /// # Arguments
    /// * `account_id` - The account ID
    /// * `credentials` - The credentials to verify
    /// * `account_metadata` - The account metadata containing authorization info
    pub fn verify(
        &self,
        account_id: &str,
        credentials: &Credentials,
        account_metadata: &AccountMetadata,
    ) -> Result<(), String> {
        match self {
            Auth::MidenFalconRpo => {
                let (pubkey, signature) = credentials
                    .as_signature()
                    .ok_or_else(|| "MidenFalconRpo requires signature credentials".to_string())?;

                if !account_metadata
                    .cosigner_pubkeys
                    .contains(&pubkey.to_string())
                {
                    return Err(format!("Public key '{pubkey}' is not authorized"));
                }

                miden_falcon_rpo::verify_request_signature(account_id, pubkey, signature)
            }
        }
    }
}

// Implementation for HTTP headers (axum::http::HeaderMap)
impl ExtractCredentials for axum::http::HeaderMap {
    type Error = String;

    fn extract_credentials(&self) -> Result<Credentials, Self::Error> {
        let pubkey = self
            .get("x-pubkey")
            .ok_or_else(|| "Missing x-pubkey header".to_string())?
            .to_str()
            .map_err(|_| "Invalid x-pubkey header".to_string())?
            .to_string();

        let signature = self
            .get("x-signature")
            .ok_or_else(|| "Missing x-signature header".to_string())?
            .to_str()
            .map_err(|_| "Invalid x-signature header".to_string())?
            .to_string();

        Ok(Credentials::signature(pubkey, signature))
    }
}

// Implementation for gRPC metadata (tonic::metadata::MetadataMap)
impl ExtractCredentials for tonic::metadata::MetadataMap {
    type Error = tonic::Status;

    fn extract_credentials(&self) -> Result<Credentials, Self::Error> {
        let pubkey = self
            .get("x-pubkey")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| tonic::Status::invalid_argument("Missing or invalid x-pubkey metadata"))?
            .to_string();

        let signature = self
            .get("x-signature")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                tonic::Status::invalid_argument("Missing or invalid x-signature metadata")
            })?
            .to_string();

        Ok(Credentials::signature(pubkey, signature))
    }
}
