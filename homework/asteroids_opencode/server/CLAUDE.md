# Server Module

[Root](../CLAUDE.md) > **server**

## Module Purpose

Tokio-based WebSocket game server for Asteroids Online multiplayer mode. Handles room management, player matching, game state synchronization, and authoritative physics simulation.

## Entry Point

`src/main.rs` - Single-file server implementation

## Build and Run

```bash
cd server
cargo build --release
cargo run --release
```

Server listens on `0.0.0.0:9001` by default.

## Architecture

### Connection Flow

```
Client Connect -> Peer Created -> JoinQueue -> Room Assignment -> Ready -> GameStart -> Game Loop
```

### Key Components

| Component | Purpose |
|-----------|---------|
| `Peer` | Individual client connection with tx channel |
| `Room` | Game room with players, mode, and game state |
| `GameState` | Server-authoritative game simulation state |
| `PeerMap` | `Arc<RwLock<HashMap<Uuid, Peer>>>` |
| `RoomMap` | `Arc<RwLock<HashMap<Uuid, Room>>>` |

### Message Protocol

**Client -> Server:**
- `JoinQueue { mode, nickname }` - Join matchmaking
- `LeaveQueue` - Leave matchmaking
- `GameInput { keys, seq }` - Player input with sequence number
- `Ready` - Signal ready to start
- `LeaveRoom` - Exit current room
- `Ping` - Heartbeat

**Server -> Client:**
- `Connected { player_id }` - Connection confirmed
- `MatchFound { room_id, players, mode }` - Match ready
- `GameStart` - Game beginning
- `GameState { players, asteroids, bullets, vortices, powerups, last_input_seqs, timestamp }` - State sync
- `PlayerDisconnected { player_id }` - Player left
- `GameOver { winner, scores }` - Game ended
- `Pong` - Heartbeat response

### Game Loop

Runs at 30 FPS (configurable `TICK_RATE`):
1. Update player physics based on input
2. Update asteroids, bullets, vortices
3. Collision detection (bullets-asteroids, ships-asteroids, ships-bullets in Duel)
4. Spawn new vortices and powerups
5. Check game over conditions
6. Broadcast `GameState` to all room players

### Game Constants

Defined in `game_constants` module to match client:
- `SCREEN_WIDTH/HEIGHT`: 1024x768
- `SHIP_ACCEL`: 200.0
- `MAX_SPEED`: 300.0
- `BULLET_SPEED`: 500.0
- `SHOOT_COOLDOWN`: 0.15s

### Supported Game Modes

| Mode | End Condition | Players |
|------|--------------|---------|
| `Survival` | All players dead | 2 |
| `Duel` | One player remaining | 2 |

## Dependencies

- `tokio` - Async runtime
- `tokio-tungstenite` - WebSocket support
- `serde` / `serde_json` - JSON serialization
- `uuid` - Player/room IDs
- `rand` - Asteroid spawning

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | All server logic (single file) |
| `Cargo.toml` | Dependencies |
| `start.sh` | Launch script |
| `server.log` | Runtime logs |

## Testing

Currently manual testing via WebSocket clients. Planned:
- Integration tests with mock clients
- Load testing tools (Phase 8)

## Notes

- Input timeout protection: clears input after 0.3s without updates
- Room cleanup: Empty rooms removed every 30 seconds
- Client prediction support: `last_input_seq` tracking per player
