# GEMINI.md - Asteroids Game

## Project Overview

This project is a modern remake of the classic arcade game "Asteroids," written in Rust using the `macroquad` game engine. It features both single-player and multiplayer modes, with a client-server architecture for online play. The game can be compiled to run as a native desktop application or as a WebAssembly (WASM) module for web browsers.

**Key Technologies:**

*   **Game Engine:** `macroquad`
*   **Language:** Rust
*   **Web Target:** WebAssembly (WASM)
*   **Server:** Custom WebSocket server using `tokio` and `tokio-tungstenite`
*   **Serialization:** `serde` and `serde_json`

**Architecture:**

*   **Client:** The core game logic is in the `src` directory. It handles rendering, player input, game state, and different game modes (Survival, Duel). It can be compiled to a native executable or a WASM module.
*   **Server:** The `server` directory contains a separate Rust project for the WebSocket server. It manages player connections, game rooms, and real-time synchronization for the online multiplayer mode.

## Building and Running

### Native (Desktop)

To build and run the native desktop version of the game:

```bash
# Build the game
cargo build

# Run the game
cargo run
```

### Web (WebAssembly)

To build the WebAssembly version and run it locally:

```bash
# Build the web version (compiles to WASM, copies files)
./build_web.sh

# Start a local web server
./serve.sh
# Alternatively:
# cd web && python3 -m http.server 8000
```

Then, open your web browser and go to `http://localhost:8000`.

### Server (for Online Multiplayer)

To run the WebSocket server for online multiplayer:

```bash
# Navigate to the server directory
cd server

# Run the server
cargo run
```

The server will start listening for connections on `0.0.0.0:9001`.

## Development Conventions

The project is structured into several modules, each responsible for a specific aspect of the game:

*   `achievement.rs`: Manages achievements and player statistics.
*   `asteroid.rs`: Defines asteroid behavior and spawning.
*   `bullet.rs`: Handles bullet logic.
*   `duel.rs`: Contains the logic for the "Duel" game mode.
*   `font.rs`: Manages font loading and rendering.
*   `network.rs`: Client-side networking for online multiplayer.
*   `player.rs`: Defines the player ship and controls.
*   `server/src/main.rs`: The server for online multiplayer.
*   `ui.rs`: Handles the user interface, including menus and HUD.
*   `main.rs`: The main entry point of the game, containing the main game loop.

The code is well-commented (in Chinese) and follows standard Rust conventions.
