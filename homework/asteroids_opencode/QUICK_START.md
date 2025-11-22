# 🚀 快速开始 - 在线多人测试

## 一键启动
```bash
./test_online.sh
```

## 测试步骤（5分钟）

### 1️⃣ 打开浏览器
- 打开两个标签页
- 都访问: http://localhost:8000
- 按 **F12** 打开控制台

### 2️⃣ 连接服务器
- 点击 **Online**
- 输入昵称（如 `Player1`, `Player2`）
- 点击 **Connect**

### 3️⃣ 开始游戏
- 选择相同模式（Survival/Duel）
- 等待匹配（需要2个玩家）
- 游戏自动开始

### 4️⃣ 控制
**玩家1 (W/A/S/D)**:
- W: 推进
- A: 左转
- D: 右转
- Space: 射击

**玩家2 (方向键)**:
- ↑: 推进
- ←: 左转
- →: 右转
- Enter: 射击

## 预期输出

### 控制台（浏览器 F12）
```
[WS] Connecting to: ws://localhost:9001
[WS] Connected!
[WS] ← Received: {"type":"Connected",...}
[WS] → Sent: {"type":"JoinQueue",...}
[WS] ← Received: {"type":"MatchFound",...}
[WS] ← Received: {"type":"GameStart"}
[WS] ← Received: {"type":"GameState",...}
```

### 服务器日志
```bash
tail -f server/server.log
```

## 故障排查

### 无法连接？
```bash
# 检查服务器
lsof -i :9001
lsof -i :8000

# 重启
pkill -f 'target/debug/server'
./test_online.sh
```

### 看不到日志？
- 按 **Ctrl+Shift+R** 强制刷新
- 清除浏览器缓存

### 单人无法匹配？
- 必须有 2 个玩家
- 确保选择相同模式

## 详细文档
- 测试指南: `ONLINE_MULTIPLAYER_TEST.md`
- 技术总结: `ONLINE_MULTIPLAYER.md`

---
**祝测试顺利！** 🎮
