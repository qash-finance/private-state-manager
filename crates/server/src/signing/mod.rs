pub mod miden_falcon_rpo;

use crate::error::PsmError;
use crate::storage::DeltaObject;
use miden_objects::{Felt, Word};
use miden_objects::crypto::dsa::rpo_falcon512::{PublicKey, SecretKey, Signature};
use miden_objects::crypto::hash::rpo::Rpo256;
use miden_objects::utils::Serializable;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, PsmError>;

#[derive(Debug, Clone)]
pub enum KeystoreConfig {
    Filesystem(PathBuf),
}

/// Main signer - provides cryptographic signing operations
#[derive(Clone)]
pub enum Signer {
    MidenFalconRpo(miden_falcon_rpo::MidenFalconRpoSigner),
}

impl Signer {
    /// Create a new Miden Falcon RPO signer with filesystem keystore
    pub fn miden_falcon_rpo(keystore: KeystoreConfig) -> Result<Self> {
        match keystore {
            KeystoreConfig::Filesystem(path) => Ok(Signer::MidenFalconRpo(
                miden_falcon_rpo::MidenFalconRpoSigner::new(path)?,
            )),
        }
    }

    /// Sign a message with the server's signing key
    pub fn sign_with_server_key(&self, message: Word) -> Result<Signature> {
        match self {
            Signer::MidenFalconRpo(signer) => signer.sign_with_server_key(message),
        }
    }

    /// Get the server's public key
    pub fn server_pubkey(&self) -> PublicKey {
        match self {
            Signer::MidenFalconRpo(signer) => signer.server_pubkey(),
        }
    }

    /// Get the server's public key as a hex string
    pub fn server_pubkey_hex(&self) -> String {
        let pubkey = self.server_pubkey();
        let pubkey_word: Word = pubkey.into();
        format!("0x{}", hex::encode(pubkey_word.to_bytes()))
    }

    /// Sign a delta with the server key and return it with ack_sig loaded
    pub fn ack_delta(&self, mut delta: DeltaObject) -> Result<DeltaObject> {
        let commitment_digest = commitment_to_digest(&delta.new_commitment)?;
        let signature = self.sign_with_server_key(commitment_digest)?;
        delta.ack_sig = Some(hex::encode(signature.to_bytes()));
        Ok(delta)
    }

    /// Add a key to the keystore
    pub fn add_key(&self, key: &SecretKey) -> Result<()> {
        match self {
            Signer::MidenFalconRpo(signer) => signer.add_key(key),
        }
    }

    /// Get a key from the keystore by its public key
    pub fn get_key(&self, pub_key: Word) -> Result<SecretKey> {
        match self {
            Signer::MidenFalconRpo(signer) => signer.get_key(pub_key),
        }
    }

    /// Sign a message with a specific key from the keystore
    pub fn sign(&self, pub_key: Word, message: Word) -> Result<Signature> {
        match self {
            Signer::MidenFalconRpo(signer) => signer.sign(pub_key, message),
        }
    }
}

fn commitment_to_digest(commitment_hex: &str) -> Result<Word> {
    let commitment_hex = commitment_hex.strip_prefix("0x").unwrap_or(commitment_hex);

    let bytes = hex::decode(commitment_hex)
        .map_err(|e| PsmError::InvalidCommitment(format!("Invalid hex: {e}")))?;

    if bytes.len() != 32 {
        return Err(PsmError::InvalidCommitment(format!(
            "Commitment must be 32 bytes, got {}",
            bytes.len()
        )));
    }

    let mut felts = Vec::new();
    for chunk in bytes.chunks(8) {
        let mut arr = [0u8; 8];
        arr[..chunk.len()].copy_from_slice(chunk);
        let value = u64::from_le_bytes(arr);
        felts.push(
            Felt::try_from(value)
                .map_err(|e| PsmError::InvalidCommitment(format!("Invalid field element: {e}")))?,
        );
    }

    let message_elements = vec![felts[0], felts[1], felts[2], felts[3]];

    let digest = Rpo256::hash_elements(&message_elements);
    Ok(digest)
}
