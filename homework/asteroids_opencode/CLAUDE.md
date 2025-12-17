# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Development
cargo build                    # Debug build
cargo run                      # Run debug build
cargo check                    # Quick compile check (use this frequently)

# Production
cargo build --release          # Optimized build (opt-level='z', LTO enabled)
cargo run --release            # Run release build

# Quality
cargo test                     # Run unit tests (in-module tests)
cargo fmt                      # Format code (2024 edition)
cargo clippy -- -D warnings    # Strict linting

# Profiling (native only)
cargo run --release --features profiling  # Enable puffin profiler

# CLI testing flags
cargo run -- --frames 1000                   # Run N frames then exit
cargo run -- --dump-metrics metrics.json     # Export performance metrics
cargo run -- --entities 500                  # Stress test with N entities
cargo run -- --network-test                  # Enable network test mode
cargo run -- --headless                      # CI mode (no graphics)
```

### Server (in `server/` directory)

```bash
cd server
cargo build --release
cargo run --release
```

### Web/WASM Build

```bash
./build_web.sh    # Outputs to web/
```

## Architecture Overview

**Engine**: Macroquad 0.4.14 with custom entity-component pattern (not full ECS)
**Edition**: Rust 2024
**Targets**: Native (Linux/Windows/macOS) + WASM

### Core Game Loop (`main.rs`)

```
Input → Physics Update → QuadTree Collision → Response/Scoring → Effects → UI → Render
```

### Module Responsibilities

| Module | Purpose |
|--------|---------|
| `main.rs` | Game loop, state machine (`GameState`/`GameMode`), collision orchestration |
| `player.rs` | Player state, controls, dash, hyperspace, modifiers |
| `bullet.rs` | 5 weapon types: Normal, Spread, Penetrating, Homing, ChainIon |
| `network.rs` | WebSocket client (ewebsock), message protocol |
| `interpolation.rs` | `InterpBuffer<T>`, entity interpolation for network play |
| `battle_draft.rs` | Card selection system (12 cards, 4 categories, rarity tiers) |
| `constants.rs` | **All tuning values** - gameplay, timing, UI, particles |

### Key Patterns

- **Constants centralization**: All game tuning in `src/constants.rs` organized by subsystem
- **State enums**: `GameState` (Menu/Playing/Paused/GameOver) + `GameMode` (Survival/TimeAttack/Duel/Online)
- **QuadTree collision**: O(log n) spatial partitioning in `quadtree.rs`
- **Client prediction**: Local input prediction with server reconciliation (`reconcile()`, `reset_prediction_state()`)

### Network Architecture (Online Mode)

```
Client → Server: JoinQueue, GameInput, Ready, LeaveRoom, Ping
Server → Client: Queued, MatchFound, GameState, GameUpdate, GameEnd
```

- JSON serialization via serde
- 100ms render delay for interpolation
- Phases 1-3 complete (see `docs/PHASE_ROADMAP.md`)

## Key Constants Reference

Important tuning values in `src/constants.rs`:

```rust
// Gameplay
gameplay::INITIAL_ASTEROID_COUNT = 10
gameplay::ASTEROID_WAVE_INCREMENT = 2

// Phase Dash (相位闪现)
phase_dash::DISTANCE = 150.0     // Teleport distance
phase_dash::COOLDOWN = 3.0       // Seconds
phase_dash::EXPLOSION_RADIUS = 70.0

// Chain Ion (链式离子炮)
chain_ion::MAX_JUMPS = 3
chain_ion::RANGE = 260.0
chain_ion::DAMAGE_RATIOS = [1.0, 0.7, 0.5]

// Homing Missiles
homing::SPEED = 600.0
homing::TURN_RATE = 4.0
homing::TRACKING_RANGE = 500.0
```

## Documentation

- `docs/PHASE_ROADMAP.md` - Network sync phases 1-8 roadmap
- `docs/adr/` - Architecture Decision Records (ECS choice, network protocol)
- `docs/perf.md` - Performance monitoring guide

## Debug Features

- **F3**: Toggle debug panel (FPS, entity count, quadtree depth)
- `--dump-metrics`: Export JSON performance data
- `profiling` feature: Enable puffin HTTP server for flame graphs
