# Vault Project Memory Bank

## Project Overview
- **Name**: rpass (Rust Password Manager)
- **Type**: CLI password manager written in Rust
- **Main File**: `test.code` (contains the complete implementation)
- **Default Storage**: `passwords.json`

## Core Functionality
- Store, retrieve, and manage passwords with associated service information
- Search passwords by service name
- Copy passwords to clipboard
- Hide/show passwords in display
- JSON-based persistent storage

## Data Structure
```rust
struct Password {
    service: String,    // Service name (e.g., "gmail", "github")
    email: String,      // Associated email
    username: String,   // Username for the service
    password: String,   // The actual password
    index: usize,       // Auto-generated index for operations
}
```

## Key Dependencies
- `tabled`: Table formatting and display
- `serde`: JSON serialization/deserialization
- `prompted`: User input handling
- `arboard`: Clipboard operations

## CLI Commands
- `new`: Create new password entry
- `list`: Display all passwords (with optional hiding)
- `search`: Find passwords by service name
- `copy`: Copy password to clipboard by index
- `remove`: Delete password by index
- `save`: Save changes to file
- `quit`: Exit with save prompt
- `forcequit`: Exit without saving
- `clear`: Clear terminal screen

## Command Line Arguments
- `rpass <file.json>`: Use specific JSON file
- `rpass --hidden`: Hide passwords in list view
- `rpass -h, --help`: Show help message

## Architecture Patterns
- Functional approach with immutable data transformations
- Error handling with Result types and match statements
- Modular functions for each operation
- Clipboard abstraction through Clippy struct
- Auto-indexing system for password management

## File Operations
- Loads from JSON file on startup
- Auto-creates file if doesn't exist
- Pretty-printed JSON output
- Tracks unsaved changes to prevent data loss