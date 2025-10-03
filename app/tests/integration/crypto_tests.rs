use vault_cli::crypto::{
    MasterKey, EncryptionAlgorithm, generate_salt, generate_nonce,
    derive_key_argon2id, derive_key_hkdf, KeyDerivationParams,
    envelope_encrypt, envelope_decrypt, LocalKeyEncryptionKey,
};

#[test]
fn test_salt_generation() {
    let salt1 = generate_salt();
    let salt2 = generate_salt();
    
    // Salts should be different
    assert_ne!(salt1, salt2);
    assert_eq!(salt1.len(), 32);
    assert_eq!(salt2.len(), 32);
}

#[test]
fn test_nonce_generation() {
    let nonce1 = generate_nonce(12);
    let nonce2 = generate_nonce(12);
    
    // Nonces should be different
    assert_ne!(nonce1, nonce2);
    assert_eq!(nonce1.len(), 12);
    assert_eq!(nonce2.len(), 12);
}

#[test]
fn test_key_derivation_argon2id() {
    let salt = generate_salt();
    let passphrase = "test-passphrase";
    
    let key1 = derive_key_argon2id(passphrase, &salt).unwrap();
    let key2 = derive_key_argon2id(passphrase, &salt).unwrap();
    
    // Same passphrase and salt should produce same key
    assert_eq!(key1, key2);
    assert_eq!(key1.len(), 32);
}

#[test]
fn test_key_derivation_different_salts() {
    let salt1 = generate_salt();
    let mut salt2 = salt1;
    salt2[0] = salt2[0].wrapping_add(1);
    let passphrase = "test-passphrase";
    
    let key1 = derive_key_argon2id(passphrase, &salt1).unwrap();
    let key2 = derive_key_argon2id(passphrase, &salt2).unwrap();
    
    // Different salts should produce different keys
    assert_ne!(key1, key2);
}

#[test]
fn test_hkdf_derivation() {
    let input_key = b"input key material";
    let salt = b"salt";
    let info = b"application info";
    
    let key1 = derive_key_hkdf(input_key, salt, info).unwrap();
    let key2 = derive_key_hkdf(input_key, salt, info).unwrap();
    
    // Same inputs should produce same output
    assert_eq!(key1, key2);
    assert_eq!(key1.len(), 32);
}

#[test]
fn test_aes256gcm_encryption() {
    let salt = generate_salt();
    let master_key = MasterKey::derive_from_passphrase(
        "test-passphrase",
        &salt,
        EncryptionAlgorithm::Aes256Gcm,
    ).unwrap();
    
    let plaintext = b"secret message";
    let encrypted = master_key.encrypt(plaintext).unwrap();
    let decrypted = master_key.decrypt(&encrypted).unwrap();
    
    assert_eq!(plaintext, decrypted.as_slice());
    assert_eq!(encrypted.algorithm, EncryptionAlgorithm::Aes256Gcm);
}

#[test]
fn test_chacha20poly1305_encryption() {
    let salt = generate_salt();
    let master_key = MasterKey::derive_from_passphrase(
        "test-passphrase",
        &salt,
        EncryptionAlgorithm::ChaCha20Poly1305,
    ).unwrap();
    
    let plaintext = b"secret message";
    let encrypted = master_key.encrypt(plaintext).unwrap();
    let decrypted = master_key.decrypt(&encrypted).unwrap();
    
    assert_eq!(plaintext, decrypted.as_slice());
    assert_eq!(encrypted.algorithm, EncryptionAlgorithm::ChaCha20Poly1305);
}

#[test]
fn test_encryption_different_outputs() {
    let salt = generate_salt();
    let master_key = MasterKey::derive_from_passphrase(
        "test-passphrase",
        &salt,
        EncryptionAlgorithm::Aes256Gcm,
    ).unwrap();
    
    let plaintext = b"secret message";
    let encrypted1 = master_key.encrypt(plaintext).unwrap();
    let encrypted2 = master_key.encrypt(plaintext).unwrap();
    
    // Different encryptions should produce different ciphertexts (due to random nonces)
    assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    assert_ne!(encrypted1.nonce, encrypted2.nonce);
    
    // But both should decrypt to the same plaintext
    let decrypted1 = master_key.decrypt(&encrypted1).unwrap();
    let decrypted2 = master_key.decrypt(&encrypted2).unwrap();
    
    assert_eq!(decrypted1, decrypted2);
    assert_eq!(plaintext, decrypted1.as_slice());
}

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
    assert_eq!(envelope.algorithm, EncryptionAlgorithm::Aes256Gcm);
}

#[test]
fn test_custom_key_derivation_params() {
    let salt = generate_salt();
    let passphrase = "test-passphrase";
    
    let params = KeyDerivationParams {
        memory_cost: 32768, // 32 MB
        time_cost: 2,
        parallelism: 1,
    };
    
    let key = vault_cli::crypto::derive_key_argon2id_with_params(passphrase, &salt, &params).unwrap();
    assert_eq!(key.len(), 32);
}

#[test]
fn test_master_key_generation() {
    let key1 = MasterKey::generate(EncryptionAlgorithm::Aes256Gcm);
    let key2 = MasterKey::generate(EncryptionAlgorithm::Aes256Gcm);
    
    let plaintext = b"test data";
    
    // Both keys should work for encryption/decryption
    let encrypted1 = key1.encrypt(plaintext).unwrap();
    let decrypted1 = key1.decrypt(&encrypted1).unwrap();
    assert_eq!(plaintext, decrypted1.as_slice());
    
    let encrypted2 = key2.encrypt(plaintext).unwrap();
    let decrypted2 = key2.decrypt(&encrypted2).unwrap();
    assert_eq!(plaintext, decrypted2.as_slice());
    
    // But they should produce different results (different keys)
    assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
}

#[test]
fn test_wrong_key_decryption_fails() {
    let salt1 = generate_salt();
    let salt2 = generate_salt();
    
    let key1 = MasterKey::derive_from_passphrase(
        "passphrase1",
        &salt1,
        EncryptionAlgorithm::Aes256Gcm,
    ).unwrap();
    
    let key2 = MasterKey::derive_from_passphrase(
        "passphrase2",
        &salt2,
        EncryptionAlgorithm::Aes256Gcm,
    ).unwrap();
    
    let plaintext = b"secret message";
    let encrypted = key1.encrypt(plaintext).unwrap();
    
    // Decryption with wrong key should fail
    let result = key2.decrypt(&encrypted);
    assert!(result.is_err());
}