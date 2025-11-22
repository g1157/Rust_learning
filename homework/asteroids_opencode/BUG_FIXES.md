# 🐛 Bug 修复记录

## Bug #1: 主菜单无法选择 Online 选项

**症状**: 使用方向键导航时，会直接跳过 "Online Multiplayer" 选项

**原因**: 导航逻辑不完整，没有将 Online 加入循环顺序

**修复**:
- 文件: `src/main.rs` (行 626-641)
- 更新导航顺序：
  ```
  Survival → Duel → Online → Achievements → Settings
  ```

**提交**: src/main.rs (导航逻辑修复)

---

## Bug #2: 输入昵称后黑屏崩溃

**症状**: 
- 进入 Online 模式后输入昵称立即黑屏
- 控制台错误: `panic_already_borrowed`
- 完整错误信息:
  ```
  Uncaught RuntimeError: unreachable executed
  Location: miniquad-0.4.8/src/native/wasm.rs:34:35
  panic_already_borrowed at RefCell::borrow_mut
  ```

**原因**: 
`get_char_pressed()` 在 macroquad 内部与其他输入函数（`mouse_wheel()`, `is_key_pressed()` 等）共享同一个 `RefCell<InputContext>`。在同一帧内调用多个输入函数会导致重复借用 panic。

**技术细节**:
```rust
// 问题代码
if let Some(ch) = get_char_pressed() {  // 可变借用 InputContext
    // ...
}
// 同一帧内，其他地方也在借用
let (_, wheel_y) = mouse_wheel();  // 又一次借用 -> panic!
```

**修复方案**:
- 文件: `src/main.rs` (行 1013-1060)
- 移除 `get_char_pressed()` 调用
- 改用 `is_key_pressed()` 逐个检测按键:
  - A-Z: 26 个字母键
  - 0-9: 10 个数字键
  - Minus: 下划线
  - Backspace: 删除
  - Enter: 确认

**代码示例**:
```rust
// 修复后的代码
for key in [KeyCode::A, KeyCode::B, /* ... */] {
    if is_key_pressed(key) && online_nickname.len() < 16 {
        let ch = format!("{:?}", key).chars().next().unwrap();
        online_nickname.push(ch);
        break;
    }
}
```

**提交**: src/main.rs (移除 get_char_pressed，使用 is_key_pressed)

---

## Bug #3: 主菜单缺少 Online 选项

**症状**: 主菜单只显示 4 个选项（Survival, Duel, Achievements, Settings），没有 Online

**原因**: `draw_mode_selection()` 函数没有绘制 Online 卡片

**修复**:
- 文件: `src/ui.rs` (行 363-435)
- 添加第 5 个卡片: "Online Multiplayer"
- 调整布局计算（从 4 个改为 5 个）
- 颜色: 紫色 (RGB: 0.6, 0.3, 0.9)
- 位置: Duel 和 Achievements 之间

**提交**: src/ui.rs (添加 Online Multiplayer 卡片)

---

## 修复验证

### 测试步骤
1. 编译: `cargo build --target wasm32-unknown-unknown --release`
2. 构建: `./build_web.sh`
3. 访问: http://localhost:8000
4. 强制刷新: Ctrl+Shift+R

### 预期结果
✓ 主菜单显示 5 个选项，包括 Online Multiplayer
✓ 可以使用方向键导航到 Online
✓ 输入昵称不再黑屏
✓ 成功连接到 WebSocket 服务器
✓ 显示 "Welcome, [昵称]!" 界面

---

## 相关文件

### 修改的文件
- `src/main.rs` - 导航逻辑 + 输入处理
- `src/ui.rs` - 主菜单 UI

### 受影响的功能
- 主菜单导航
- 在线模式昵称输入
- WebSocket 连接流程

---

## 技术笔记

### macroquad 输入系统注意事项

**危险的函数** (可能导致 RefCell panic):
- `get_char_pressed()` - 不要与其他输入函数混用
- 原因: 内部使用 `RefCell<InputContext>`

**安全的函数**:
- `is_key_pressed(KeyCode)` - 单次调用安全
- `is_key_down(KeyCode)` - 单次调用安全
- `mouse_wheel()` - 独立调用安全

**最佳实践**:
1. 在同一帧内只调用一种输入函数类型
2. 或者在调用 `get_char_pressed()` 前确保没有其他输入调用
3. 优先使用 `is_key_pressed()` 而不是 `get_char_pressed()`

---

**修复完成日期**: 2025-11-19  
**测试状态**: ✅ 通过
