use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use hkdf::Hkdf;
use sha2::Sha256;

use crate::error::{VaultError, Result};

pub struct KeyDerivationParams {
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl Default for KeyDerivationParams {
    fn default() -> Self {
        Self {
            memory_cost: 65536, // 64 MB
            time_cost: 3,
            parallelism: 1,
        }
    }
}

pub fn derive_key_argon2id(passphrase: &str, salt: &[u8; 32]) -> Result<[u8; 32]> {
    derive_key_argon2id_with_params(passphrase, salt, &KeyDerivationParams::default())
}

pub fn derive_key_argon2id_with_params(
    passphrase: &str,
    salt: &[u8; 32],
    params: &KeyDerivationParams,
) -> Result<[u8; 32]> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(
            params.memory_cost,
            params.time_cost,
            params.parallelism,
            Some(32),
        ).map_err(|e| VaultError::Crypto(format!("Invalid Argon2 parameters: {}", e)))?,
    );
    
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| VaultError::Crypto(format!("Salt encoding error: {}", e)))?;
    
    let hash = argon2.hash_password(passphrase.as_bytes(), &salt_string)
        .map_err(|e| VaultError::Crypto(format!("Key derivation error: {}", e)))?;
    
    let hash_bytes = hash.hash
        .ok_or_else(|| VaultError::Crypto("No hash output".to_string()))?;
    
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes.as_bytes()[..32]);
    
    Ok(key)
}

pub fn derive_key_hkdf(input_key: &[u8], salt: &[u8], info: &[u8]) -> Result<[u8; 32]> {
    let hk = Hkdf::<Sha256>::new(Some(salt), input_key);
    let mut output = [0u8; 32];
    
    hk.expand(info, &mut output)
        .map_err(|e| VaultError::Crypto(format!("HKDF expansion failed: {}", e)))?;
    
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_argon2id_deterministic() {
        let passphrase = "test-passphrase";
        let salt = [0u8; 32];
        
        let key1 = derive_key_argon2id(passphrase, &salt).unwrap();
        let key2 = derive_key_argon2id(passphrase, &salt).unwrap();
        
        assert_eq!(key1, key2);
    }
    
    #[test]
    fn test_argon2id_different_salts() {
        let passphrase = "test-passphrase";
        let salt1 = [0u8; 32];
        let mut salt2 = [0u8; 32];
        salt2[0] = 1;
        
        let key1 = derive_key_argon2id(passphrase, &salt1).unwrap();
        let key2 = derive_key_argon2id(passphrase, &salt2).unwrap();
        
        assert_ne!(key1, key2);
    }
    
    #[test]
    fn test_hkdf_deterministic() {
        let input_key = b"input key material";
        let salt = b"salt";
        let info = b"application info";
        
        let key1 = derive_key_hkdf(input_key, salt, info).unwrap();
        let key2 = derive_key_hkdf(input_key, salt, info).unwrap();
        
        assert_eq!(key1, key2);
    }
}