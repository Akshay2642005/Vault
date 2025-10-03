# Vault Test Environment

This directory contains a complete test environment for the Vault password manager.

## Quick Start

1. **Build the vault CLI:**
   ```bash
   cargo build --release
   ```

2. **Initialize two test hosts:**
   ```bash
   cd test.code
   ./scripts/init.sh
   ```

3. **Run sync simulation:**
   ```bash
   ./scripts/simulate-sync.sh
   ```

## Directory Structure

- `dev-1/` - First test host environment
- `dev-2/` - Second test host environment  
- `scripts/` - Test automation scripts
- `tests/` - Integration tests
- `secrets/` - Sample secrets for testing

## Test Scenarios

The test environment demonstrates:

- Multi-host secret synchronization
- Encrypted storage and transport
- Tenant isolation
- Role-based access control
- Conflict resolution

## Security Notes

- All test data uses strong encryption (AES-256-GCM)
- Secrets are never stored in plaintext
- Each host has isolated storage
- Sync operations are authenticated and encrypted