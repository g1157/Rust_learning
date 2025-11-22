#!/bin/bash
# 构建 Web 版本的完整脚本

set -e  # 遇到错误立即退出

echo "🚀 构建 Asteroids Web 版本"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 1. 编译 WASM
echo "📦 步骤 1/4: 编译 WASM..."
cargo build --target wasm32-unknown-unknown --release
echo "✅ WASM 编译完成"

# 2. 复制 WASM 到 web 目录
echo ""
echo "📋 步骤 2/4: 复制 WASM 文件..."
cp target/wasm32-unknown-unknown/release/asteroids_opencode.wasm web/
echo "✅ WASM 文件已复制"

# 3. 复制资源文件（如果需要）
echo ""
echo "📁 步骤 3/4: 检查资源文件..."
if [ ! -d "web/assets" ]; then
    echo "⚠️  警告: web/assets/ 目录不存在，正在复制..."
    cp -r assets web/
    echo "✅ 资源文件已复制"
else
    echo "✅ 资源文件已存在"
fi

# 4. 验证符号链接
echo ""
echo "🔗 步骤 4/4: 验证符号链接..."
cd web/assets/sounds/
if [ ! -f "explosion.wav" ]; then
    ln -sf powerup.wav explosion.wav
    echo "   ✓ 创建 explosion.wav 链接"
fi
if [ ! -f "hit.wav" ]; then
    ln -sf powerup.wav hit.wav
    echo "   ✓ 创建 hit.wav 链接"
fi
if [ ! -f "thrust.wav" ]; then
    ln -sf shoot.wav thrust.wav
    echo "   ✓ 创建 thrust.wav 链接"
fi
cd ../fonts/
if [ ! -f "font.ttf" ]; then
    ln -sf DejaVuSans.ttf font.ttf
    echo "   ✓ 创建 font.ttf 链接"
fi
cd ../../..

echo "✅ 符号链接验证完成"

# 显示文件大小
echo ""
echo "📊 构建结果:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ls -lh web/asteroids_opencode.wasm | awk '{print "WASM 主程序: " $5}'
ls -lh web/mq_js_bundle.js | awk '{print "JS 引擎:     " $5}'
echo ""
echo "总大小: $(du -sh web/ | awk '{print $1}')"

echo ""
echo "✨ 构建完成！"
echo ""
echo "🌐 本地测试:"
echo "   ./serve.sh"
echo "   或者: cd web && python3 -m http.server 8000"
echo ""
echo "🔗 访问地址:"
echo "   http://localhost:8000"
echo ""
echo "💾 功能验证:"
echo "   ✓ 游戏窗口大小: 1024x768"
echo "   ✓ LocalStorage 持久化（成就系统）"
echo "   ✓ 音效和字体"
echo "   ✓ 双人本地游戏"
echo ""
echo "📦 部署文件:"
echo "   web/ 目录包含所有需要的文件"
echo "   参考 DEPLOYMENT.md 了解详细部署步骤"
