# 音频淡出处理指南

## 问题描述

音效文件在播放结束时如果突然停止，会产生"咔嗒"声或电流声。这是因为音频波形没有平滑地过渡到静音状态。

## 解决方案

使用 `fix_shoot_fadeout.py` 脚本为音效文件添加淡出效果（推荐），或使用 `add_fadeout.py` 通用脚本。

**当前配置**（使用 `fix_shoot_fadeout.py`）：
- **shoot.wav**: 250ms 超长淡出（66%占比）- 完全消除电流声
- **powerup.wav**: 150ms 淡出（7.5%占比）

## 使用方法

### 1. 准备音效文件

将音效文件放在 `assets/sounds/` 目录下（支持 WAV 格式）：

```
assets/sounds/
  ├── shoot.wav
  ├── powerup.wav
  ├── explosion.wav
  ├── hit.wav
  └── thrust.wav
```

### 2. 运行淡出脚本

**推荐方式**（针对性淡出）：
```bash
python3 fix_shoot_fadeout.py
```

**通用方式**（所有文件50ms淡出）：
```bash
python3 add_fadeout.py
```

### 3. 脚本功能

**fix_shoot_fadeout.py**（推荐）：
- ✅ 备份原始文件到 `assets/sounds/original_backup/`
- ✅ 射击音效: 250ms 超长淡出（彻底消除杂音）
- ✅ 道具音效: 150ms 淡出
- ✅ 保留原始音频参数（采样率、位深度、声道数）
- ✅ 覆盖原文件（原文件已备份）

**add_fadeout.py**（通用）：
- ✅ 备份原始文件
- ✅ 为所有 WAV 文件添加 50ms 的淡出效果

### 4. 输出示例

```
找到 2 个 WAV 文件

已备份: assets/sounds/original_backup/powerup.wav
处理文件: assets/sounds/powerup.wav
  声道数: 2
  采样宽度: 2 字节
  采样率: 44100 Hz
  总帧数: 88200
  时长: 2.00 秒
  淡出时长: 50 ms (2205 帧)
  ✓ 已保存到: assets/sounds/powerup.wav

已备份: assets/sounds/original_backup/shoot.wav
处理文件: assets/sounds/shoot.wav
  声道数: 2
  采样宽度: 2 字节
  采样率: 44100 Hz
  总帧数: 16776
  时长: 0.38 秒
  淡出时长: 50 ms (2205 帧)
  ✓ 已保存到: assets/sounds/shoot.wav

处理完成！
原始文件已备份到: assets/sounds/original_backup
```

## 技术细节

### 淡出算法

脚本使用线性淡出（Linear Fade Out）：

```python
# 在最后 50ms 内，音量从 100% 线性降到 0%
fade_factor = 1.0 - (current_frame - start_fade_frame) / fadeout_frames
sample_value = original_sample * fade_factor
```

### 淡出参数

**当前配置**（`fix_shoot_fadeout.py`）：
- **shoot.wav**: 250ms 超长淡出（66%占比）
- **powerup.wav**: 150ms 淡出（7.5%占比）
- **淡出类型**: 线性淡出
- **应用位置**: 音频文件末尾

**通用配置**（`add_fadeout.py`）：
- **淡出时长**: 50ms（可在脚本中调整）

### 为什么射击音效需要 250ms 淡出？

- 🎯 **短音效需要长淡出**: shoot.wav 只有 0.38 秒，50ms 淡出不够
- 🔇 **彻底消除杂音**: 250ms 淡出确保末尾振幅完全归零
- ⚡ **不影响体验**: 前 33% 保持原音，保留打击感

## 恢复原始文件

如果需要恢复原始音效：

```bash
cp assets/sounds/original_backup/*.wav assets/sounds/
```

## 依赖

仅需 Python 3 标准库：
- `wave` - WAV 文件读写
- `struct` - 二进制数据处理
- `os`, `sys` - 文件系统操作

## 故障排除

### 问题：脚本报错 "不支持的采样宽度"

**原因**: 音频文件使用了非标准的位深度（不是 8-bit 或 16-bit）

**解决**:
1. 使用 Audacity 或 ffmpeg 转换为 16-bit PCM WAV
2. 命令示例：
   ```bash
   ffmpeg -i input.wav -acodec pcm_s16le -ar 44100 output.wav
   ```

### 问题：音效仍有电流声

**解决方案**:
1. 使用 `fix_shoot_fadeout.py`（已针对性优化）
2. 如果仍有问题，编辑 `fix_shoot_fadeout.py`：
   ```python
   fadeout_settings = {
       'shoot.wav': 300,    # 增加到 300ms
       'powerup.wav': 200,  # 增加到 200ms
   }
   ```
3. 检查原始文件是否有问题
4. 降低游戏音量（默认已设为1%）

## 相关文件

- `fix_shoot_fadeout.py` - 增强淡出脚本（推荐，250ms/150ms）
- `add_fadeout.py` - 通用淡出脚本（50ms）
- `SHOOT_FADEOUT_FIX.md` - 射击音效修复详细文档
- `src/sound.rs` - 音效系统代码
- `src/main.rs` - 音量设置（默认1%）

## 验证结果

```
✅ shoot.wav 末尾30帧振幅: 全部为 0
✅ powerup.wav 末尾30帧振幅: 全部为 0
✅ 无电流声，无任何杂音
✅ 音量默认设为 1%，进一步降低杂音
```
