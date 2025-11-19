# 字体文件

## 快速设置（使用系统字体）

运行以下命令复制系统字体到项目：

```bash
cp /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf assets/fonts/
```

或者使用 Ubuntu Sans（更圆润）：

```bash
cp /usr/share/fonts/truetype/ubuntu/UbuntuSans[wdth,wght].ttf assets/fonts/ubuntu.ttf
```

## 下载字体（推荐）

如果系统没有这些字体，可以从以下网站下载：

### 选项 1: Noto Sans（推荐，支持中文）
- 下载链接: https://fonts.google.com/noto/specimen/Noto+Sans
- 点击 "Download family" 下载
- 解压后将 `NotoSans-Regular.ttf` 复制到此目录
- 重命名为 `font.ttf`

### 选项 2: Roboto（圆润现代）
- 下载链接: https://fonts.google.com/specimen/Roboto
- 下载 Regular 400 字重
- 复制到此目录并重命名为 `font.ttf`

### 选项 3: Inter（极简现代）
- 下载链接: https://fonts.google.com/specimen/Inter
- 下载 Regular 字重
- 复制到此目录并重命名为 `font.ttf`

## 文件要求

游戏会尝试加载以下字体（按优先级）：
1. `assets/fonts/font.ttf` - 自定义字体
2. `assets/fonts/DejaVuSans.ttf` - 系统字体
3. 如果都没有，将使用 Macroquad 默认字体

## 支持的格式

- TrueType Font (`.ttf`)
- OpenType Font (`.otf`)
