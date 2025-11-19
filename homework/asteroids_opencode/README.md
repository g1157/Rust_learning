# Asteroids 🚀

A modern take on the classic Asteroids arcade game, built with Rust and Macroquad. Features cooperative survival mode and competitive duel mode with smooth gameplay, particle effects, and audio.

## Features

- **Survival Mode**: Team up with a friend to clear waves of asteroids and compete for the highest score
  - Wave-by-wave progression with victory pause screens
  - Progressive difficulty (more asteroids each wave)
- **Duel Mode**: Face off in competitive flag-capture battles (in development)
- **Local Multiplayer**: Two players on one keyboard
- **Power-ups**: Shield pickups for temporary invincibility
- **Smooth Physics**: Realistic momentum-based ship controls
- **Particle Effects**: 
  - Explosive asteroid destruction with colored particles
  - Thruster flames when accelerating
  - Collision impact effects
- **Audio System**: 
  - Sound effects with adjustable volume (default 1%)
  - Professional audio fadeout processing (250ms/150ms)
  - No audio artifacts or popping sounds
  - See `AUDIO_FADEOUT.md` for technical details
- **Polished UI**: Clean gradients, shadows, and visual feedback

## Controls

### Player 1
- **W/A/D**: Thrust / Rotate Left / Rotate Right
- **J or F**: Shoot
- **U**: Switch weapon (when enabled in settings)

### Player 2
- **↑/←/→**: Thrust / Rotate Left / Rotate Right
- **1 or Numpad 1**: Shoot
- **4 or Numpad 4**: Switch weapon (when enabled in settings)

### General
- **Enter**: Start game / Confirm selection
- **Esc or P**: Pause game
- **M**: Return to mode selection
- **F3**: Toggle debug panel (FPS, entity count, quadtree depth)
- **←/→ or A/D** (in settings): Adjust volume and other settings

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
Work together to eliminate all asteroids across increasingly difficult waves. Each destroyed asteroid splits into smaller fragments. Collect shield power-ups to protect against collisions. 

**Wave System:**
- Clear all asteroids to trigger a 2-second victory pause
- Next wave spawns with 2 additional asteroids
- Game ends when all players are eliminated

**Scoring:**
- Large asteroids: 20 points
- Medium asteroids: 50 points  
- Small asteroids: 100 points

**Visual Effects:**
- Particle explosions colored by the destroying player
- Thruster particle trails
- Shield power-up glowing effects

### Duel Mode (In Development)
Capture the flag before your opponent. First player to reach the target score wins. Features respawning flags and strategic bullet-based combat.

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
├── main.rs       # Game loop, state management, and settings
├── ship.rs       # Player ship physics
├── asteroid.rs   # Asteroid spawning and splitting logic
├── bullet.rs     # Projectile mechanics with weapon types
├── player.rs     # Player state, controls, and killstreaks
├── powerup.rs    # Shield power-up system
├── particle.rs   # Particle effects system
├── sound.rs      # Audio system with volume control
├── duel.rs       # Flag capture game mode
├── score.rs      # Score tracking
├── quadtree.rs   # Spatial partitioning for collision detection
├── font.rs       # Font loading system
├── ui.rs         # All rendering and UI components
└── utils.rs      # Collision detection and helpers

assets/
└── sounds/              # Audio files
    ├── shoot.wav        # (250ms fadeout)
    ├── powerup.wav      # (150ms fadeout)
    └── original_backup/ # Original unprocessed audio

Scripts:
├── fix_shoot_fadeout.py  # Audio fadeout processor (250ms/150ms)
├── add_fadeout.py        # Generic fadeout processor (50ms)

Documentation:
├── README.md                      # This file
├── FONT_AND_UI_IMPROVEMENTS.md    # Font system and UI improvements (v0.2)
├── ACHIEVEMENT_FIX.md             # Achievement system fixes
├── AUDIO_FADEOUT.md               # Audio fadeout guide
├── SHOOT_FADEOUT_FIX.md           # Shooting sound fix details
└── AGENTS.md                      # Development guidelines
```

## Contributing

This is a homework/learning project, but feedback and suggestions are welcome! Please ensure all code passes `cargo fmt` and `cargo clippy -- -D warnings` before submitting.

## License

Educational project - feel free to learn from and adapt the code.

## Roadmap

- [x] ~~Victory pause between waves~~
- [x] ~~Particle effects system~~
- [x] ~~Sound effects support~~
- [x] ~~Audio fadeout processing~~
- [x] ~~QuadTree collision optimization~~
- [x] ~~Adjustable game settings~~
- [x] ~~Volume control system~~
- [x] ~~Killstreak and slow motion~~
- [x] ~~Font system with Chinese support~~
- [x] ~~Achievement system (38 achievements)~~
- [x] ~~Settings screen font support~~
- [x] ~~Reset success notifications~~
- [ ] Complete Duel mode features
- [ ] Background music
- [ ] Implement online multiplayer
- [ ] Custom arena designs
- [ ] Gamepad support
- [ ] More particle types (shield breaks, power-up trails)

## Documentation

- **README.md** - This file (overview and quick start)
- **FONT_AND_UI_IMPROVEMENTS.md** - Font system and UI improvements (v0.2)
- **ACHIEVEMENT_FIX.md** - Achievement system complete fix
- **AUDIO_FADEOUT.md** - Audio fadeout processing guide
- **SHOOT_FADEOUT_FIX.md** - Shooting sound fix technical details
- **AGENTS.md** - Development guidelines and code style
- **assets/sounds/README.md** - Audio asset information
