# 在线模式测试指南

## 🎯 测试目标
验证 RefCell panic 已修复，可以成功进入在线多人模式

---

## 🚀 前置条件

### 1. 服务状态 ✅
```bash
# WebSocket 服务器 (PID 90861)
lsof -i :9001
# 应该显示: server  90861 blue

# HTTP 服务器
lsof -i :8000  
# 应该显示: python3 ... 8000
```

### 2. 文件版本 ✅
```bash
ls -lh web/asteroids_opencode.wasm
# 应该显示: 711K Nov 19 23:59 (刚刚编译)
```

---

## 📝 测试步骤

### Step 1: 打开浏览器
```
URL: http://localhost:8000/index.html
```

### Step 2: 打开开发者工具
- 按 `F12` 或 `Ctrl+Shift+I`
- 切换到 Console 标签页

### Step 3: 导航到 Online 模式
1. 主菜单显示 5 个选项
2. 按 `→` (右箭头) 3次，选中 "Online Multiplayer" (紫色卡片)
3. 按 `Enter` 或 `Space` 确认

### Step 4: 输入昵称
1. 应该看到昵称输入界面 (白色背景)
2. 标题: "Online Multiplayer"
3. 输入框: "Enter your nickname: _"
4. 输入昵称: 例如 "Player1"
5. 按 `Enter` 确认

---

## ✅ 成功标志

### 不应该出现的问题 ❌
- ❌ 黑屏
- ❌ 控制台报错: `RuntimeError: unreachable`
- ❌ 控制台报错: `panic_already_borrowed`
- ❌ 浏览器卡死/无响应

### 应该看到的现象 ✅
1. **控制台输出**:
   ```
   [WS] Connecting to ws://localhost:9001
   [WS] WebSocket opened
   [WS] Connected!
   [WS] Received: {"Connected":{"player_id":"xxx"}}
   ```

2. **游戏界面**:
   - 显示 "Waiting for opponent..." (白色文字)
   - 左上角显示: "Players: 1/2"
   - 背景是深蓝色星空

3. **服务器日志** (server/server.log):
   ```
   新连接: <IP>
   玩家连接: ID = xxx
   ```

---

## 🐛 如果仍然失败

### 检查点 1: RefCell panic
```
错误信息: "already borrowed: BorrowMutError"
位置: macroquad input context

→ 可能原因: 还有其他地方多次调用输入函数
→ 解决方案: 搜索 `is_key_pressed`, `mouse_wheel`, `get_char_pressed`
```

### 检查点 2: WebSocket 连接失败
```
控制台: WebSocket connection failed
服务器日志: 无新连接

→ 可能原因: 
  1. WebSocket 服务器未运行 (ps -p 90861)
  2. 端口被防火墙阻止
  3. CORS 问题

→ 解决方案:
  1. 重启服务器: cd server && cargo run
  2. 检查防火墙: sudo ufw status
```

### 检查点 3: WASM 加载失败
```
控制台: Failed to fetch *.wasm

→ 可能原因: 
  1. HTTP 服务器未运行
  2. WASM 文件路径错误
  3. 浏览器缓存

→ 解决方案:
  1. 重启 HTTP: python3 -m http.server 8000
  2. 强制刷新: Ctrl+Shift+R
  3. 清除缓存: F12 → Network → Disable cache
```

---

## 📊 当前代码修改

### 修改1: 移除全局输入检测 (main.rs:371-376)
```rust
// 删除前:
let esc_pressed = is_key_pressed(KeyCode::Escape);
let pause_pressed = esc_pressed || is_key_pressed(KeyCode::P);

// 删除后:
// (无全局输入检测)
```

### 修改2: Playing 状态局部检测 (main.rs:834-842)
```rust
GameState::Playing => {
    // 在状态内部检测，避免 RefCell 冲突
    let pause_pressed = is_key_pressed(KeyCode::Escape) 
                     || is_key_pressed(KeyCode::P);
    if pause_pressed {
        state = GameState::Paused { ... };
        continue;
    }
}
```

### 修改3: Paused 状态局部检测 (main.rs:902)
```rust
// 修改前:
if esc_pressed { ... }  // ❌ 使用全局变量

// 修改后:
if is_key_pressed(KeyCode::Escape) { ... }  // ✅ 局部检测
```

---

## 🎯 预期结果

**如果修复成功**:
- ✅ 可以顺利进入 Online Lobby
- ✅ 可以输入昵称
- ✅ 可以连接到 WebSocket 服务器
- ✅ 可以看到等待对手界面
- ✅ 控制台无 panic 错误

**下一步** (等待对手加入):
- 打开第二个浏览器标签页
- 重复测试步骤
- 两个玩家应该同时进入游戏

---

## 📎 相关文件

- 主逻辑: `src/main.rs` (行360-920)
- 网络层: `src/network.rs`
- UI: `src/ui.rs` (`draw_online_lobby`, `draw_online_waiting`)
- 服务器: `server/src/main.rs`
- WebSocket 客户端: `web/websocket_client.js`

---

**测试时间**: 2025-11-20 00:00
**编译版本**: asteroids_opencode.wasm (711K, Nov 19 23:59)
**服务器 PID**: 90861 (运行时长 50 分钟)
