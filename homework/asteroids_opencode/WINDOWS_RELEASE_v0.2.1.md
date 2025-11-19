# Windows Release v0.2.1 - Build Summary

## Build Information

**Build Date**: 2025-11-19  
**Target**: `x86_64-pc-windows-gnu`  
**Build Time**: 1.70 seconds  
**Executable Size**: 4.3 MB  
**Package Size**: 12 MB (uncompressed), 5.3 MB (compressed)

## What's New in v0.2.1

### 🎮 Independent Weapon Switching
- **Per-Player Control**: Each player can now independently switch weapons
- **Player 1**: Press **U** to cycle through weapons
- **Player 2**: Press **4** or **Numpad 4** to cycle through weapons
- Players can use different weapon types simultaneously for strategic gameplay

### 🔧 Bug Fixes from v0.2
- Screen shake effects now work properly during gameplay
- Fixed 6 compiler warnings related to unused screen shake variable
- Achievement reset now properly clears all statistics
- Settings UI font switching fully functional

### 🎯 Weapon Types
1. **Normal**: Standard single-shot bullets
2. **Spread**: Multiple bullets in a cone (wider coverage)
3. **Penetrating**: Powerful bullets that pierce through asteroids

## Package Contents

```
release_windows/
├── asteroids_opencode.exe (4.3 MB)
├── run.bat (startup script)
├── README.txt (Windows quick start guide)
├── README.md (full documentation - English)
├── README.zh-CN.md (full documentation - Chinese)
└── assets/
    ├── sounds/ (shoot.wav, powerup.wav)
    └── fonts/ (DejaVu, Ubuntu, Chinese fonts)
```

## Installation

1. Extract `asteroids_windows_v0.2.1.tar.gz` to a folder
2. Run `asteroids_opencode.exe` or `run.bat`
3. Enjoy!

## Controls Reference

### Player 1
- **W/A/D**: Thrust / Rotate Left / Rotate Right
- **J or F**: Shoot
- **U**: Switch weapon (when enabled)

### Player 2
- **↑/←/→**: Thrust / Rotate Left / Rotate Right
- **1 or Numpad 1**: Shoot
- **4 or Numpad 4**: Switch weapon (when enabled)

### General
- **Enter**: Start game / Confirm
- **Esc or P**: Pause
- **M**: Return to menu
- **F3**: Toggle debug panel

## System Requirements

- **OS**: Windows 10/11 (64-bit)
- **Graphics**: DirectX 11 compatible
- **Disk Space**: ~12 MB
- **RAM**: 100 MB+

## Technical Details

### Build Process
```bash
cargo build --release --target x86_64-pc-windows-gnu
```

### Dependencies
- Rust toolchain with mingw-w64
- Macroquad game engine
- Cross-compilation from Linux

### File Structure
- Executable: statically linked (no DLL dependencies)
- Assets: embedded in executable or loaded from `assets/` folder
- Configuration: saved to user's local app data

## Known Issues

- First launch may take a few seconds to initialize
- Some antivirus software may flag the executable (false positive)
- Screen shake intensity cannot be adjusted (fixed values)

## Changelog from v0.2

### Added
- Independent weapon switching per player
- `weapon_switch_pressed()` method in Controls
- Alternative weapon switch key support (Numpad 4)

### Changed
- Weapon switching from global (Q key) to per-player (U/4 keys)
- Updated all control documentation
- Improved code structure for weapon management

### Fixed
- Screen shake now properly connected to render pipeline
- Debug stats and time scale correctly passed to rendering
- All compiler warnings resolved

## Distribution

**Archive**: `asteroids_windows_v0.2.1.tar.gz` (5.3 MB)

**Recommended Distribution**:
1. GitHub Releases
2. itch.io
3. Direct download links

## Testing Checklist

- [ ] Game launches successfully
- [ ] Both players can control ships independently
- [ ] Weapon switching works for both players
- [ ] Settings menu functional (volume, lives, fonts, etc.)
- [ ] Screen shake triggers on collisions
- [ ] Audio playback works correctly
- [ ] Achievement system saves progress
- [ ] Both Survival and Duel modes playable

## Credits

Built with:
- **Rust** - Systems programming language
- **Macroquad** - Game engine
- **mingw-w64** - Windows cross-compilation toolchain

---

For bug reports or feedback, please check the project repository.
