#!/bin/bash
# 启动 Asteroids WebSocket 服务器

cd "$(dirname "$0")"

echo "🚀 启动 Asteroids 游戏服务器..."
echo ""

# 检查是否已编译
if [ ! -f "target/release/server" ]; then
    echo "📦 首次运行，正在编译..."
    cargo build --release
    echo ""
fi

echo "🎮 服务器启动中..."
echo "📡 WebSocket 地址: ws://localhost:9001"
echo "---"

cargo run --release
