# 🧪 Vault Test Commands

## Basic Functionality Tests

### 1. Help and Version
```bash
# Show help
vault --help

# Show version
vault --version

# Show command-specific help
vault init --help
vault put --help
vault users --help
vault audit --help
```

### 2. Configuration Setup
```bash
# Configure cloud mode (interactive)
vault sync configure

# Check current status
vault status

# Run diagnostics
vault doctor
```

### 3. Tenant Management
```bash
# Initialize new tenant
vault init --tenant test-company --admin admin@test.com

# Login to tenant
vault login --tenant test-company

# Check current user
vault whoami

# Logout
vault logout
```

### 4. Secret Management
```bash
# Store simple secret
vault put test-secret --namespace development

# Store secret with tags
vault put api-key --namespace production --tags api,external,critical

# List secrets
vault list --namespace development
vault list --namespace production --detailed

# Get secret
vault get test-secret --namespace development
vault get api-key --namespace production --metadata

# Search secrets
vault search "api" --namespace production

# Delete secret
vault delete test-secret --namespace development --force
```

### 5. Enhanced Secret Types (Interactive)
```bash
# Create different secret types (will prompt for type selection)
vault put database-creds --namespace production
# Choose: Database Credentials -> postgres

vault put ssh-keys --namespace infrastructure  
# Choose: SSH Key Pair

vault put secure-password --namespace development
# Choose: Simple Password -> Generate -> 32 chars -> Include symbols

vault put custom-api-key --namespace integration
# Choose: API Key -> Custom prefix: "myapp"
```

### 6. Password-Protected Secrets
```bash
# Create password-protected secret
vault put sensitive-data --namespace production
# Interactive: Choose secret type -> Enable password protection -> Set access password

# Retrieve password-protected secret
vault get sensitive-data --namespace production
# Will prompt for access password
```

## Collaborative Mode Tests

### 7. User Management (Requires collaborative mode)
```bash
# Configure collaborative mode first
vault sync configure
# Choose: Collaborative mode -> Backend -> Configure

# Invite users
vault users invite --email developer@company.com --role writer
vault users invite --email manager@company.com --role owner
vault users invite --email auditor@company.com --role auditor

# List users
vault users list

# Change user role
vault users change-role --email developer@company.com --role reader

# Remove user
vault users remove --email developer@company.com
```

### 8. User Invitation Acceptance
```bash
# Accept invitation (as new user)
vault users accept --token <invitation-token-received>
# Interactive: Enter email -> Create password

# Login as new user
vault login --tenant company --email newuser@company.com
```

### 9. Role-Based Access Testing
```bash
# Login as different roles and test permissions

# As Admin/Owner
vault users list                    # Should work
vault audit tail                    # Should work
vault put secret --namespace prod   # Should work

# As Writer  
vault put secret --namespace prod   # Should work
vault get secret --namespace prod   # Should work
vault users list                    # Should fail

# As Reader
vault get secret --namespace prod   # Should work
vault put secret --namespace prod   # Should fail
vault users list                    # Should fail

# As Auditor
vault get secret --namespace prod   # Should work
vault audit tail                    # Should work
vault put secret --namespace prod   # Should fail
```

## Audit System Tests

### 10. Audit Operations (Admin/Auditor only)
```bash
# View recent audit logs
vault audit tail --lines 50

# Follow audit logs in real-time
vault audit tail --follow

# Search audit logs
vault audit search "secret_created"
vault audit search "login" --since "2024-01-01"
vault audit search "user_added" --since "2024-01-01" --until "2024-12-31"
```

## Cloud Sync Tests

### 11. Sync Operations
```bash
# Check sync status
vault sync status

# Push secrets to cloud
vault sync push

# Pull secrets from cloud
vault sync pull

# Force operations
vault sync push --force
vault sync pull --force
```

## Import/Export Tests

### 12. Data Management
```bash
# Export secrets
vault export --output secrets-backup.json --namespace production
vault export --output all-secrets.json

# Import secrets
vault import secrets-backup.json --namespace staging
vault import external-secrets.json --namespace integration
```

## Advanced Tests

### 13. Multi-Namespace Operations
```bash
# Create secrets in different namespaces
vault put prod-db --namespace production
vault put dev-db --namespace development  
vault put test-api --namespace testing

# List all namespaces
vault list --namespace production
vault list --namespace development
vault list --namespace testing

# Search across namespaces
vault search "db"
vault search "api" --namespace testing
```

### 14. Tag-Based Operations
```bash
# Create secrets with tags
vault put secret1 --namespace prod --tags database,critical
vault put secret2 --namespace prod --tags api,external
vault put secret3 --namespace prod --tags database,internal

# List by tags
vault list --namespace prod --tag database
vault list --namespace prod --tag critical
```

### 15. Session Management
```bash
# Login with remember option
vault login --tenant company --remember

# Check session info
vault whoami

# Test session expiration (wait or modify session file)
vault whoami  # Should show expired session

# Re-login
vault login --tenant company
```

## Error Handling Tests

### 16. Permission Tests
```bash
# Try operations without login
vault put test  # Should fail with "Please login first"

# Try admin operations as non-admin
vault users list  # Should fail if not admin

# Try audit operations as non-auditor
vault audit tail  # Should fail if not admin/auditor
```

### 17. Invalid Input Tests
```bash
# Invalid tenant
vault login --tenant nonexistent

# Invalid namespace/key
vault get nonexistent --namespace nonexistent

# Invalid role
vault users invite --email test@test.com --role invalid-role

# Invalid commands
vault invalid-command
vault put  # Missing required arguments
```

## Performance Tests

### 18. Bulk Operations
```bash
# Create multiple secrets (script this)
for i in {1..10}; do
    vault put "secret-$i" --namespace bulk-test
done

# List all
vault list --namespace bulk-test

# Search performance
vault search "secret" --namespace bulk-test

# Bulk export
vault export --output bulk-secrets.json --namespace bulk-test
```

## Configuration Tests

### 19. Different Cloud Modes
```bash
# Test none mode
vault sync configure  # Choose: None
vault sync status      # Should show "None (fully local)"

# Test backup mode  
vault sync configure  # Choose: Backup -> S3
vault sync status      # Should show "Backup"

# Test collaborative mode
vault sync configure  # Choose: Collaborative -> Postgres
vault sync status      # Should show "Collaborative"
```

### 20. Configuration Validation
```bash
# Test with custom config file
vault --config custom-config.toml status

# Test with invalid config
vault --config invalid-config.toml status
```

## Expected Behaviors

### Success Cases
- All commands should provide clear success messages
- Interactive prompts should be user-friendly
- Error messages should be helpful and actionable
- Audit logs should capture all operations
- Role-based access should be enforced consistently

### Security Validations
- Passwords should never be displayed in plain text
- Session timeouts should be enforced
- Access passwords for secrets should be required
- Audit logs should be admin/auditor only
- User management should be admin only

### Performance Expectations
- Commands should complete within reasonable time
- Large secret lists should be handled efficiently
- Search operations should be fast
- Import/export should handle reasonable file sizes

This comprehensive test suite validates all implemented features and ensures the vault operates correctly in all supported modes.