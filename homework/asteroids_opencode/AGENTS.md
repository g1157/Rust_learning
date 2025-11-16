# Repository Guidelines

## Project Structure & Module Organization
Source lives under `src/`, with gameplay pieces split into focused modules such as `ship.rs`, `asteroid.rs`, `bullet.rs`, and helpers in `utils.rs`. Keep the async entry point lean in `src/main.rs` by delegating systems into their own files. Future integration tests belong in `tests/`, while reusable sprites or sounds should go under `assets/`. `Cargo.toml` declares the Macroquad target—tune shared constants or optional features there. Treat `target/` as build output only and never commit it.

## Build, Test, and Development Commands
Use `cargo check` for quick compiler feedback, `cargo build` (or `--release` when profiling) to produce binaries, and `cargo run` to launch the Macroquad window (`cargo run -- <args>` to pass gameplay tweaks). Run `cargo test` to execute the Rust test suite, `cargo test test_name` to run a single test function, `cargo test --test test_name` for a specific integration test. Run `cargo fmt` to enforce formatting, and `cargo clippy -- -D warnings` before opening a PR to keep lint debt at zero.

## Coding Style & Naming Conventions
Stick to idiomatic Rust: 4-space indentation, trailing commas where rustfmt inserts them, and modules/functions in `snake_case`. Types and enums use `PascalCase`; constants like `SHIP_HEIGHT` stay in `SCREAMING_SNAKE_CASE`. Order imports as `std`, third-party crates (`macroquad::prelude::*`), then local modules, removing anything unused. Keep structs (`Ship`, `Asteroid`, `Score`) as data holders and move logic into helper functions or modules for clarity. Use `f64` for time calculations and `f32` for physics/rendering. Chinese variable names are acceptable for physics constants (e.g., `像素/秒`).

## Testing Guidelines
Use Rust's built-in test framework; unit tests can sit beside the module they cover, while integration tests go under `tests/`. Name tests after behavior (`wrap_around_resets_position`) and keep them deterministic—inject RNG seeds when simulating physics. Ensure collision, wraparound, and win/loss paths have coverage, and document any `#[ignore]` cases that require a window or user interaction.

## Commit & Pull Request Guidelines
Follow the existing history: short, imperative commit titles such as `Add wraparound check` or concise Chinese equivalents. Each commit should build and pass `cargo fmt` plus `cargo clippy -- -D warnings`. Pull requests need a short summary, reproduction steps, and screenshots/GIFs for gameplay-facing changes. Reference related issues, list follow-up tasks explicitly, and mention any testing gaps so reviewers can focus on gameplay behavior instead of plumbing.
