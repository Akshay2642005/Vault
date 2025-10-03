use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, aead::{Aead, KeyInit}};
use rand::{rngs::OsRng, RngCore};

use crate::error::{VaultError, Result};

pub fn encrypt_chacha20poly1305(key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let key = Key::from_slice(key);
    let cipher = ChaCha20Poly1305::new(key);
    
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| VaultError::Crypto(format!("ChaCha20-Poly1305 encryption failed: {}", e)))?;
    
    Ok((ciphertext, nonce_bytes.to_vec()))
}

pub fn decrypt_chacha20poly1305(key: &[u8; 32], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
    let key = Key::from_slice(key);
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| VaultError::Crypto(format!("ChaCha20-Poly1305 decryption failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chacha20poly1305_roundtrip() {
        let key = [0u8; 32];
        let plaintext = b"Hello, World!";
        
        let (ciphertext, nonce) = encrypt_chacha20poly1305(&key, plaintext).unwrap();
        let decrypted = decrypt_chacha20poly1305(&key, &ciphertext, &nonce).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
    
    #[test]
    fn test_chacha20poly1305_different_nonces() {
        let key = [0u8; 32];
        let plaintext = b"Hello, World!";
        
        let (ciphertext1, nonce1) = encrypt_chacha20poly1305(&key, plaintext).unwrap();
        let (ciphertext2, nonce2) = encrypt_chacha20poly1305(&key, plaintext).unwrap();
        
        // Different nonces should produce different ciphertexts
        assert_ne!(nonce1, nonce2);
        assert_ne!(ciphertext1, ciphertext2);
        
        // But both should decrypt to the same plaintext
        let decrypted1 = decrypt_chacha20poly1305(&key, &ciphertext1, &nonce1).unwrap();
        let decrypted2 = decrypt_chacha20poly1305(&key, &ciphertext2, &nonce2).unwrap();
        
        assert_eq!(decrypted1, decrypted2);
        assert_eq!(plaintext, decrypted1.as_slice());
    }
}