# Asteroids

A modern take on the classic Asteroids arcade game, built with Rust and Macroquad. Features multiple game modes including cooperative survival, competitive duel, and time attack challenges. Supports both single-player and local multiplayer with smooth gameplay, particle effects, and audio.

## Features

### Game Modes
- **Survival Mode**: Clear waves of asteroids solo or with a friend
  - Wave-by-wave progression with victory pause screens
  - Progressive difficulty (more asteroids each wave)
  - Vortex hazards that pull ships toward the center
- **TimeAttack Mode**: Race against the clock to clear waves as fast as possible
  - Timer tracks your completion speed
  - Optimized for speedrun competition
- **Duel Mode**: Competitive flag-capture battles (in development)
- **Online Mode**: WebSocket-based multiplayer (work in progress)

### Core Gameplay
- **Single-Player & Local Multiplayer**: Play solo or with two players on one keyboard
- **Dash Ability**: Quick evasive maneuver with temporary invincibility
  - 2.0s cooldown, 0.35s duration, 3.5x speed boost
  - 0.4s invulnerability window
  - Visual trail effect during dash
  - Cooldown indicator on HUD
- **4 Weapon Types**: Normal, Spread, Penetrating, Homing missiles
- **Power-ups**: Shield pickups for temporary invincibility
- **Smooth Physics**: Realistic momentum-based ship controls

### Visual Effects
- Explosive asteroid destruction with colored particles
- Thruster flames when accelerating
- Collision impact effects
- Dash trails with fade effect
- Parallax starfield background
- Screen shake on impacts
- Slow motion on high killstreaks

### Audio System
- Sound effects with adjustable volume (default 1%)
- Professional audio fadeout processing (250ms/150ms)
- No audio artifacts or popping sounds
- See `AUDIO_FADEOUT.md` for technical details

### Progression
- **38 Achievements** with persistent storage
- High score tracking
- Statistics tracking (asteroids destroyed, games played, etc.)

### Polished UI
- Clean gradients, shadows, and visual feedback
- Multiple font options including Chinese support
- Debug panel (F3) showing FPS, entity count, quadtree depth

## Controls

### Player 1
- **W/A/D**: Thrust / Rotate Left / Rotate Right
- **J or F**: Shoot
- **Space**: Dash (quick evasive maneuver with invincibility)
- **U**: Switch weapon (when enabled in settings)

### Player 2
- **Arrow Keys**: Thrust / Rotate Left / Rotate Right
- **1 or Numpad 1**: Shoot
- **Numpad 0**: Dash
- **4 or Numpad 4**: Switch weapon (when enabled in settings)

### General
- **Enter**: Start game / Confirm selection
- **Esc or P**: Pause game
- **M**: Return to mode selection
- **F3**: Toggle debug panel (FPS, entity count, quadtree depth)
- **Arrow Keys or A/D** (in menus): Navigate / Adjust settings

## Building and Running

### Prerequisites
- Rust 1.70+ (2024 edition)
- Cargo

### Quick Start

```bash
# Build and run in debug mode
cargo run

# Build and run in release mode (better performance)
cargo run --release

# Just build without running
cargo build --release
```

### Development Commands

```bash
# Quick compile check
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Lint with clippy
cargo clippy -- -D warnings
```

## Game Settings

Access the settings menu from the main menu to customize your experience:

- **Starting Lives**: 1-9 lives per round (default: 3)
- **Ship Speed**: 0.5x - 2.0x multiplier for thrust and rotation (default: 1.0x)
- **Ship Size**: 0.5x - 2.0x multiplier for ship rendering and collision (default: 1.0x)
  - Adjusts visual size, collision detection, shield radius, bullet spawn position
  - Useful for accessibility and player preference
- **Asteroid Speed**: 0.5x - 2.0x multiplier (default: 1.0x)
- **Sound Volume**: 0% - 100% (default: 1%)
  - Adjust with ←/→ or A/D keys in 1% increments
  - Volume uses relative multiplier (0.0 - 1.0)
- **UI Font**: Choose between Default, Chinese (WQY Micro Hei), or system fonts
  - Full settings screen font support with real-time preview
- **Weapon Switch**: Enable/disable Q key weapon switching
- **Screen Shake**: Toggle screen shake effects
- **Slow Motion**: Toggle slow motion on high killstreaks
- **Debug Panel**: Toggle F3 debug overlay by default
- **Flag Radius** (Duel mode): 50-150px capture radius (default: 90px)
- **Reset to Defaults**: Restore all settings to default values
- **Reset Achievements**: Clear all achievement progress and statistics

**New in v0.2:**
- ✅ Settings screen fully supports font switching
- ✅ Reset operations show success notifications
- ✅ Achievement reset properly clears all statistics

## Audio System

### Sound Effects
- **shoot.wav**: Shooting sound (250ms fadeout, 66% ratio)
- **powerup.wav**: Power-up collection sound (150ms fadeout, 7.5% ratio)
- All sounds processed with professional fadeout to eliminate audio artifacts

### Volume Control
- Default volume: **1%** (0.01 relative multiplier)
- Adjust in settings menu with ←/→ or A/D keys
- Range: 0% - 100% in 1% increments

### Audio Processing
All sound files use linear fadeout to prevent "popping" or "clicking" sounds:
- Short sounds (like shooting) use longer fadeout ratios
- See `AUDIO_FADEOUT.md` and `SHOOT_FADEOUT_FIX.md` for technical details
- Original audio backed up in `assets/sounds/original_backup/`

## Game Modes

### Survival Mode
Work together (or solo) to eliminate all asteroids across increasingly difficult waves. Each destroyed asteroid splits into smaller fragments. Collect shield power-ups to protect against collisions.

**Wave System:**
- Clear all asteroids to trigger a 2-second victory pause
- Next wave spawns with 2 additional asteroids
- Vortex hazards appear and pull ships toward center
- Game ends when all players are eliminated

**Scoring:**
- Large asteroids: 20 points
- Medium asteroids: 50 points  
- Small asteroids: 100 points

### TimeAttack Mode
Race against the clock to clear asteroid waves as fast as possible. Perfect for speedrunners and competitive players.

**Features:**
- Real-time timer tracking your performance
- Same wave progression as Survival
- Timer resets properly on restart
- Compare your best times

### Duel Mode (In Development)
Capture the flag before your opponent. First player to reach the target score wins. Features respawning flags and strategic bullet-based combat.

### Online Mode (Work in Progress)
WebSocket-based multiplayer allowing players to compete over the internet. Server code included in `server/` directory.

## Technical Details

- **Engine**: [Macroquad](https://github.com/not-fl3/macroquad) 0.4
- **Language**: Rust (2024 edition)
- **Architecture**: Entity-component pattern with clean module separation
- **Physics**: Time-based delta updates at ~60 FPS
- **Collision Detection**: QuadTree spatial partitioning for O(log n) performance
- **Particle System**: Custom implementation with up to 1000 concurrent particles
- **Audio**: Macroquad audio with custom fadeout processing (250ms/150ms)
- **Volume System**: Adjustable volume with 1% default and 1% increment steps
- **Slow Motion**: Time-scale based slow motion (0.4x-0.6x) on high killstreaks
- **Screen Shake**: Dynamic intensity based on event magnitude

## Project Structure

```
src/
├── main.rs           # Game loop, state management, input handling
├── player.rs         # Player state, controls, dash mechanics, killstreaks
├── ship.rs           # Ship physics and movement
├── asteroid.rs       # Asteroid spawning and splitting logic
├── bullet.rs         # Projectile mechanics with 4 weapon types
├── powerup.rs        # Shield power-up system
├── vortex.rs         # Vortex hazard system
├── particle.rs       # Particle effects system
├── effects.rs        # Screen shake, slow motion effects
├── background.rs     # Parallax starfield background
├── render.rs         # Scene rendering, dash trails, visual effects
├── ui.rs             # UI components and HUD
├── ui_achievements.rs # Achievement display UI
├── achievement.rs    # 38 achievements with persistence
├── score.rs          # Score tracking
├── duel.rs           # Flag capture game mode
├── network.rs        # WebSocket multiplayer client
├── sound.rs          # Audio system with volume control
├── font.rs           # Font loading with Chinese support
├── input.rs          # Input handling abstraction
├── wasm_input.rs     # Web/WASM input handling
├── storage.rs        # Save/load game data
├── quadtree.rs       # Spatial partitioning for collision
├── constants.rs      # Game constants
└── utils.rs          # Collision detection and helpers

server/               # Online multiplayer server (Rust)
web/                  # Web build files and assets

assets/
└── sounds/              # Audio files
    ├── shoot.wav        # (250ms fadeout)
    ├── powerup.wav      # (150ms fadeout)
    └── original_backup/ # Original unprocessed audio
```

## Contributing

This is a homework/learning project, but feedback and suggestions are welcome! Please ensure all code passes `cargo fmt` and `cargo clippy -- -D warnings` before submitting.

## License

Educational project - feel free to learn from and adapt the code.

## Roadmap

### Completed
- [x] Victory pause between waves
- [x] Particle effects system
- [x] Sound effects support
- [x] Audio fadeout processing
- [x] QuadTree collision optimization
- [x] Adjustable game settings
- [x] Volume control system
- [x] Killstreak and slow motion
- [x] Font system with Chinese support
- [x] Achievement system (38 achievements)
- [x] Settings screen font support
- [x] Reset success notifications
- [x] Single-player mode
- [x] TimeAttack mode
- [x] Dash ability with invincibility
- [x] Vortex hazards
- [x] Parallax starfield background

### In Progress
- [ ] Complete Duel mode features
- [ ] Online multiplayer (WebSocket server ready)

### Future Ideas
- [ ] Background music
- [ ] UFO enemies (classic Asteroids style)
- [ ] More power-up types (magnet, score multiplier, speed boost)
- [ ] Challenge/mission system with specific objectives
- [ ] Different asteroid types with unique behaviors
- [ ] Persistent upgrades between runs
- [ ] Leaderboards
- [ ] Custom arena designs
- [ ] Gamepad support
- [ ] More particle effects (shield breaks, power-up trails)

## Documentation

- **README.md** - This file (overview and quick start)
- **FONT_AND_UI_IMPROVEMENTS.md** - Font system and UI improvements (v0.2)
- **ACHIEVEMENT_FIX.md** - Achievement system complete fix
- **AUDIO_FADEOUT.md** - Audio fadeout processing guide
- **SHOOT_FADEOUT_FIX.md** - Shooting sound fix technical details
- **AGENTS.md** - Development guidelines and code style
- **assets/sounds/README.md** - Audio asset information
