# 音频系统修复指南

## 🎯 问题诊断

### 问题原因
Macroquad 的 `audio` 特性未在 `Cargo.toml` 中启用。

### 解决方案
已修复！在 `Cargo.toml` 中添加：

```toml
[dependencies]
macroquad = { version = "0.4.14", features = ["audio"] }
```

## ⚠️ ALSA 音频设备问题

如果看到以下错误：

```
ALSA lib pcm.c:2721:(snd_pcm_open_noupdate) Unknown PCM default
thread panicked at Can't open PCM device.
```

**原因**：系统没有音频设备或 ALSA 配置问题

### 解决方案A：使用虚拟音频设备（推荐）

创建 `~/.asoundrc` 文件：

```bash
cat > ~/.asoundrc << 'EOF'
pcm.!default {
    type plug
    slave.pcm "null"
}
EOF
```

这会创建一个虚拟音频设备，允许游戏运行但不输出声音。

### 解决方案B：禁用音频（临时）

如果只是想测试游戏，可以暂时移除音频特性：

```toml
# Cargo.toml
[dependencies]
macroquad = "0.4.14"  # 移除 features = ["audio"]
```

但这样会导致编译错误，因为代码中使用了音频 API。

### 解决方案C：条件编译音频（推荐）

修改代码，使音频系统可选：

1. 在 `Cargo.toml` 添加 feature flag：
```toml
[features]
default = ["audio"]
audio = ["macroquad/audio"]

[dependencies]
macroquad = "0.4.14"
```

2. 在代码中使用条件编译：
```rust
#[cfg(feature = "audio")]
use macroquad::audio::*;
```

### 解决方案D：安装音频驱动（本地开发）

如果在本地 Linux 系统：

```bash
# Ubuntu/Debian
sudo apt-get install alsa-utils pulseaudio

# 启动 PulseAudio
pulseaudio --start

# 测试音频
speaker-test -t wav -c 2
```

## 🎮 测试音频

### 方法1：在有音频设备的系统上运行

```bash
cargo run
# 进入游戏
# 按 Space 射击 - 应该听到音效
```

### 方法2：使用虚拟设备（静音模式）

```bash
# 创建 ~/.asoundrc (见上文)
cargo run
# 游戏正常运行，但无声音输出
```

### 方法3：检查音频是否真的播放

添加调试输出（已实现）：

游戏启动时会显示：
```
正在加载音效文件...
音效加载状态:
  射击 (shoot.wav):      ✓
  爆炸 (explosion.wav):  ✗
  推进 (thrust.wav):     ✗
  道具 (powerup.wav):    ✓
  碰撞 (hit.wav):        ✗
✅ 音效系统已启用
```

## 📊 完整修复清单

- [x] ✅ 在 `Cargo.toml` 启用 `audio` 特性
- [x] ✅ 添加详细的音效加载日志
- [x] ✅ 设置音量为 50% (可调整)
- [ ] ⚠️ 解决 ALSA 设备问题（环境相关）

## 🔧 当前状态

```
音频特性: ✅ 已启用
音效文件: ✅ 已加载 (shoot.wav, powerup.wav)
音频设备: ⚠️ 需要配置 (见上文解决方案)
```

## 💡 推荐设置

### 开发环境（有声音）
```bash
# 确保有音频设备
pulseaudio --check || pulseaudio --start
cargo run
```

### 测试环境（无需声音）
```bash
# 创建虚拟音频设备
cat > ~/.asoundrc << 'EOF'
pcm.!default {
    type plug
    slave.pcm "null"
}
EOF

cargo run
```

### CI/CD 环境
禁用音频或使用虚拟设备

## 🎯 下一步

1. **本地开发**：配置真实音频设备
2. **远程/容器**：使用虚拟音频设备
3. **测试**：添加更多音效文件（explosion.wav, hit.wav）

---

**快速修复**：创建虚拟音频设备
```bash
echo 'pcm.!default { type plug; slave.pcm "null"; }' > ~/.asoundrc
cargo run
```

游戏将正常运行，但不会输出声音。
