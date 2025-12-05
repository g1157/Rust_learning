# AGENTS.md

## Build/Lint/Test Commands
- `cargo build` / `cargo build --release` - Build debug/release
- `cargo run` / `cargo run -- <args>` - Run with optional arguments
- `cargo test` - Run all tests
- `cargo test test_name` - Run single test by name
- `cargo test --test integration_test` - Run specific integration test
- `cargo clippy` / `cargo clippy --fix` - Lint and auto-fix
- `cargo fmt` / `cargo fmt --check` - Format code

## Code Style Guidelines
- **Imports**: Group as std → external crates → local modules
- **Naming**: `snake_case` (functions/variables), `PascalCase` (types), `SCREAMING_SNAKE_CASE` (constants)
- **Types**: Use explicit types for function signatures; `f64` for physics/math; `Result<T, E>` for fallible ops
- **Errors**: Prefer `expect("msg")` or `unwrap_or_else()` over bare `unwrap()`; use `Result<(), Box<dyn Error>>` for main
- **Structure**: Keep functions focused; add comments for complex physics/math logic; follow ownership rules strictly
