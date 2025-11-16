# 字体系统使用指南

## 🎨 功能概述

游戏现在支持4种可切换的UI字体，可以在设置菜单中实时切换。

## 📋 可用字体

### 1. Default（默认）
- **风格**: 像素风格，方正
- **优点**: 复古游戏感觉，无需额外文件
- **缺点**: 不够圆润

### 2. DejaVu Sans（推荐）⭐
- **风格**: 圆润，现代，专业
- **优点**: 支持多语言，易读性强
- **文件**: `assets/fonts/DejaVuSans.ttf`
- **大小**: ~742KB

### 3. Ubuntu
- **风格**: 非常圆润，现代简洁
- **优点**: 视觉效果优秀
- **文件**: `assets/fonts/ubuntu.ttf`
- **大小**: ~1.1MB

### 4. Custom（自定义）
- **风格**: 用户自定义
- **文件**: `assets/fonts/font.ttf`
- **说明**: 将任何 .ttf 字体文件重命名为 `font.ttf` 并放到此目录

## 🎮 使用方法

### 游戏内切换

1. 启动游戏：`cargo run`
2. 主菜单选择 **Settings**
3. 移动到 **UI Font** 选项
4. 使用左右键或 A/D 切换：
   ```
   Default → DejaVu Sans → Ubuntu → Custom → (循环)
   ```
5. 字体效果**立即生效**

### 快捷键

- **上/下** 或 **W/S**: 选择设置项
- **左/右** 或 **A/D**: 切换字体
- **ESC**: 返回主菜单

## 🔧 技术实现

### 字体加载

游戏启动时会自动加载所有可用字体：

```rust
字体加载状态:
  DejaVu Sans: ✓
  Ubuntu:      ✓
  Custom:      ✗
```

### 实时切换

- 字体切换无需重启游戏
- 主菜单和设置界面立即应用新字体
- 游戏内HUD保持默认字体（性能优化）

### 应用范围

字体影响以下UI元素：

✅ **主菜单**
- "Choose Your Adventure" 标题
- 3个模式卡片标题和描述
- 底部操作提示

✅ **设置界面**
- "Game Settings" 标题
- 底部操作提示

❌ **游戏内HUD**（使用默认字体）
- 生命值、分数显示
- 性能监控面板

## 📦 添加更多字体

### 方法1：使用系统字体

```bash
# 复制系统字体到项目
cp /usr/share/fonts/truetype/your-font.ttf assets/fonts/font.ttf
```

### 方法2：下载免费字体

推荐网站：
- **Google Fonts**: https://fonts.google.com/
  - Noto Sans, Roboto, Inter, Poppins
- **Font Squirrel**: https://www.fontsquirrel.com/
  - 100% 免费商用字体

### 方法3：使用现有脚本

```bash
./setup_fonts.sh
```

## 🎯 最佳实践

### 性能建议

- ✅ 主菜单/设置使用圆润字体（用户体验）
- ✅ 游戏内HUD使用默认字体（性能优先）
- ✅ 字体文件控制在 2MB 以内

### 可读性建议

- **DejaVu Sans**: 综合最佳，推荐默认
- **Ubuntu**: 适合现代简洁风格
- **Custom**: 可尝试 Roboto, Inter, Poppins

### 兼容性建议

- 使用支持英文的字体
- 如需中文支持，使用 Noto Sans CJK
- 避免使用过于花哨的装饰字体

## 🐛 故障排除

### 问题：字体显示为方框

**原因**: 字体文件不存在或损坏

**解决**:
1. 检查文件是否存在：`ls -lh assets/fonts/`
2. 尝试重新复制字体文件
3. 切换到 Default 字体

### 问题：Custom 字体不可用

**原因**: 未放置 `font.ttf` 文件

**解决**:
```bash
# 下载或复制字体文件
cp your-font.ttf assets/fonts/font.ttf
# 重启游戏
cargo run
```

### 问题：字体太小/太大

**说明**: 字体大小已优化，无需调整

主要字号：
- 标题: 48px
- 卡片标题: 32px
- 描述文字: 22px
- 提示文字: 26px

## 📊 完整设置列表

游戏设置系统现在包含 **9个可配置项**：

1. Starting Lives (1-9)
2. Ship Speed (0.5x-2.0x)
3. Asteroid Speed (0.5x-2.0x)
4. **UI Font** 🆕 (Default/DejaVu/Ubuntu/Custom)
5. Weapon Switch (ON/OFF)
6. Screen Shake (ON/OFF)
7. Slow Motion (ON/OFF)
8. Debug Panel (ON/OFF)
9. Reset to Defaults

## 🎉 更新日志

### v0.2.0 - 字体系统

- ✅ 添加字体选择系统（4个可选字体）
- ✅ 主菜单和设置界面支持自定义字体
- ✅ 实时字体切换，无需重启
- ✅ 自动加载所有可用字体
- ✅ 优雅降级（字体不可用时使用默认）

---

**提示**: 运行 `cargo run` 开始体验圆润字体！
