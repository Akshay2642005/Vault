# Vault Documentation

Welcome to the Vault documentation. This directory contains comprehensive guides, API references, and security documentation for the Vault password manager.

## Quick Links

- [Installation Guide](guides/installation.md)
- [Getting Started](guides/getting-started.md)
- [CLI Reference](api/cli-reference.md)
- [Security Architecture](security/architecture.md)
- [Cloud Sync Setup](guides/cloud-sync.md)

## Documentation Structure

### Guides (`guides/`)
Step-by-step tutorials and how-to guides for common tasks:
- Installation and setup
- Basic usage
- Cloud synchronization
- Multi-tenant configuration
- Backup and recovery

### API Reference (`api/`)
Detailed reference documentation:
- CLI command reference
- Configuration options
- Library API (for developers)

### Security (`security/`)
Security-focused documentation:
- Cryptographic architecture
- Threat model
- Security best practices
- Audit and compliance

## Contributing to Documentation

Documentation is written in Markdown and follows these conventions:
- Use clear, concise language
- Include code examples where appropriate
- Keep sections focused and well-organized
- Update the table of contents when adding new sections

## Building Documentation

The documentation can be built using various tools:

```bash
# Using mdBook (recommended)
mdbook build

# Using GitBook
gitbook build

# Using Sphinx (for API docs)
sphinx-build -b html docs/ docs/_build/
```

## Feedback

If you find any issues with the documentation or have suggestions for improvement, please:
1. Open an issue on GitHub
2. Submit a pull request with improvements
3. Contact us at docs@vault.dev