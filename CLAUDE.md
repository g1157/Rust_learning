# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## High-Level Architecture

This repository is a collection of individual Rust projects, likely created for learning purposes. The projects are organized into the following top-level directories:

- `homework/`: Contains various homework assignments.
- `test/`: Includes test projects and examples.
- `yufa/`: Holds projects related to Rust syntax and features.

Each subdirectory within these top-level folders is a self-contained Rust project managed by Cargo.

## Common Development Tasks

The following commands should be run from within the directory of the specific project you are working on (e.g., `cd homework/guessing_game`).

### Building a Project

To build any project in this repository, use the standard Cargo build command:

```bash
cargo build
```

### Running a Project

To build and run a project, use:

```bash
cargo run
```

### Running Tests

To run the tests for a project, use:

```bash
cargo test
```

### Managing Dependencies

Dependencies for each project are managed in its `Cargo.toml` file. To add a dependency, add it to the `[dependencies]` section of the `Cargo.toml` file. For example, the `guessing_game` project uses the `rand` crate, which is included in `homework/guessing_game/Cargo.toml`.

### Example Workflow: Running the Guessing Game

1.  Navigate to the project directory:
    ```bash
    cd homework/guessing_game
    ```
2.  Build and run the project:
    ```bash
    cargo run
    ```
