# AGENTS.md

## Build/Lint/Test Commands

### Building
- `cargo build` - Build the project in debug mode
- `cargo build --release` - Build the project in release mode

### Running
- `cargo run` - Build and run the game

### Testing
- `cargo test` - Run all tests
- `cargo test test_name` - Run a specific test function (e.g., `cargo test test_collision_detection`)
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

### Comments and Documentation
- Use Chinese comments for module-level documentation (/// 模块说明)
- Use Chinese inline comments for complex logic
- Document all public structs, enums, and functions with /// doc comments

### Types
- Use explicit types for function parameters and return values
- Use `f32` for game coordinates, sizes, and speeds
- Prefer concrete types over generics when clarity is improved
- Use `Result<(), macroquad::Error>` for main function

### Error Handling
- Use `Result` for recoverable errors
- Use `unwrap()` sparingly - prefer `unwrap_or()`, `unwrap_or_else()`, or proper error handling
- Use `expect("descriptive message")` when the unwrap reason needs explanation
- Handle `Option` values with `match`, `if let`, or combinators

### Code Structure
- Keep functions small and focused
- Use modules to organize related functionality (entity, systems, config, etc.)
- Use meaningful variable names (can be in Chinese for comments)
- Add Chinese comments for complex game logic
- Follow Rust ownership and borrowing rules strictly
