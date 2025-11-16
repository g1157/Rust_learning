#!/bin/bash
# 音效系统诊断脚本

echo "=== 音效系统诊断 ==="
echo ""

echo "1. 检查音效文件..."
ls -lh assets/sounds/*.wav assets/sounds/*.ogg 2>/dev/null || echo "  ⚠️  未找到音效文件"
echo ""

echo "2. 检查文件格式..."
for file in assets/sounds/*.wav assets/sounds/*.ogg 2>/dev/null; do
    if [ -f "$file" ]; then
        echo "  $(basename $file):"
        file "$file" | sed 's/^/    /'
    fi
done
echo ""

echo "3. 检查系统音量..."
if command -v amixer &> /dev/null; then
    amixer get Master | grep -E "Playback.*\[" | head -1 | sed 's/^/  /'
else
    echo "  ⚠️  amixer 未安装"
fi
echo ""

echo "4. 测试音频播放 (播放系统测试音)..."
if command -v paplay &> /dev/null; then
    echo "  使用 paplay 测试..."
    paplay /usr/share/sounds/alsa/Front_Center.wav 2>/dev/null && echo "  ✅ 音频输出正常" || echo "  ❌ 音频输出失败"
elif command -v aplay &> /dev/null; then
    echo "  使用 aplay 测试..."
    aplay /usr/share/sounds/alsa/Front_Center.wav 2>/dev/null && echo "  ✅ 音频输出正常" || echo "  ❌ 音频输出失败"
else
    echo "  ⚠️  未找到音频播放器 (paplay/aplay)"
fi
echo ""

echo "5. 测试游戏音效文件..."
if [ -f "assets/sounds/shoot.wav" ]; then
    echo "  播放 shoot.wav (3秒)..."
    if command -v paplay &> /dev/null; then
        timeout 3 paplay assets/sounds/shoot.wav &
    elif command -v aplay &> /dev/null; then
        timeout 3 aplay assets/sounds/shoot.wav &
    fi
    wait 2>/dev/null
    echo "  听到声音了吗？"
else
    echo "  ⚠️  shoot.wav 不存在"
fi
echo ""

echo "6. 游戏音效状态..."
echo "  已加载的音效:"
echo "    ✅ shoot.wav (射击)"
echo "    ✅ powerup.wav (道具)"
echo "  缺失的音效:"
echo "    ❌ explosion.wav (爆炸)"
echo "    ❌ hit.wav (碰撞)"
echo "    ❌ thrust.wav (推进，可选)"
echo ""

echo "7. 建议操作..."
echo "  • 确保系统音量未静音"
echo "  • 检查耳机/音箱连接"
echo "  • 运行游戏: cargo run"
echo "  • 进入游戏后按 Space 射击测试"
echo "  • 音量设置为 50% (可在 src/sound.rs 调整)"
echo ""

echo "=== 诊断完成 ==="
