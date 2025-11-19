#!/bin/bash
# 构建 Web 版本的完整脚本

set -e  # 遇到错误立即退出

echo "🚀 构建 Asteroids Web 版本"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 1. 编译 WASM
echo "📦 步骤 1/3: 编译 WASM..."
cargo build --target wasm32-unknown-unknown --release
echo "✅ WASM 编译完成"

# 2. 生成 wasm-bindgen 绑定
echo ""
echo "🔗 步骤 2/3: 生成 JS 绑定..."
wasm-bindgen target/wasm32-unknown-unknown/release/asteroids_opencode.wasm \
  --out-dir web/pkg \
  --target web \
  --no-typescript
echo "✅ JS 绑定生成完成"

# 3. 复制资源文件（如果需要）
echo ""
echo "📁 步骤 3/3: 检查资源文件..."
if [ ! -d "web/assets" ]; then
    echo "⚠️  警告: web/assets/ 目录不存在，正在复制..."
    cp -r assets web/
    echo "✅ 资源文件已复制"
else
    echo "✅ 资源文件已存在"
fi

# 显示文件大小
echo ""
echo "📊 构建结果:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ls -lh web/pkg/asteroids_opencode_bg.wasm | awk '{print "WASM 文件: " $5 " (" $9 ")"}'
ls -lh web/pkg/asteroids_opencode.js | awk '{print "JS 绑定:  " $5 " (" $9 ")"}'
ls -lh web/mq_js_bundle.js | awk '{print "Macroquad: " $5 " (" $9 ")"}'

echo ""
echo "✨ 构建完成！"
echo ""
echo "🌐 启动测试服务器:"
echo "   cd web && python3 -m http.server 8000"
echo ""
echo "🔗 访问地址:"
echo "   http://localhost:8000"
echo ""
echo "📝 测试 LocalStorage:"
echo "   1. 打开浏览器开发者工具 (F12)"
echo "   2. Application → Local Storage"
echo "   3. 游玩游戏，解锁成就"
echo "   4. 刷新页面，验证成就保留"
