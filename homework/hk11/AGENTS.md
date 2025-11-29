# AGENTS.md

## Build/Lint/Test Commands
- `cargo build` / `cargo build --release` - Build debug/release
- `cargo run` - Run the project
- `cargo test` - Run all tests; `cargo test test_name` - Run single test
- `cargo clippy` - Lint; `cargo fmt` - Format code

## Code Style Guidelines
- **Imports**: Group by std → external crates → local modules; use `mod` then `use`
- **Naming**: `snake_case` (functions/variables), `PascalCase` (types), `SCREAMING_SNAKE_CASE` (constants)
- **Types**: Use `f64` for math/graphics calculations; explicit types for function signatures
- **Error Handling**: Use `expect("描述信息")` with descriptive messages; prefer `Result` for recoverable errors
- **Comments**: Chinese comments are acceptable for math/physics explanations
- **Formatting**: Run `cargo fmt`; follow rustfmt conventions
- **Structure**: Keep functions focused; use modules (`mod`) to organize related code
