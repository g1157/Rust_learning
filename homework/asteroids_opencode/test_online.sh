#!/bin/bash

echo "🎮 Asteroids 在线多人测试"
echo "=========================="
echo ""

# 检查服务器是否已经在运行
if lsof -Pi :9001 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo "⚠️  WebSocket 服务器已在 9001 端口运行"
else
    echo "🚀 启动 WebSocket 服务器 (端口 9001)..."
    cd server
    cargo run &
    SERVER_PID=$!
    cd ..
    echo "   服务器进程 PID: $SERVER_PID"
    sleep 3
fi

echo ""

# 检查 Web 服务器是否已经在运行
if lsof -Pi :8000 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo "⚠️  Web 服务器已在 8000 端口运行"
else
    echo "🌐 启动 Web 服务器 (端口 8000)..."
    cd web
    python3 -m http.server 8000 &
    WEB_PID=$!
    cd ..
    echo "   Web 服务器进程 PID: $WEB_PID"
    sleep 2
fi

echo ""
echo "✅ 测试环境已就绪！"
echo ""
echo "📋 测试步骤:"
echo "   1. 打开两个浏览器标签页"
echo "   2. 都访问 http://localhost:8000"
echo "   3. 按 F12 打开控制台（查看日志）"
echo "   4. 选择 'Online' 模式"
echo "   5. 输入昵称（如 Player1, Player2）"
echo "   6. 点击 Connect 按钮"
echo "   7. 选择相同的游戏模式 (Survival 或 Duel)"
echo "   8. 等待匹配（需要 2 个玩家）"
echo ""
echo "🧪 测试页面:"
echo "   http://localhost:8000/test_websocket.html"
echo ""
echo "📖 详细测试指南:"
echo "   cat ONLINE_MULTIPLAYER_TEST.md"
echo ""
echo "🔍 查看服务器日志:"
echo "   tail -f server/server.log"
echo ""
echo "🛑 停止服务:"
echo "   pkill -f 'target/debug/server'"
echo "   pkill -f 'python3.*http.server'"
echo ""
