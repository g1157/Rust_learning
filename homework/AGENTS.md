# AGENTS.md

## Build/Lint/Test Commands

### Building
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build the project in release mode

### Running
- `cargo run` - Build and run the project
- `cargo run -- <args>` - Run with command-line arguments (e.g., `cargo run -- delta`)

### Testing
- `cargo test` - Run all tests
- `cargo test test_name` - Run a specific test function
- `cargo test --test test_name` - Run a specific integration test

### Linting and Formatting
- `cargo clippy` - Run the linter (clippy)
- `cargo clippy --fix` - Auto-fix clippy warnings where possible
- `cargo fmt` - Format code with rustfmt
- `cargo fmt --check` - Check if code is properly formatted

## Code Style Guidelines

### Imports
- Use `use` statements at the top of files
- Group imports: std library, external crates, then local modules
- Order: `use std::...;` then `mod module_name;` then `use module_name::...;`

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
- Use `f64` for floating-point calculations (physics simulations)
- Prefer concrete types over generics when clarity is improved
- Use `Result<T, E>` for functions that can fail

### Error Handling
- Use `Result<(), Box<dyn Error>>` for main functions that can fail
- Use `unwrap()` sparingly - prefer `unwrap_or_else()`, `expect()`, or proper error handling
- Use `expect("descriptive message")` when the unwrap reason needs explanation
- Handle `Option` values with `match`, `if let`, or combinators

### Code Structure
- Keep functions small and focused
- Use modules to organize related functionality
- Use meaningful variable names (can be in Chinese for physics variables)
- Add comments for complex physics/math logic
- Follow Rust ownership and borrowing rules strictly
