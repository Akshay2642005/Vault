# Vault Development Guide

## Project Structure

```
vault/
├── app/                    # Rust CLI application
│   ├── src/
│   │   ├── auth/          # Authentication and session management
│   │   ├── cli/           # Command-line interface
│   │   ├── config/        # Configuration management
│   │   ├── crypto/        # Encryption and key derivation
│   │   ├── storage/       # Local storage with Sled
│   │   ├── sync/          # Cloud synchronization
│   │   ├── error.rs       # Error types
│   │   ├── lib.rs         # Library exports
│   │   └── main.rs        # CLI entry point
│   ├── tests/             # Integration tests
│   └── Cargo.toml         # Rust dependencies
├── docs/                  # Documentation
├── test.code/             # Test scenarios and examples
├── website/               # Marketing website (React)
├── install.sh             # Unix installation script
├── install.ps1            # Windows installation script
└── config.example.toml    # Example configuration
```

## Development Setup

### Prerequisites

1. **Rust** (latest stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Git**
   ```bash
   # Ubuntu/Debian
   sudo apt install git
   
   # macOS
   brew install git
   
   # Windows
   # Download from https://git-scm.com/
   ```

### Building

```bash
# Clone the repository
git clone https://github.com/vault/vault.git
cd vault/app

# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- --help
```

### Development Workflow

1. **Make changes** to the source code
2. **Test locally**:
   ```bash
   cargo test
   cargo run -- init --tenant test --admin test@example.com
   cargo run -- login --tenant test
   cargo run -- put test-secret
   cargo run -- get test-secret
   ```
3. **Run integration tests**:
   ```bash
   cd ../test.code
   ./scripts/run-tests.sh
   ```
4. **Format and lint**:
   ```bash
   cargo fmt
   cargo clippy
   ```

## Architecture

### Storage Layer

- **Primary**: Sled embedded database for local-first storage
- **Encryption**: AES-256-GCM or ChaCha20-Poly1305
- **Key Derivation**: Argon2id with configurable parameters
- **Structure**: Hierarchical with tenants → namespaces → secrets

### Sync Layer

- **Backends**: S3, PostgreSQL, SQLite
- **Conflict Resolution**: Vector clocks with manual resolution UI
- **Security**: Zero-knowledge (server only sees encrypted data)

### CLI Layer

- **Framework**: Clap v4 with derive macros
- **UX**: Progress bars, colored output, interactive prompts
- **Completions**: Shell completions for bash, zsh, fish, PowerShell

## Adding New Features

### Adding a New Command

1. **Define the command** in `src/cli/mod.rs`:
   ```rust
   #[derive(Subcommand)]
   pub enum Commands {
       // ... existing commands
       NewCommand {
           #[arg(help = "Description")]
           param: String,
       },
   }
   ```

2. **Handle the command** in the match statement:
   ```rust
   Commands::NewCommand { param } => {
       new_command(&param).await
   }
   ```

3. **Implement the handler** in `src/cli/commands.rs`:
   ```rust
   pub async fn new_command(param: &str) -> Result<()> {
       // Implementation here
       Ok(())
   }
   ```

### Adding a New Storage Backend

1. **Create the module** in `src/sync/`:
   ```rust
   // src/sync/new_backend.rs
   use crate::{storage::VaultStorage, sync::{SyncResult, SyncMetadata}, error::Result};
   
   pub async fn new_backend_push(storage: &VaultStorage, config: &str, force: bool) -> Result<SyncResult> {
       // Implementation
   }
   ```

2. **Add to sync manager** in `src/sync/mod.rs`:
   ```rust
   #[derive(Debug, Serialize, Deserialize)]
   pub enum SyncBackend {
       // ... existing backends
       NewBackend { config: String },
   }
   ```

3. **Handle in sync operations**:
   ```rust
   match &self.backend {
       // ... existing cases
       SyncBackend::NewBackend { config } => {
           new_backend_push(&self.storage, config, force).await
       }
   }
   ```

## Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_encryption

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```bash
cd test.code
./scripts/run-tests.sh
```

### Manual Testing

```bash
# Initialize test vault
cargo run -- init --tenant test-org --admin admin@test.com

# Test basic operations
cargo run -- login --tenant test-org
cargo run -- put github-token --namespace development
cargo run -- get github-token --namespace development
cargo run -- list --namespace development
cargo run -- delete github-token --namespace development --force

# Test export/import
cargo run -- export --output secrets.json --namespace development
cargo run -- import secrets.json --namespace development

# Test sync (requires configuration)
cargo run -- sync status
cargo run -- sync push
cargo run -- sync pull
```

## Security Considerations

### Encryption

- **Never store plaintext secrets** in memory longer than necessary
- **Use zeroization** for sensitive data structures
- **Implement constant-time comparisons** for authentication
- **Use secure random number generation** for salts and nonces

### Key Management

- **Master keys derived from passphrases** using Argon2id
- **Envelope encryption** for cloud KMS integration
- **Key rotation** capabilities (planned)

### Audit Trail

- **Log all secret operations** with timestamps and user context
- **Tamper-evident logging** (planned)
- **Export capabilities** for SIEM integration

## Performance

### Benchmarks

```bash
# Run benchmarks
cargo bench

# Profile with perf (Linux)
cargo build --release
perf record target/release/vault put test-secret
perf report
```

### Optimization Tips

- **Batch operations** when possible
- **Use async/await** for I/O operations
- **Minimize memory allocations** in hot paths
- **Consider compression** for large secret values

## Debugging

### Logging

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- command

# Enable trace logging for specific module
RUST_LOG=vault_cli::storage=trace cargo run -- command

# Log to file
RUST_LOG=debug cargo run -- command 2> debug.log
```

### Common Issues

1. **Database corruption**:
   ```bash
   # Backup and recreate
   cp ~/.vault/vault.db ~/.vault/vault.db.backup
   rm ~/.vault/vault.db
   # Re-initialize
   ```

2. **Permission errors**:
   ```bash
   # Check file permissions
   ls -la ~/.vault/
   chmod 600 ~/.vault/vault.db
   ```

3. **Sync conflicts**:
   ```bash
   # Check sync status
   cargo run -- sync status
   # Force resolution
   cargo run -- sync pull --force
   ```

## Contributing

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/new-feature`
3. **Make changes** and add tests
4. **Run the test suite**: `cargo test`
5. **Format code**: `cargo fmt`
6. **Submit a pull request**

### Code Style

- Follow **Rust standard formatting** (`cargo fmt`)
- Use **meaningful variable names**
- Add **documentation comments** for public APIs
- Include **unit tests** for new functionality
- Keep **functions focused** and small

### Commit Messages

```
feat: add SQLite sync backend
fix: resolve memory leak in encryption module
docs: update installation instructions
test: add integration tests for export functionality
```

## Release Process

1. **Update version** in `Cargo.toml`
2. **Update CHANGELOG.md**
3. **Create release tag**: `git tag v0.2.0`
4. **Push tag**: `git push origin v0.2.0`
5. **GitHub Actions** will build and publish releases

## Roadmap

### v0.2.0 (Current)
- ✅ Basic CLI functionality
- ✅ Local storage with encryption
- ✅ Multi-tenant support
- ✅ Export/import capabilities
- 🔄 Cloud sync (S3, PostgreSQL, SQLite)
- 🔄 Audit logging

### v0.3.0 (Planned)
- 🔄 Hardware security key support
- 🔄 Browser extension
- 🔄 Mobile apps
- 🔄 GUI application (Tauri)

### v1.0.0 (Future)
- 🔄 Enterprise features
- 🔄 LDAP/SSO integration
- 🔄 Advanced audit capabilities
- 🔄 Plugin system