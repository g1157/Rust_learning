# 🎮 双人在线对战测试指南

## ✅ 前置准备

### 1. 确认服务运行
```bash
# 检查 WebSocket 服务器
lsof -i :9001

# 检查 Web 服务器
lsof -i :8000

# 如果没有运行，启动它们：
cd server && cargo run &
cd web && python3 -m http.server 8000 &
```

### 2. 打开浏览器
- 推荐使用 Chrome/Edge 的隐身模式（避免缓存问题）
- 或者按 **Ctrl+Shift+R** 强制刷新

---

## 🧪 测试步骤

### 步骤 1: 打开两个浏览器标签页

**标签页 1 (玩家1)**:
1. 访问: `http://localhost:8000`
2. 按 **F12** 打开控制台
3. 点击主菜单的 **"Online"**
4. 输入昵称: `Player1`
5. 按 **Enter** 或点击确认

**预期输出** (控制台):
```
[WS] WebSocket client initialized (FFI mode)
```

**标签页 2 (玩家2)**:
1. 打开新标签页: `http://localhost:8000`
2. 按 **F12** 打开控制台
3. 点击主菜单的 **"Online"**
4. 输入昵称: `Player2`
5. 按 **Enter**

---

### 步骤 2: 连接到服务器

**两个标签页都执行**:
1. 点击 **"Connect"** 按钮

**预期输出** (两个控制台都应该显示):
```javascript
[WS] Connecting to: ws://localhost:9001
[WS] Connected!
[WS] ← Received: {"type":"Connected","player_id":"xxx-xxx-xxx"}
```

**如果失败**:
- 检查服务器是否运行: `lsof -i :9001`
- 查看服务器日志: `tail -f server/server.log`

---

### 步骤 3: 加入队列

**两个标签页都执行**:
1. 选择相同的游戏模式（Survival 或 Duel）
2. 点击对应模式按钮

**玩家1 控制台预期输出**:
```javascript
[WS] → Sent: {"type":"JoinQueue","mode":"Survival","nickname":"Player1"}
```

**玩家2 控制台预期输出**:
```javascript
[WS] → Sent: {"type":"JoinQueue","mode":"Survival","nickname":"Player2"}
```

**服务器日志预期输出**:
```
📨 收到消息: JoinQueue { mode: Survival, nickname: "Player1" }
🎮 玩家 Player1 加入 Survival 队列
📨 收到消息: JoinQueue { mode: Survival, nickname: "Player2" }
🎮 玩家 Player2 加入 Survival 队列
```

---

### 步骤 4: 等待匹配

**预期**:
- 两个玩家都应该进入 "等待匹配" 界面
- 界面显示搜索动画和 "Searching for players..."

**控制台预期输出** (两个都应该收到):
```javascript
[WS] ← Received: {"type":"MatchFound","room_id":"1","players":["player_id_1","player_id_2"],"mode":"Survival"}
匹配成功! 房间: 1, 玩家: [...], 模式: Survival
```

**服务器日志预期输出**:
```
🎉 匹配成功! 房间 1: Player1 vs Player2 (Survival)
📤 发送 MatchFound 到 2 个玩家
```

---

### 步骤 5: 游戏开始

**预期**:
- 短暂等待后，两个客户端都应该收到 `GameStart` 消息
- 游戏界面出现，显示两架飞船

**控制台预期输出**:
```javascript
[WS] ← Received: {"type":"GameStart"}
游戏开始!
```

**服务器日志预期输出**:
```
🚀 游戏开始! 房间 1
```

---

### 步骤 6: 游戏进行中

**玩家1 控制** (W/A/S/D):
- **W**: 推进
- **A**: 左转
- **D**: 右转
- **Space**: 射击

**玩家2 控制** (方向键):
- **↑**: 推进
- **←**: 左转
- **→**: 右转
- **Enter**: 射击

**预期行为**:
1. 按键时，控制台应显示发送输入:
   ```javascript
   [WS] → Sent: {"type":"GameInput","keys":["thrust"]}
   [WS] → Sent: {"type":"GameInput","keys":["left","shoot"]}
   ```

2. 每秒应该收到多次游戏状态更新:
   ```javascript
   [WS] ← Received: {"type":"GameState","players":[...],"asteroids":[...]}
   收到游戏状态: 2 玩家, 5 小行星
   ```

3. 飞船应该移动
4. 可以看到对方的飞船
5. 小行星应该同步显示

---

### 步骤 7: 游戏结束

**预期触发条件**:
- Survival: 所有玩家生命值为 0
- Duel: 一方生命值为 0

**控制台预期输出**:
```javascript
[WS] ← Received: {"type":"GameOver","winner":"player_id","scores":[["Player1",100],["Player2",80]]}
游戏结束! 胜者: Some("player_id"), 分数: [...]
```

**游戏界面**:
- 显示 Game Over 界面
- 显示胜者和分数

---

## 🐛 故障排查

### 问题 1: 无法连接到服务器

**症状**: 点击 Connect 后没有反应或报错

**检查**:
```bash
# 1. 确认服务器运行
ps aux | grep server

# 2. 查看服务器日志
tail -f server/server.log

# 3. 测试连接
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" -H "Sec-WebSocket-Key: test==" \
  http://localhost:9001/
```

**解决**:
```bash
# 重启服务器
pkill server
cd server && cargo run
```

---

### 问题 2: 看不到 JavaScript 日志

**症状**: 控制台没有 `[WS]` 输出

**原因**: 浏览器缓存

**解决**:
1. 按 **Ctrl+Shift+R** 强制刷新
2. 或清除缓存: F12 → Network → "Disable cache"
3. 或使用隐身模式

---

### 问题 3: 单人无法匹配

**症状**: 一直显示 "Searching for players..."

**原因**: 需要 2 个玩家才能匹配

**解决**:
- 打开第二个浏览器标签页
- 或使用不同浏览器
- 确保两个玩家选择**相同的游戏模式**

---

### 问题 4: 匹配后没有进入游戏

**症状**: 收到 `MatchFound` 但没有 `GameStart`

**检查服务器日志**:
```bash
tail -20 server/server.log
```

**可能原因**:
- 服务器逻辑未实现自动 GameStart
- 需要玩家发送 Ready 消息

**临时解决** (如果服务器支持):
在控制台手动发送:
```javascript
window.wsSend('{"type":"Ready"}')
```

---

### 问题 5: 游戏中看不到对方

**症状**: 游戏开始了，但只看到自己的飞船

**检查**:
1. 控制台是否收到 `GameState` 消息
2. `GameState` 中是否包含 2 个玩家

**调试**:
在控制台运行:
```javascript
// 查看消息队列
window.wsMessageQueue.slice(-5)
```

---

### 问题 6: 输入延迟或不同步

**症状**: 按键后飞船移动延迟

**原因**: 网络延迟或服务器处理慢

**检查延迟**:
- 游戏界面应显示 ping 值
- 正常应该 < 50ms (本地)

**优化**:
- 客户端预测（需要额外开发）
- 降低更新频率

---

## 📊 成功标志

**如果看到以下现象，说明在线对战成功**:

- ✅ 两个浏览器都连接成功
- ✅ 匹配成功，房间 ID 显示
- ✅ 游戏开始，两架飞船出现
- ✅ 控制自己的飞船，能看到移动
- ✅ 能看到对方飞船的移动
- ✅ 小行星同步显示
- ✅ 游戏结束时显示结果

---

## 📝 测试清单

### 连接测试
- [ ] 玩家1 连接成功
- [ ] 玩家2 连接成功
- [ ] 两个都收到 `Connected` 消息

### 匹配测试
- [ ] 玩家1 加入队列
- [ ] 玩家2 加入队列
- [ ] 两个都收到 `MatchFound`

### 游戏测试
- [ ] 收到 `GameStart` 消息
- [ ] 看到两架飞船
- [ ] 玩家1 控制正常
- [ ] 玩家2 控制正常
- [ ] 输入同步（对方能看到我的移动）
- [ ] 状态同步（我能看到对方的移动）
- [ ] 小行星同步

### 结束测试
- [ ] 游戏结束触发
- [ ] 收到 `GameOver` 消息
- [ ] 显示正确的胜者和分数

---

## 🎯 下一步改进

如果基础功能正常，可以考虑：

1. **客户端预测**: 减少输入延迟
2. **插值**: 使对方飞船移动更流畅
3. **断线重连**: 处理网络断开
4. **观战模式**: 允许观看正在进行的游戏
5. **排行榜**: 记录玩家分数
6. **房间系统**: 支持创建私人房间

---

**测试时间**: 预计 10-15 分钟

**准备好了吗？** 打开浏览器，开始测试！🚀
