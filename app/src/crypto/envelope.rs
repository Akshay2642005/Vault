use serde::{Deserialize, Serialize};
use rand::{rngs::OsRng, RngCore};

use crate::error::{VaultError, Result};
use super::{MasterKey, EncryptionAlgorithm, EncryptedData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeEncryption {
    pub encrypted_dek: Vec<u8>,
    pub encrypted_data: EncryptedData,
    pub kek_id: String,
    pub algorithm: EncryptionAlgorithm,
}

#[allow(dead_code)]
pub struct DataEncryptionKey {
    key: [u8; 32],
    #[allow(dead_code)]
    algorithm: EncryptionAlgorithm,
}

#[allow(dead_code)]
impl DataEncryptionKey {
    pub fn generate(algorithm: EncryptionAlgorithm) -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        
        Self {
            key,
            algorithm,
        }
    }
    
    pub fn from_bytes(key: [u8; 32], algorithm: EncryptionAlgorithm) -> Self {
        Self {
            key,
            algorithm,
        }
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        let master_key = MasterKey::derive_from_passphrase(
            &hex::encode(self.key),
            &[0u8; 32], // DEK doesn't need salt
            self.algorithm.clone(),
        )?;
        
        master_key.encrypt(plaintext)
    }
    
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        let master_key = MasterKey::derive_from_passphrase(
            &hex::encode(self.key),
            &[0u8; 32], // DEK doesn't need salt
            self.algorithm.clone(),
        )?;
        
        master_key.decrypt(encrypted)
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

#[allow(dead_code)]
pub trait KeyEncryptionKey {
    fn encrypt_dek(&self, dek: &DataEncryptionKey) -> Result<Vec<u8>>;
    fn decrypt_dek(&self, encrypted_dek: &[u8], algorithm: EncryptionAlgorithm) -> Result<DataEncryptionKey>;
    fn key_id(&self) -> String;
}

#[allow(dead_code)]
pub struct LocalKeyEncryptionKey {
    master_key: MasterKey,
    id: String,
}

#[allow(dead_code)]
impl LocalKeyEncryptionKey {
    pub fn new(master_key: MasterKey, id: String) -> Self {
        Self {
            master_key,
            id,
        }
    }
}

impl KeyEncryptionKey for LocalKeyEncryptionKey {
    fn encrypt_dek(&self, dek: &DataEncryptionKey) -> Result<Vec<u8>> {
        let encrypted = self.master_key.encrypt(dek.as_bytes())?;
        Ok(bincode::serialize(&encrypted)?)
    }
    
    fn decrypt_dek(&self, encrypted_dek: &[u8], algorithm: EncryptionAlgorithm) -> Result<DataEncryptionKey> {
        let encrypted: EncryptedData = bincode::deserialize(encrypted_dek)?;
        let dek_bytes = self.master_key.decrypt(&encrypted)?;
        
        if dek_bytes.len() != 32 {
            return Err(VaultError::Crypto("Invalid DEK length".to_string()));
        }
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&dek_bytes);
        
        Ok(DataEncryptionKey::from_bytes(key, algorithm))
    }
    
    fn key_id(&self) -> String {
        self.id.clone()
    }
}

#[allow(dead_code)]
pub fn envelope_encrypt<K: KeyEncryptionKey>(
    kek: &K,
    plaintext: &[u8],
    algorithm: EncryptionAlgorithm,
) -> Result<EnvelopeEncryption> {
    // Generate a new DEK
    let dek = DataEncryptionKey::generate(algorithm.clone());
    
    // Encrypt the data with the DEK
    let encrypted_data = dek.encrypt(plaintext)?;
    
    // Encrypt the DEK with the KEK
    let encrypted_dek = kek.encrypt_dek(&dek)?;
    
    Ok(EnvelopeEncryption {
        encrypted_dek,
        encrypted_data,
        kek_id: kek.key_id(),
        algorithm,
    })
}

#[allow(dead_code)]
pub fn envelope_decrypt<K: KeyEncryptionKey>(
    kek: &K,
    envelope: &EnvelopeEncryption,
) -> Result<Vec<u8>> {
    // Decrypt the DEK with the KEK
    let dek = kek.decrypt_dek(&envelope.encrypted_dek, envelope.algorithm.clone())?;
    
    // Decrypt the data with the DEK
    dek.decrypt(&envelope.encrypted_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_salt;
    
    #[test]
    fn test_envelope_encryption() {
        let salt = generate_salt();
        let master_key = MasterKey::derive_from_passphrase(
            "test-passphrase",
            &salt,
            EncryptionAlgorithm::Aes256Gcm,
        ).unwrap();
        
        let kek = LocalKeyEncryptionKey::new(master_key, "test-kek".to_string());
        let plaintext = b"secret data for envelope encryption";
        
        let envelope = envelope_encrypt(&kek, plaintext, EncryptionAlgorithm::Aes256Gcm).unwrap();
        let decrypted = envelope_decrypt(&kek, &envelope).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
        assert_eq!(envelope.kek_id, "test-kek");
    }
}