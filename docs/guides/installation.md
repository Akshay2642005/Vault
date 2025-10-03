# Installation Guide

This guide covers various methods to install Vault on different operating systems.

## Quick Install (Recommended)

### Linux and macOS

```bash
curl -sSL https://releases.vault.dev/install.sh | sh
```

### Windows (PowerShell)

```powershell
iwr -useb https://releases.vault.dev/install.ps1 | iex
```

## Package Managers

### Homebrew (macOS/Linux)

```bash
brew install vault-cli
```

### Chocolatey (Windows)

```powershell
choco install vault-cli
```

### Cargo (All platforms)

```bash
cargo install vault-cli
```

## Manual Installation

### Download Pre-built Binaries

1. Visit the [releases page](https://github.com/vault/vault/releases)
2. Download the appropriate binary for your platform:
   - `vault-linux-amd64` (Linux x86_64)
   - `vault-linux-amd64-musl` (Linux x86_64, static)
   - `vault-macos-amd64` (macOS Intel)
   - `vault-macos-arm64` (macOS Apple Silicon)
   - `vault-windows-amd64.exe` (Windows x86_64)

3. Make the binary executable (Linux/macOS):
   ```bash
   chmod +x vault
   ```

4. Move to a directory in your PATH:
   ```bash
   sudo mv vault /usr/local/bin/
   ```

### Build from Source

#### Prerequisites

- Rust 1.70 or later
- Git

#### Steps

```bash
# Clone the repository
git clone https://github.com/vault/vault.git
cd vault/app

# Build release binary
cargo build --release

# Install to system
cargo install --path .
```

## Verification

Verify the installation by running:

```bash
vault --version
```

You should see output similar to:
```
vault-cli 0.1.0
```

## Shell Completions

Generate shell completions for your shell:

```bash
# Bash
vault completions bash > ~/.bash_completion.d/vault

# Zsh
vault completions zsh > ~/.zsh/completions/_vault

# Fish
vault completions fish > ~/.config/fish/completions/vault.fish

# PowerShell
vault completions powershell > $PROFILE
```

## Next Steps

- [Getting Started Guide](getting-started.md)
- [Configuration](configuration.md)
- [Cloud Sync Setup](cloud-sync.md)

## Troubleshooting

### Common Issues

#### Permission Denied (Linux/macOS)
```bash
sudo chmod +x /usr/local/bin/vault
```

#### Command Not Found
Ensure the binary is in your PATH:
```bash
echo $PATH
which vault
```

#### Windows Defender/Antivirus
Some antivirus software may flag the binary. Add an exception for the vault executable.

### Getting Help

If you encounter issues:
1. Check the [troubleshooting guide](troubleshooting.md)
2. Search existing [GitHub issues](https://github.com/vault/vault/issues)
3. Create a new issue with details about your system and the error