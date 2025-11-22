#!/bin/bash
# 创建部署包脚本

set -e

VERSION="0.3.0"
PACKAGE_NAME="asteroids-web-v${VERSION}"

echo "📦 创建部署包: ${PACKAGE_NAME}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 1. 先构建
echo ""
echo "🔨 步骤 1/4: 构建项目..."
./build_web.sh

# 2. 创建临时目录
echo ""
echo "📁 步骤 2/4: 准备打包目录..."
rm -rf /tmp/${PACKAGE_NAME}
mkdir -p /tmp/${PACKAGE_NAME}

# 3. 复制必要文件
echo ""
echo "📋 步骤 3/4: 复制文件..."
cp web/index.html /tmp/${PACKAGE_NAME}/
cp web/mq_js_bundle.js /tmp/${PACKAGE_NAME}/
cp web/asteroids_opencode.wasm /tmp/${PACKAGE_NAME}/
cp -r web/assets /tmp/${PACKAGE_NAME}/
cp DEPLOYMENT.md /tmp/${PACKAGE_NAME}/
cp README.md /tmp/${PACKAGE_NAME}/

# 创建版本信息文件
cat > /tmp/${PACKAGE_NAME}/VERSION.txt << EOF
Asteroids Web Version
━━━━━━━━━━━━━━━━━━━━
Version: ${VERSION}
Build Date: $(date '+%Y-%m-%d %H:%M:%S')
Engine: Macroquad 0.4.14
WASM Size: $(ls -lh web/asteroids_opencode.wasm | awk '{print $5}')

Features:
✓ Local multiplayer (2 players)
✓ Achievement system with LocalStorage
✓ Survival and Duel modes
✓ Particle effects and sound system

Quick Start:
1. Upload all files to your web server
2. Ensure proper MIME type for .wasm files (application/wasm)
3. Visit index.html in your browser

See DEPLOYMENT.md for detailed instructions.
EOF

# 4. 打包
echo ""
echo "🗜️  步骤 4/4: 创建压缩包..."
cd /tmp
tar -czf ${PACKAGE_NAME}.tar.gz ${PACKAGE_NAME}/
zip -r ${PACKAGE_NAME}.zip ${PACKAGE_NAME}/ > /dev/null

# 移动到项目目录
mv ${PACKAGE_NAME}.tar.gz ~/Rust_learning/homework/asteroids_opencode/
mv ${PACKAGE_NAME}.zip ~/Rust_learning/homework/asteroids_opencode/

# 清理
rm -rf /tmp/${PACKAGE_NAME}

echo ""
echo "✨ 打包完成！"
echo ""
echo "📦 部署包:"
echo "   ${PACKAGE_NAME}.tar.gz ($(ls -lh ${PACKAGE_NAME}.tar.gz | awk '{print $5}'))"
echo "   ${PACKAGE_NAME}.zip     ($(ls -lh ${PACKAGE_NAME}.zip | awk '{print $5}'))"
echo ""
echo "📤 上传到服务器:"
echo "   scp ${PACKAGE_NAME}.tar.gz user@server:/var/www/"
echo "   ssh user@server 'cd /var/www && tar -xzf ${PACKAGE_NAME}.tar.gz'"
echo ""
echo "🌐 或上传到 GitHub Releases:"
echo "   gh release create v${VERSION} ${PACKAGE_NAME}.tar.gz ${PACKAGE_NAME}.zip"
