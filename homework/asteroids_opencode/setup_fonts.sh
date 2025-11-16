#!/bin/bash
# 快速复制系统字体到项目

echo "正在复制系统字体到 assets/fonts/..."

# 创建目录
mkdir -p assets/fonts

# 尝试复制 DejaVu Sans（圆润，支持多语言）
if [ -f /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf ]; then
    cp /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf assets/fonts/
    echo "✓ 已复制 DejaVuSans.ttf"
else
    echo "✗ 未找到 DejaVuSans.ttf"
fi

# 尝试复制 Ubuntu Sans（更圆润）
if [ -f /usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf ]; then
    cp /usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf assets/fonts/ubuntu.ttf
    echo "✓ 已复制 ubuntu.ttf"
else
    echo "✗ 未找到 Ubuntu font"
fi

echo ""
echo "完成！运行 'cargo run' 查看效果"
echo "如果想使用其他字体，请将 .ttf 文件复制到 assets/fonts/ 并命名为 font.ttf"
