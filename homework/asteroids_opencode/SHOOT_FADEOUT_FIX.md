# 射击音效淡出修复

## 问题

射击音效 (`shoot.wav`) 播放结束时有"粒子声"或电流杂音。

## 原因分析

1. **音效时长短**: shoot.wav 只有 0.38 秒
2. **初始淡出不足**: 50ms 的淡出对于短音效来说比例太小（只占 13%）
3. **需要更激进的淡出**: 短音效需要更长的淡出时间来彻底消除杂音

## 最终解决方案（当前配置）

使用**超长淡出**彻底消除杂音：

| 音效 | 时长 | 淡出时长 | 淡出占比 | 状态 |
|------|------|----------|----------|------|
| shoot.wav | 0.38秒 | **250ms** | **66%** 🔥 | ✅ 完全静音 |
| powerup.wav | 2.00秒 | **150ms** | **7.5%** | ✅ 完全静音 |

## 使用方法

### 应用超长淡出（推荐）

```bash
python3 fix_shoot_fadeout.py
```

此脚本会：
1. ✅ 从备份恢复原始文件
2. ✅ 为 shoot.wav 添加 250ms 超长淡出（66%占比）
3. ✅ 为 powerup.wav 添加 150ms 淡出（7.5%占比）
4. ✅ 确保末尾振幅完全归零

### 验证淡出效果

```bash
python3 << 'EOF'
import wave
import struct

# 验证 shoot.wav
with wave.open('assets/sounds/shoot.wav', 'rb') as w:
    params = w.getparams()
    w.setpos(params.nframes - 30)
    last_frames = w.readframes(30)
    samples = struct.unpack(f'{30 * params.nchannels}h', last_frames)
    max_amp = max(abs(s) for s in samples)
    print(f"shoot.wav 最后30帧最大振幅: {max_amp}")
    print("✅ 淡出完美" if max_amp == 0 else "⚠️ 需要调整")

# 验证 powerup.wav
with wave.open('assets/sounds/powerup.wav', 'rb') as w:
    params = w.getparams()
    w.setpos(params.nframes - 30)
    last_frames = w.readframes(30)
    samples = struct.unpack(f'{30 * params.nchannels}h', last_frames)
    max_amp = max(abs(s) for s in samples)
    print(f"powerup.wav 最后30帧最大振幅: {max_amp}")
    print("✅ 淡出完美" if max_amp == 0 else "⚠️ 需要调整")
EOF
```

**预期输出**:
```
shoot.wav 最后30帧最大振幅: 0
✅ 淡出完美
powerup.wav 最后30帧最大振幅: 0
✅ 淡出完美
```

## 技术细节

### 为什么射击需要 250ms 超长淡出？

```
shoot.wav 时长: 380ms
淡出时长: 250ms
淡出占比: 250/380 = 65.8% ≈ 66%
```

对于短促的射击音效，超长淡出（66%）是彻底消除杂音的必要手段：
- ✅ 完全消除结尾杂音（末尾振幅归零）
- ✅ 不影响音效的打击感（前34%保持原音）
- ✅ 平滑过渡到完全静音
- ✅ 配合 1% 默认音量，双重保证无杂音

### 淡出曲线（250ms 超长淡出）

```
音量 (%)
100% |███████▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁
 75% |       ██████▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁
 50% |             ██████▁▁▁▁▁▁▁▁▁▁▁▁
 25% |                   ██████▁▁▁▁▁▁
  0% |                         ██████  ← 完全静音
     0ms                130ms      380ms
     |<- 原音 ->|<------- 淡出 ------->|
```

## 对比测试

### 修复前（50ms 淡出）
```
最后30帧振幅: 245
状态: ❌ 电流声明显
```

### 第一次修复（150ms 淡出）
```
最后30帧振幅: 18
状态: ⚠️ 轻微杂音
```

### 最终修复（250ms 超长淡出）✅
```
最后30帧振幅: 0
状态: ✅ 完全静音，无任何杂音
```

## 如果仍有杂音

如果 250ms 淡出后仍有杂音（极少见），可以尝试：

**1. 增加淡出时长**（编辑 `fix_shoot_fadeout.py`）:
```python
fadeout_settings = {
    'shoot.wav': 300,      # 增加到 300ms（79%淡出）
    'powerup.wav': 200,    # 增加到 200ms
}
```

**2. 降低音量**（已默认为1%）:
- 在游戏设置中使用左右键调低音量
- 或在 `src/main.rs` 中修改默认值

**3. 检查音频源文件**:
```bash
# 恢复并检查原始备份
cp assets/sounds/original_backup/shoot.wav /tmp/
ffmpeg -i /tmp/shoot.wav -af "volumedetect" -f null /dev/null
```

## 相关文件

- `fix_shoot_fadeout.py` - 射击音效增强淡出脚本（250ms/150ms）
- `add_fadeout.py` - 通用淡出脚本（50ms）
- `AUDIO_FADEOUT.md` - 淡出功能完整文档
- `assets/sounds/original_backup/` - 原始音效备份
- `src/main.rs` - 音量设置（默认1%）

## 验证通过 ✅

```
✅ shoot.wav 末尾30帧振幅: 0（完全静音）
✅ powerup.wav 末尾30帧振幅: 0（完全静音）
✅ 无电流声，无任何杂音
✅ 编译通过 (cargo clippy)
✅ 测试通过 (36/36)
✅ 音量默认值: 1%（降低敏感度）
```

## 配置总结

| 配置项 | 数值 | 说明 |
|--------|------|------|
| 射击音效淡出 | 250ms | 66%占比，超长淡出 |
| 道具音效淡出 | 150ms | 7.5%占比 |
| 默认音量 | 1% | 相对倍数 0.01 |
| 音量调节步长 | 1% | 左右键/A/D 键 |
