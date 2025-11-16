# 字体和音效设置指南

## 🎨 字体设置（让UI更圆润）

### 快速设置（推荐）

运行自动设置脚本：

```bash
./setup_fonts.sh
```

这会自动从系统复制 DejaVu Sans 字体到项目。

### 手动设置

#### 选项 1：使用系统字体

```bash
# DejaVu Sans（圆润，支持多语言）
cp /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf assets/fonts/

# 或者 Ubuntu Sans（更圆润）
cp /usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf assets/fonts/ubuntu.ttf
```

#### 选项 2：下载免费字体

访问以下网站下载字体（.ttf 文件）：

1. **Noto Sans**（推荐，支持中文）
   - https://fonts.google.com/noto/specimen/Noto+Sans
   - 下载后将 `NotoSans-Regular.ttf` 放到 `assets/fonts/`

2. **Roboto**（圆润现代）
   - https://fonts.google.com/specimen/Roboto
   - 下载 Regular 400 字重

3. **Inter**（极简现代）
   - https://fonts.google.com/specimen/Inter
   - 下载 Regular 字重

#### 字体优先级

游戏会按以下顺序尝试加载字体：
1. `assets/fonts/font.ttf` - 你的自定义字体
2. `assets/fonts/DejaVuSans.ttf` - DejaVu Sans
3. `assets/fonts/ubuntu.ttf` - Ubuntu Sans
4. Macroquad 默认字体（如果都没找到）

## 🔊 音效设置

### 需要的音效文件

将以下文件放到 `assets/sounds/` 目录：

- `shoot.wav` - 射击音效
- `explosion.wav` - 爆炸音效
- `powerup.wav` - 拾取道具音效
- `hit.wav` - 碰撞音效
- `thrust.wav` - 推进器音效（可选）

### 免费音效资源

#### 推荐网站（按优先级）

1. **Kenney.nl** ⭐⭐⭐⭐⭐
   - 网址: https://kenney.nl/assets
   - 推荐包: "Digital Audio", "Impact Sounds"
   - 优点: CC0 协议，免费商用，质量高

2. **Freesound.org** ⭐⭐⭐⭐
   - 网址: https://freesound.org/
   - 搜索: "laser shoot", "explosion", "powerup"
   - 优点: 资源丰富

3. **OpenGameArt.org** ⭐⭐⭐⭐
   - 网址: https://opengameart.org/
   - 分类: Sound Effects > Sci-Fi
   - 优点: 专注游戏

### 支持的音频格式

- WAV (`.wav`)
- OGG Vorbis (`.ogg`)

## 🎮 验证设置

运行游戏：

```bash
cargo run
```

启动时会显示：

```
✓ Custom font loaded successfully
  Font will be used for all UI text
```

或者：

```
✗ No custom font found in assets/fonts/
  Using default Macroquad font
```

音效系统会显示类似信息。

## 📝 注意事项

- 字体和音效文件不会提交到 git（已在 .gitignore 中配置）
- 如果没有字体/音效文件，游戏仍会正常运行
- 推荐使用 Regular（普通）字重的字体，避免太粗或太细
