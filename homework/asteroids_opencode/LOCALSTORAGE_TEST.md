# LocalStorage 持久化测试指南

## 🎯 测试目标
验证 Web 版本的成就和统计数据能够在浏览器中持久化保存。

## ✅ 测试步骤

### 1. 启动本地测试服务器
```bash
cd web
python3 -m http.server 8000
```

然后在浏览器访问: http://localhost:8000

### 2. 打开浏览器开发者工具
- Chrome/Edge: 按 F12 或 Ctrl+Shift+I (Windows/Linux) / Cmd+Option+I (Mac)
- 切换到 "Application" 标签页
- 左侧菜单: Storage → Local Storage → http://localhost:8000

### 3. 游玩游戏并解锁成就
- 启动游戏（按 ENTER）
- 尝试以下操作解锁成就：
  - 摧毁第一颗小行星 → 解锁 "First Blood"
  - 发射子弹 → 增加 "Armed" 进度
  - 拾取护盾 → 解锁 "Protected"
  - 存活一段时间 → 解锁 "Survivor"

### 4. 检查 LocalStorage 数据
在开发者工具的 Local Storage 中应该能看到：
- **Key**: `achievements`
- **Value**: JSON 格式的成就数据

示例数据结构：
```json
{
  "progress": {
    "FirstBlood": {
      "unlocked": true,
      "unlock_time": 123.456,
      "current": 1
    },
    "Armed": {
      "unlocked": false,
      "unlock_time": null,
      "current": 25
    }
  },
  "stats": {
    "total_playtime": 60.0,
    "total_kills": 15,
    "bullets_fired": 50,
    "shields_collected": 2,
    "games_played": 1,
    ...
  }
}
```

### 5. 测试持久化
1. **刷新页面** (F5 或 Ctrl+R)
2. 游戏重新加载后，查看成就界面
3. **验证**: 之前解锁的成就应该仍然显示为已解锁
4. **验证**: 统计数据应该保留（可以继续累积）

### 6. 测试跨会话持久化
1. 完全关闭浏览器标签页
2. 重新打开 http://localhost:8000
3. **验证**: 所有成就和统计数据应该完整保留

### 7. 测试重置功能
如果游戏有重置成就的选项：
1. 触发重置
2. 检查 LocalStorage - `achievements` key 的数据应该被重置
3. 刷新页面 - 验证重置后的状态被正确加载

## 🔍 调试技巧

### 查看控制台日志
在浏览器控制台 (Console 标签) 中查看：
- 成就保存/加载的错误信息
- WASM 模块加载状态

### 手动检查 LocalStorage
在控制台中运行以下 JavaScript 代码：
```javascript
// 读取成就数据
JSON.parse(localStorage.getItem('achievements'))

// 清除数据（测试首次运行）
localStorage.removeItem('achievements')

// 查看所有存储的 key
Object.keys(localStorage)
```

### 常见问题排查

**问题**: 刷新后成就丢失
- 检查是否有控制台错误
- 确认 LocalStorage 中存在 `achievements` key
- 检查 JSON 格式是否有效

**问题**: 无法保存数据
- 检查浏览器是否阻止了 LocalStorage (隐私模式)
- 确认 LocalStorage 配额没有超限 (通常 5-10 MB)
- 查看控制台是否有权限错误

**问题**: 数据不一致
- 清除 LocalStorage: `localStorage.clear()`
- 硬刷新页面: Ctrl+Shift+R (Windows/Linux) / Cmd+Shift+R (Mac)
- 检查是否有多个标签页同时运行游戏

## 🎮 测试案例示例

### 测试案例 1: 新手成就
1. 首次运行游戏（清空 LocalStorage）
2. 完成新手教程相关操作
3. 解锁以下成就：
   - First Flight
   - First Blood
   - Protected
4. 刷新页面，验证这 3 个成就仍然解锁

### 测试案例 2: 进度累积
1. 游玩游戏，发射 50 发子弹（但不到 100）
2. 刷新页面
3. 继续发射子弹
4. 验证总数能正确累积到 100，解锁 "Armed" 成就

### 测试案例 3: 跨会话
1. 解锁 5 个不同的成就
2. 关闭浏览器（不仅是标签页）
3. 第二天重新打开游戏
4. 验证所有成就和统计数据完整保留

## 📊 期望结果

✅ **成功标准**:
- 成就解锁后刷新页面仍然保持
- 统计数据能够正确累积
- 跨浏览器会话数据不丢失
- LocalStorage 中能看到正确的 JSON 数据
- 没有控制台错误

❌ **失败情况**:
- 刷新后成就全部重置
- 统计数据不累积或归零
- LocalStorage 中没有数据
- 控制台报错 "Failed to save/load"

## 🚀 下一步优化

如果基本功能正常，可以考虑：
1. **压缩存储**: 使用 MessagePack 或 CBOR 代替 JSON
2. **版本管理**: 添加数据版本号，支持迁移
3. **备份导出**: 允许用户导出/导入成就数据
4. **云同步**: 实现多设备同步（需要后端服务）
5. **存储配额检测**: 提前警告用户存储空间不足

## 📝 相关文件

- `src/storage.rs` - 跨平台存储接口
- `src/achievement.rs` - 成就系统
- `Cargo.toml` - web-sys 依赖配置
- `web/asteroids_opencode.wasm` - 包含 LocalStorage 支持的 WASM 二进制
