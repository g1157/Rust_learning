#!/bin/bash
# 简单的 HTTP 服务器用于测试 WASM

echo "🚀 Starting Asteroids Web Server..."
echo ""
echo "Open your browser and visit:"
echo "  http://localhost:8000"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# 使用 Python 的简单 HTTP 服务器
python3 -m http.server 8000
