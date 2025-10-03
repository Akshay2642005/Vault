use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_vault_help() {
    let mut cmd = Command::cargo_bin("vault").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A local-first, multi-tenant password manager"));
}

#[test]
fn test_vault_version() {
    let mut cmd = Command::cargo_bin("vault").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("vault-cli"));
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let mut cmd = Command::cargo_bin("vault").unwrap();
    cmd.env("VAULT_STORAGE_PATH", db_path.to_str().unwrap())
        .args(&["init", "--tenant", "test-tenant", "--admin", "admin@test.com"]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Vault initialized"));
}

#[test]
fn test_init_and_basic_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Initialize vault
    let mut cmd = Command::cargo_bin("vault").unwrap();
    cmd.env("VAULT_STORAGE_PATH", db_path.to_str().unwrap())
        .env("VAULT_CONFIG_DIR", config_dir.to_str().unwrap())
        .args(&["init", "--tenant", "test-tenant", "--admin", "admin@test.com"]);
    cmd.assert().success();
    
    // Test status command
    let mut cmd = Command::cargo_bin("vault").unwrap();
    cmd.env("VAULT_STORAGE_PATH", db_path.to_str().unwrap())
        .env("VAULT_CONFIG_DIR", config_dir.to_str().unwrap())
        .arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Vault Status"));
}

#[test]
fn test_put_get_secret() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Initialize vault
    let mut cmd = Command::cargo_bin("vault").unwrap();
    cmd.env("VAULT_STORAGE_PATH", db_path.to_str().unwrap())
        .env("VAULT_CONFIG_DIR", config_dir.to_str().unwrap())
        .args(&["init", "--tenant", "test-tenant", "--admin", "admin@test.com"]);
    cmd.assert().success();
    
    // Note: These tests would require implementing non-interactive mode
    // or mocking the password input for full integration testing
}

#[test]
fn test_list_empty_namespace() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Initialize vault
    let mut cmd = Command::cargo_bin("vault").unwrap();
    cmd.env("VAULT_STORAGE_PATH", db_path.to_str().unwrap())
        .env("VAULT_CONFIG_DIR", config_dir.to_str().unwrap())
        .args(&["init", "--tenant", "test-tenant", "--admin", "admin@test.com"]);
    cmd.assert().success();
    
    // List secrets in empty namespace
    let mut cmd = Command::cargo_bin("vault").unwrap();
    cmd.env("VAULT_STORAGE_PATH", db_path.to_str().unwrap())
        .env("VAULT_CONFIG_DIR", config_dir.to_str().unwrap())
        .args(&["list", "--namespace", "empty"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No secrets found"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("vault").unwrap();
    cmd.arg("invalid-command");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_doctor_command() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Initialize vault
    let mut cmd = Command::cargo_bin("vault").unwrap();
    cmd.env("VAULT_STORAGE_PATH", db_path.to_str().unwrap())
        .env("VAULT_CONFIG_DIR", config_dir.to_str().unwrap())
        .args(&["init", "--tenant", "test-tenant", "--admin", "admin@test.com"]);
    cmd.assert().success();
    
    // Run doctor
    let mut cmd = Command::cargo_bin("vault").unwrap();
    cmd.env("VAULT_STORAGE_PATH", db_path.to_str().unwrap())
        .env("VAULT_CONFIG_DIR", config_dir.to_str().unwrap())
        .arg("doctor");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Running diagnostics"));
}