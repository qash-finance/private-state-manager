use super::keystore::FilesystemKeyStore;
use crate::error::MidenFalconRpoError;
use miden_objects::{
    Word,
    crypto::dsa::rpo_falcon512::{PublicKey, SecretKey, Signature},
};
use rand_chacha::ChaCha20Rng;
use std::path::PathBuf;
use std::sync::Arc;

type Result<T> = std::result::Result<T, MidenFalconRpoError>;

#[derive(Clone)]
pub struct MidenFalconRpoSigner {
    keystore: Arc<FilesystemKeyStore<ChaCha20Rng>>,
    server_pubkey_word: Word,
}

impl MidenFalconRpoSigner {
    pub fn new(keystore_path: PathBuf) -> Result<Self> {
        let keystore = FilesystemKeyStore::<ChaCha20Rng>::new(keystore_path)?;
        let keystore = Arc::new(keystore);
        let server_pubkey_word = keystore.generate_key()?;

        Ok(Self {
            keystore,
            server_pubkey_word,
        })
    }
}

impl MidenFalconRpoSigner {
    pub(crate) fn sign_with_server_key(&self, message: Word) -> crate::signing::Result<Signature> {
        Ok(self.keystore.sign(self.server_pubkey_word, message)?)
    }

    pub(crate) fn server_pubkey(&self) -> PublicKey {
        let secret_key = self
            .keystore
            .get_key(self.server_pubkey_word)
            .expect("Server key must exist in keystore");
        secret_key.public_key()
    }

    pub(crate) fn add_key(&self, key: &SecretKey) -> crate::signing::Result<()> {
        Ok(self.keystore.add_key(key)?)
    }

    pub(crate) fn get_key(&self, pub_key: Word) -> crate::signing::Result<SecretKey> {
        Ok(self.keystore.get_key(pub_key)?)
    }

    pub(crate) fn sign(&self, pub_key: Word, message: Word) -> crate::signing::Result<Signature> {
        Ok(self.keystore.sign(pub_key, message)?)
    }
}
