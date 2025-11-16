# Repository Guidelines

## Project Structure & Module Organization
The workspace contains `Cargo.toml`, a single executable crate under `src/main.rs`, and rendered diagnostics (e.g., `divergence_compare.html`) generated during analysis. Keep simulation inputs close to `src/` and commit only lightweight artifacts; large intermediate data belongs in `target/` or a scratch directory ignored by git. Organize new Rust modules under `src/` with `mod.rs` or inline modules, and co-locate integration data inside `tests/` if you add black-box suites.

## Build, Test, and Development Commands
Use `cargo build` for fast debug builds and `cargo build --release` when benchmarking Lyapunov sweeps. `cargo run -- <args>` executes the Hyperion simulator with optional scenario flags. Run `cargo test` before every PR; narrow runs with `cargo test module::case` while iterating. Style checks rely on `cargo fmt` and `cargo clippy`; run `cargo clippy -- -D warnings` to ensure CI parity.

## Coding Style & Naming Conventions
Follow standard rustfmt output (4-space indentation, trailing commas on multi-line lists). Group imports as std, external crates, then local modules; remove unused `use` items. Favor descriptive snake_case identifiers (physical quantities may keep domain symbols like `theta` or `lambda_est`). Types and enums stay in PascalCase, constants in SCREAMING_SNAKE_CASE. Document nontrivial physics steps with brief comments explaining the formula or assumption.

## Testing Guidelines
Unit tests live beside their modules via `#[cfg(test)] mod tests`. Prefer deterministic seeds when checking chaotic divergence routines. Name tests after the behavior under scrutiny, e.g., `lyapunov_resets_delta`. When comparing floating-point outputs, assert within tolerances instead of exact equality. Capture any HTML regression expectations via snapshot text (not binary) to keep diffs reviewable.

## Commit & Pull Request Guidelines
Commits in this repo are short, present-tense summaries (often bilingual), e.g., "改进代码风格：移除不必要的引用". Keep each commit focused: code, docs, or assets, but not all at once. PRs should describe the simulation scenario, highlight physics assumptions, link to issues or notebook cells, and include screenshots or paths to updated `.html` plots when visuals change. Mention how you validated the change (commands run, data sets) so reviewers can reproduce results quickly.
