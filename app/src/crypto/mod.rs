use serde::{Deserialize, Serialize};

use secrecy::{Secret, ExposeSecret};
use rand::{rngs::OsRng, RngCore};

use crate::error::Result;

mod aes;
mod chacha;
mod kdf;
mod envelope;

pub use aes::*;
pub use chacha::*;
pub use kdf::*;
pub use envelope::{envelope_encrypt, envelope_decrypt, LocalKeyEncryptionKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub algorithm: EncryptionAlgorithm,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: [u8; 32],
    pub version: u8,
}

pub struct MasterKey {
    key: Secret<[u8; 32]>,
    algorithm: EncryptionAlgorithm,
}

impl MasterKey {
    pub fn derive_from_passphrase(
        passphrase: &str,
        salt: &[u8; 32],
        algorithm: EncryptionAlgorithm,
    ) -> Result<Self> {
        let key_bytes = derive_key_argon2id(passphrase, salt)?;
        let key = Secret::new(key_bytes);
        
        Ok(Self {
            key,
            algorithm,
        })
    }
    
    #[allow(dead_code)]
    pub fn generate(algorithm: EncryptionAlgorithm) -> Self {
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let key = Secret::new(key_bytes);
        
        Self {
            key,
            algorithm,
        }
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        
        match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let (ciphertext, nonce) = encrypt_aes256gcm(self.key.expose_secret(), plaintext)?;
                Ok(EncryptedData {
                    algorithm: EncryptionAlgorithm::Aes256Gcm,
                    ciphertext,
                    nonce,
                    salt,
                    version: 1,
                })
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let (ciphertext, nonce) = encrypt_chacha20poly1305(self.key.expose_secret(), plaintext)?;
                Ok(EncryptedData {
                    algorithm: EncryptionAlgorithm::ChaCha20Poly1305,
                    ciphertext,
                    nonce,
                    salt,
                    version: 1,
                })
            }
        }
    }
    
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        match encrypted.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                decrypt_aes256gcm(self.key.expose_secret(), &encrypted.ciphertext, &encrypted.nonce)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                decrypt_chacha20poly1305(self.key.expose_secret(), &encrypted.ciphertext, &encrypted.nonce)
            }
        }
    }
    
    #[allow(dead_code)]
    pub fn algorithm(&self) -> &EncryptionAlgorithm {
        &self.algorithm
    }
}

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}

#[allow(dead_code)]
pub fn generate_nonce(size: usize) -> Vec<u8> {
    let mut nonce = vec![0u8; size];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aes256gcm_encryption() {
        let salt = generate_salt();
        let master_key = MasterKey::derive_from_passphrase(
            "test-passphrase",
            &salt,
            EncryptionAlgorithm::Aes256Gcm,
        ).unwrap();
        
        let plaintext = b"secret data";
        let encrypted = master_key.encrypt(plaintext).unwrap();
        let decrypted = master_key.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
    
    #[test]
    fn test_chacha20poly1305_encryption() {
        let salt = generate_salt();
        let master_key = MasterKey::derive_from_passphrase(
            "test-passphrase",
            &salt,
            EncryptionAlgorithm::ChaCha20Poly1305,
        ).unwrap();
        
        let plaintext = b"secret data";
        let encrypted = master_key.encrypt(plaintext).unwrap();
        let decrypted = master_key.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
}