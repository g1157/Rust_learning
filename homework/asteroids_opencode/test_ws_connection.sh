#!/bin/bash

echo "🧪 测试 WebSocket 连接"
echo "======================="
echo ""

# 检查服务器是否运行
echo "1️⃣  检查服务器状态..."
if lsof -i :9001 > /dev/null 2>&1; then
    echo "   ✓ WebSocket 服务器运行中 (端口 9001)"
else
    echo "   ✗ WebSocket 服务器未运行"
    echo "   请运行: cd server && cargo run"
    exit 1
fi

if lsof -i :8000 > /dev/null 2>&1; then
    echo "   ✓ Web 服务器运行中 (端口 8000)"
else
    echo "   ✗ Web 服务器未运行"
    echo "   请运行: cd web && python3 -m http.server 8000"
    exit 1
fi

echo ""
echo "2️⃣  使用 websocat 测试连接..."

# 检查是否安装了 websocat
if ! command -v websocat &> /dev/null; then
    echo "   ⚠️  未安装 websocat，尝试使用 curl..."
    
    # 使用 curl 测试 HTTP 升级
    echo ""
    echo "3️⃣  测试 WebSocket 握手..."
    timeout 2 curl -i -N \
        -H "Connection: Upgrade" \
        -H "Upgrade: websocket" \
        -H "Sec-WebSocket-Version: 13" \
        -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
        http://localhost:9001/ 2>&1 | head -20
    
    echo ""
    echo "📊 测试结果："
    echo "   如果看到 '101 Switching Protocols'，说明服务器正常"
    echo "   如果看到 '400 Bad Request'，说明服务器不支持 WebSocket"
    echo ""
    echo "4️⃣  打开浏览器测试："
    echo "   测试页面: http://localhost:8000/test_websocket.html"
    echo "   主游戏: http://localhost:8000"
    echo ""
    echo "   在浏览器控制台 (F12) 查看日志"
else
    echo "   ✓ websocat 可用"
    echo ""
    echo "3️⃣  发送测试消息..."
    
    # 创建临时测试消息
    TEST_MSG='{"type":"Join","data":{"nickname":"TestPlayer"}}'
    
    echo "   发送: $TEST_MSG"
    echo "$TEST_MSG" | timeout 3 websocat ws://localhost:9001 2>&1 | head -5
    
    echo ""
    echo "📊 如果收到响应，说明服务器通信正常"
fi

echo ""
echo "5️⃣  查看服务器日志..."
echo "   最近 5 条日志:"
tail -5 server/server.log 2>/dev/null || echo "   (无日志文件)"

echo ""
echo "✅ 测试完成！"
echo ""
echo "🌐 打开浏览器访问:"
echo "   主游戏: http://localhost:8000"
echo "   测试页: http://localhost:8000/test_websocket.html"
