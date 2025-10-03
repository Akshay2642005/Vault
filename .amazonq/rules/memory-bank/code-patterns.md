# Code Patterns & Implementation Details

## Error Handling Pattern
```rust
match fs::read_to_string(file) {
    Ok(content) => match serde_json::from_str(&content) {
        Ok(data) => data,
        Err(_) => Vec::<Password>::new(),
    },
    Err(_) => Vec::<Password>::new(),
}
```
- Nested match statements for graceful error recovery
- Default to empty Vec when file doesn't exist or is invalid

## Input Validation Pattern
```rust
fn get_usize() -> usize {
    let input = input!("prompt: ");
    match input.parse::<usize>() {
        Ok(num) => num,
        Err(_) => {
            println!("invalid input");
            get_usize()  // Recursive retry
        },
    }
}
```
- Recursive validation with user feedback
- Used for index inputs in remove/copy operations

## State Management
- Passwords vector passed by value/reference as needed
- `mut` keyword used selectively for operations that modify data
- Index regeneration after each operation via `index()` function
- Change tracking with counter for unsaved modifications

## Display Logic
- Uses `tabled` crate for formatted table output
- Conditional column removal for password hiding
- Modern table styling with colored headers
- Separate display function for reusability

## Clipboard Integration
- Wrapped in `Clippy` struct for error handling
- Provides user feedback on copy success/failure
- Handles clipboard initialization errors gracefully

## Command Processing
- Simple string matching in main loop
- Each command maps to dedicated function
- Immutable cloning for read operations (search)
- Mutable operations return modified vector