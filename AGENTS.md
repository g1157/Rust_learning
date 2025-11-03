# AGENTS.md

## Build/Lint/Test Commands

### Building
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build the project in release mode

### Running
- `cargo run` - Build and run the project

### Testing
- `cargo test` - Run all tests
- `cargo test test_name` - Run a specific test function
- `cargo test -- --test test_name` - Run a specific integration test

### Linting and Formatting
- `cargo clippy` - Run the linter (clippy)
- `cargo clippy --fix` - Auto-fix clippy warnings where possible
- `cargo fmt` - Format code with rustfmt
- `cargo fmt --check` - Check if code is properly formatted

## Code Style Guidelines

### Imports
- Use `use` statements at the top of files
- Group imports: std library, external crates, then local modules
- Remove unused imports (cargo clippy will warn)

### Formatting
- Use rustfmt for consistent formatting
- Follow standard Rust formatting conventions

### Naming Conventions
- Functions and variables: `snake_case`
- Types, structs, enums: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Types
- Use explicit types for function parameters and return values
- Prefer concrete types over generics when clarity is improved
- Use `Result<T, E>` for functions that can fail

### Error Handling
- Use `Result` for recoverable errors
- Use `unwrap()` only in tests or when certain the value exists
- Use `expect("message")` with descriptive messages for debugging
- Handle `Option` values properly with `match`, `if let`, or `map`

### Code Structure
- Keep functions small and focused
- Use meaningful variable names
- Add comments for complex logic
- Follow Rust ownership and borrowing rules
