# Development Context & Guidelines

## Project Structure
```
Vault/
├── .amazonq/rules/memory-bank/  # Memory bank files
└── test.code                   # Main Rust implementation
```

## Development Principles
- **Minimal Dependencies**: Only essential crates for core functionality
- **Error Recovery**: Graceful handling of file I/O and parsing errors
- **User Experience**: Clear prompts and feedback messages
- **Data Safety**: Change tracking and save confirmation before exit

## Security Considerations
- Passwords stored in plain text JSON (consider encryption for production)
- Clipboard operations may leave traces in system clipboard history
- File permissions should be restricted in production use

## Extension Points
- **Encryption**: Add password encryption before JSON serialization
- **Import/Export**: Support for other password manager formats
- **Categories**: Group passwords by category/tags
- **Password Generation**: Built-in secure password generator
- **Master Password**: Add authentication layer

## Testing Strategy
- Manual testing through CLI interaction
- File I/O edge cases (missing files, invalid JSON)
- Index boundary testing for remove/copy operations
- Clipboard functionality across different platforms

## Build & Dependencies
- Standard Rust project structure expected
- Cargo.toml should include the documented dependencies
- Cross-platform compatibility through standard library usage

## Code Style
- Snake_case for functions and variables
- PascalCase for structs
- Descriptive function names matching their purpose
- Minimal comments (self-documenting code preferred)