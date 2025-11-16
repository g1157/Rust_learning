# 音效系统故障排除指南

## ✅ 当前状态

音效文件已成功加载：
- ✅ `shoot.wav` (射击音效)
- ✅ `powerup.wav` (道具音效)
- ✅ 音效系统已启用

## 🔊 音效触发条件

### 射击音效 (shoot.wav)
**触发时机**: 玩家按下射击键时
- **Player 1**: Space 键
- **Player 2**: Enter 键

**测试步骤**:
1. 启动游戏 `cargo run`
2. 主菜单选择 Survival 或 Duel
3. 按 Enter 开始游戏
4. 按 Space 或 Enter 射击
5. 应该听到 "哒" 的射击声

### 道具音效 (powerup.wav)
**触发时机**: 飞船碰到护盾道具时

**测试步骤**:
1. 进入 Survival 模式
2. 等待蓝色护盾图标出现
3. 驾驶飞船碰到护盾
4. 应该听到道具拾取音效

## 🐛 听不到声音？检查清单

### 1. 检查系统音量
```bash
# Linux - 检查音量
amixer get Master

# 调整音量
alsamixer
```

### 2. 检查音频设备
```bash
# 列出音频设备
pactl list sinks short

# 测试音频播放
paplay /usr/share/sounds/alsa/Front_Center.wav
```

### 3. 测试音效文件
```bash
# 用其他播放器测试
ffplay assets/sounds/shoot.wav
# 或
aplay assets/sounds/shoot.wav
```

### 4. 检查文件格式
```bash
# 查看文件信息
file assets/sounds/shoot.wav
ffprobe assets/sounds/shoot.wav
```

**支持的格式**:
- WAV: PCM, 16-bit, 22050Hz-48000Hz
- OGG: Vorbis 编码

### 5. 检查音量设置

游戏音量设置为 **50%** (0.5)

如果音效文件本身音量很小，可能听不清。

## 🔧 调整音量

编辑 `src/sound.rs` 文件，修改音量参数：

```rust
play_sound(
    sound,
    PlaySoundParams {
        looped: false,
        volume: 1.0, // 改为 1.0 (100%) 或更大
    },
);
```

可选值：
- `0.5` - 50% 音量（当前设置）
- `1.0` - 100% 音量（最大）
- `0.3` - 30% 音量（较小）

## 📝 添加更多音效

还需要的音效文件：

### 必需音效
- `explosion.wav` - 小行星爆炸 ⚠️ 缺失
- `hit.wav` - 飞船碰撞 ⚠️ 缺失

### 可选音效
- `thrust.wav` - 推进器（可选）

### 下载来源

1. **Kenney.nl** (推荐)
   - 网址: https://kenney.nl/assets/digital-audio
   - 或: https://kenney.nl/assets/impact-sounds
   - 免费、CC0 协议

2. **Freesound.org**
   - 搜索: "laser", "explosion", "8-bit"
   - 需要注册

3. **OpenGameArt.org**
   - 分类: Sound Effects > Sci-Fi

## 🎮 游戏内音效触发总结

| 音效 | 触发事件 | 按键/操作 | 状态 |
|------|---------|----------|------|
| **shoot.wav** | 射击子弹 | Space/Enter | ✅ 已加载 |
| **powerup.wav** | 拾取道具 | 碰到蓝色护盾 | ✅ 已加载 |
| **explosion.wav** | 小行星爆炸 | 子弹击中小行星 | ❌ 缺失 |
| **hit.wav** | 飞船碰撞 | 撞到小行星 | ❌ 缺失 |
| **thrust.wav** | 推进器 | W/I 键 | ❌ 缺失 (可选) |

## 🚀 快速测试

运行游戏后：

```bash
cargo run
```

1. **测试射击音效**:
   - 选择 Survival
   - 按 Enter 开始
   - 疯狂按 Space 键
   - 应该听到连续的射击声

2. **测试道具音效**:
   - 等待护盾道具出现
   - 飞过去拾取
   - 应该听到"叮"的声音

## 💡 进阶：添加音量控制

如果需要在设置中添加音量滑块，可以：

1. 在 `GameSettings` 添加 `sound_volume: f32` 字段
2. 在设置界面添加音量调节选项
3. 修改 `SoundSystem::play()` 接受音量参数

需要我帮你实现吗？

## 🎵 音效文件要求

### 推荐格式
- **格式**: WAV (PCM)
- **采样率**: 22050 Hz 或 44100 Hz
- **位深度**: 16-bit
- **声道**: Mono (单声道) 或 Stereo (立体声)
- **时长**: 0.1s - 2s (短促音效)

### 文件大小
- shoot.wav: ~50-100KB ✅ 当前66KB
- powerup.wav: ~100-300KB ✅ 当前345KB
- explosion.wav: ~100-200KB
- hit.wav: ~50-100KB

---

**提示**: 如果仍然听不到声音，请检查：
1. 系统音量是否静音
2. 耳机/音箱是否连接
3. 其他应用程序能否播放声音
4. Macroquad 是否支持你的音频后端
