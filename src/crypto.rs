use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use serde::{Deserialize, Serialize};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroize;
use secrecy::{Secret, ExposeSecret};

pub struct MasterKey(Secret<[u8; 32]>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub salt: [u8; 32],
}

impl MasterKey {
    pub fn derive_from_passphrase(passphrase: &str, salt: &[u8; 32]) -> Result<Self, Box<dyn std::error::Error>> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt).map_err(|e| format!("Salt encoding error: {}", e))?;
        
        let hash = argon2.hash_password(passphrase.as_bytes(), &salt_string)
            .map_err(|e| format!("Key derivation error: {}", e))?;
            
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash.hash.unwrap().as_bytes()[..32]);
        
        Ok(Self(Secret::new(key)))
    }
    
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self(Secret::new(key))
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData, Box<dyn std::error::Error>> {
        let key = Key::<Aes256Gcm>::from_slice(self.0.expose_secret());
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| format!("Encryption error: {}", e))?;
            
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        
        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes,
            salt,
        })
    }
    
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let key = Key::<Aes256Gcm>::from_slice(self.0.expose_secret());
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| format!("Decryption error: {}", e).into())
    }
}

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}