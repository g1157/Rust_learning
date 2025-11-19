#!/bin/bash
# WASM 加载诊断脚本

cd "$(dirname "$0")/web"

echo "🔍 检查 Web 目录文件..."
echo "WASM 文件:"
ls -lh *.wasm 2>/dev/null || echo "  ❌ 没有找到 .wasm 文件"

echo ""
echo "资源文件:"
ls -lR assets/ 2>/dev/null | head -20

echo ""
echo "🌐 启动 HTTP 服务器（端口 8000）..."
echo "访问: http://localhost:8000"
echo ""
echo "📋 测试步骤:"
echo "1. 在浏览器中打开 http://localhost:8000"
echo "2. 按 F12 打开开发者工具"
echo "3. 查看 Console 标签页的错误信息"
echo "4. 查看 Network 标签页，检查 .wasm 文件是否成功加载"
echo ""
echo "❓ 常见问题:"
echo "  - 一直 Loading: 检查控制台是否有 JS 错误"
echo "  - 404 错误: 确认 asteroids_opencode.wasm 存在"
echo "  - MIME type 错误: 服务器已配置正确的 MIME type"
echo ""
echo "按 Ctrl+C 停止服务器"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

python3 -c "
import http.server
import socketserver

class WasmHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory='.', **kwargs)
    
    def end_headers(self):
        # 设置 WASM MIME type
        if self.path.endswith('.wasm'):
            self.send_header('Content-Type', 'application/wasm')
        # 禁用缓存，方便调试
        self.send_header('Cache-Control', 'no-store, no-cache, must-revalidate')
        self.send_header('Expires', '0')
        super().end_headers()
    
    def log_message(self, format, *args):
        # 彩色日志
        msg = format % args
        if '200' in msg:
            print(f'✅ {msg}')
        elif '304' in msg:
            print(f'🔄 {msg}')
        elif '404' in msg:
            print(f'❌ {msg}')
        else:
            print(f'ℹ️  {msg}')

PORT = 8000
socketserver.TCPServer.allow_reuse_address = True

with socketserver.TCPServer(('', PORT), WasmHandler) as httpd:
    print(f'🚀 服务器运行在 http://localhost:{PORT}')
    httpd.serve_forever()
"
