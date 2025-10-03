# 🔐 Vault

A local-first, multi-tenant password manager with cloud synchronization capabilities. Built with Rust for maximum security and performance.

## Features

- **🔒 Zero-Knowledge Encryption**: AES-256-GCM and ChaCha20-Poly1305 with Argon2id key derivation
- **🏠 Local-First**: Works completely offline, cloud sync is optional
- **🏢 Multi-Tenant**: Organizations, projects, and role-based access control
- **☁️ Cloud Sync**: Optional encrypted sync via S3, Postgres, or other backends
- **🛡️ Security**: Memory-safe Rust implementation with automatic secret zeroization
- **🎨 Beautiful CLI**: Intuitive commands with progress indicators and colored output

## Quick Start

### Installation

```bash
# Install via script (recommended)
curl -sSL https://releases.vault.dev/install.sh | sh

# Or build from source
git clone https://github.com/vault/vault.git
cd vault
cargo build --release
```

### Basic Usage

```bash
# Initialize vault for your organization
vault init --tenant acme-corp --admin alice@acme.com

# Login to your tenant
vault login --tenant acme-corp

# Store a secret
vault put github-token --namespace development
# Enter secret value: [hidden input]

# Retrieve a secret
vault get github-token --namespace development

# List all secrets in a namespace
vault list --namespace development

# Sync with cloud (optional)
vault sync push
```

## Architecture

### Local Storage
- **Database**: Sled (embedded key-value store)
- **Encryption**: Client-side AES-256-GCM encryption
- **Key Derivation**: Argon2id with configurable parameters

### Cloud Sync (Optional)
- **Backends**: S3, Postgres, or custom implementations
- **Security**: Zero-knowledge - server only sees encrypted data
- **Conflict Resolution**: Vector clocks with merge UI

### Multi-Tenancy
- **Tenants**: Top-level organizations
- **Namespaces**: Project-level secret scoping
- **Roles**: Admin, Owner, Writer, Reader, Auditor
- **Sessions**: JWT-based authentication with expiration

## Security

### Encryption
- **Symmetric**: AES-256-GCM (primary), ChaCha20-Poly1305 (alternative)
- **Key Derivation**: Argon2id with high memory cost (configurable)
- **Envelope Encryption**: Optional integration with AWS KMS, GCP KMS, Azure KeyVault
- **Memory Safety**: Automatic zeroization of secrets in memory

### Threat Model
- ✅ Protects against data breaches (encrypted at rest)
- ✅ Protects against network interception (encrypted in transit)
- ✅ Protects against server compromise (zero-knowledge)
- ✅ Protects against memory dumps (zeroization)
- ⚠️ Does not protect against compromised client devices
- ⚠️ Does not protect against weak master passwords

## Development

### Project Structure
```
vault/
├── src/           # Rust CLI source code
├── test.code/     # Integration tests and examples
├── website/       # React marketing website
└── docs/          # Documentation
```

### Testing
```bash
# Run unit tests
cargo test

# Run integration tests
cd test.code
./scripts/run-tests.sh

# Test multi-host sync
./scripts/simulate-sync.sh
```

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Cross-platform builds
cargo install cross
cross build --target x86_64-pc-windows-gnu
cross build --target x86_64-apple-darwin
```

## Configuration

Create `~/.config/vault/config.toml`:

```toml
storage_path = "~/.vault/vault.db"
tenant_id = "my-org"

[cloud_sync]
backend = "S3"
region = "us-east-1"
bucket = "my-vault-bucket"

# Optional KMS integration
# kms_key_id = "arn:aws:kms:us-east-1:123456789012:key/..."
```

## Commands

### Core Operations
- `vault init` - Initialize new vault
- `vault login` - Authenticate to tenant
- `vault put <key>` - Store secret
- `vault get <key>` - Retrieve secret
- `vault list` - List secrets
- `vault delete <key>` - Delete secret

### Sync Operations
- `vault sync push` - Upload encrypted secrets to cloud
- `vault sync pull` - Download and merge secrets from cloud
- `vault sync status` - Show sync status

### Management
- `vault roles add` - Add user to tenant
- `vault audit tail` - View audit logs
- `vault export` - Export encrypted backup
- `vault import` - Import from backup

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## Security Reporting

Report security vulnerabilities to security@vault.dev (PGP key available).

## Roadmap

- [ ] Hardware security key support (YubiKey, WebAuthn)
- [ ] Browser extension for autofill
- [ ] Mobile apps (iOS/Android)
- [ ] Audit log streaming to SIEM systems
- [ ] Plugin system for custom backends
- [ ] GUI application (Tauri-based)