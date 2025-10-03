use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_vault_init_and_basic_operations() {
    let temp_dir = TempDir::new().unwrap();
    let vault_path = temp_dir.path().join("vault.db");
    
    // Test vault init
    let output = Command::new("cargo")
        .args(&["run", "--", "init", "--tenant", "test-tenant", "--admin", "test@example.com"])
        .env("VAULT_STORAGE_PATH", vault_path.to_str().unwrap())
        .output()
        .expect("Failed to execute vault init");
        
    assert!(output.status.success(), "Vault init failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Test putting a secret
    let output = Command::new("cargo")
        .args(&["run", "--", "put", "test-key", "--namespace", "test", "--value", "test-value"])
        .env("VAULT_STORAGE_PATH", vault_path.to_str().unwrap())
        .output()
        .expect("Failed to execute vault put");
        
    assert!(output.status.success(), "Vault put failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Test getting the secret
    let output = Command::new("cargo")
        .args(&["run", "--", "get", "test-key", "--namespace", "test"])
        .env("VAULT_STORAGE_PATH", vault_path.to_str().unwrap())
        .output()
        .expect("Failed to execute vault get");
        
    assert!(output.status.success(), "Vault get failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-value"), "Secret value not retrieved correctly");
}