# 🚀 快速开始指南

## 🎮 桌面版运行

```bash
# 直接运行（调试模式）
cargo run

# 发布模式（更流畅）
cargo run --release
```

## 🌐 Web 版测试

### 快速启动
```bash
cd /home/blue/Rust_learning/homework/asteroids_opencode
./diagnose_wasm.sh
```

或者简单版本：
```bash
cd web
python3 -m http.server 8000
```

然后访问: **http://localhost:8000**

### 测试 LocalStorage 持久化
1. 打开浏览器开发者工具 (F12)
2. Application → Local Storage → http://localhost:8000
3. 游玩游戏，解锁成就
4. 刷新页面 (F5)
5. 验证成就保留 ✅

详细测试步骤见: **LOCALSTORAGE_TEST.md**

## 📋 发布/展示前 5 分钟检查清单

### 1️⃣ 代码质量（1分钟）
```bash
cargo fmt           # 格式化代码
cargo clippy        # 检查代码质量
cargo test          # 运行测试（36个）
```

### 2️⃣ 音频检查（1分钟）
```bash
# 确认音频文件存在
ls assets/sounds/*.wav

# 如果缺失，运行淡出脚本
python3 fix_shoot_fadeout.py
```

### 3️⃣ 试运行（2分钟）
```bash
cargo run --release

# 检查:
# ✅ 游戏正常启动
# ✅ 音效正常播放
# ✅ 进入设置，调整音量到 5-10%（演示用）
```

### 4️⃣ Git 提交（1分钟）
```bash
git add -A
git commit -m "feat: 音量系统优化，默认1%"
git push
```

## 🎬 现场演示步骤

### 开场（30秒）
```
"这是一个用 Rust 开发的小行星游戏
主要特点是完善的音频系统和性能优化"
```

### 生存模式演示（3分钟）
1. 选择 Survival Mode
2. 展示基本操作（W/A/D移动，J射击）
3. 展示粒子效果（爆炸、推进器）
4. 拾取护盾道具
5. 清空一波小行星

### 对战模式演示（3分钟）
1. 返回主菜单，选择 Duel Mode
2. 展示夺旗机制
3. 展示击杀连击（快速打多个小行星）
4. 展示慢动作效果
5. 演示回合制系统

### 技术亮点（2分钟）
1. 打开 Settings
   - 展示音量调节
   - 展示速度调节等设置
   
2. 按 F3 显示性能面板
   - FPS（60）
   - 实体数量
   - QuadTree 深度

3. 讲解技术特点
   - 音频淡出处理（250ms）
   - QuadTree 优化（O(n log n)）
   - 36个单元测试

### 结尾（30秒）
```
"代码在 GitHub 上开源
欢迎查看 README.md 了解更多技术细节"
```

## 📦 给别人展示/分享

### 方式1: GitHub 链接（推荐）
```bash
# 推送到 GitHub
git push origin main

# 分享链接
https://github.com/你的用户名/asteroids_opencode
```

### 方式2: 本地演示包
```bash
# 创建演示包
mkdir asteroids_demo
cp -r . asteroids_demo/
cd asteroids_demo
cargo build --release

# 打包
cd ..
tar czf asteroids_demo.tar.gz asteroids_demo/ \
  --exclude='target' --exclude='.git'

# 分享 asteroids_demo.tar.gz
# 对方只需: 解压 -> cargo run --release
```

### 方式3: 预编译二进制
```bash
# 构建发布版本
cargo build --release

# 二进制文件位置
target/release/asteroids_opencode

# 复制到任意位置运行（需要 assets/ 目录）
```

## 🔧 常见问题快速解决

### 音频没声音
```bash
# 1. 检查音频文件
ls assets/sounds/*.wav

# 2. 重新处理音频
python3 fix_shoot_fadeout.py

# 3. 调高游戏音量
# 在游戏设置中调到 5-10%
```

### 编译失败
```bash
cargo clean
cargo build --release
```

### 字体显示异常
```bash
# Linux/macOS
./setup_fonts.sh

# 或使用默认字体（游戏设置中选择 Default）
```

## 📊 项目亮点速查

| 特性 | 说明 |
|------|------|
| **音频系统** | 250ms超长淡出，1%默认音量 |
| **性能优化** | QuadTree 碰撞检测（O(n log n)） |
| **代码质量** | 36测试100%通过，0 clippy警告 |
| **粒子效果** | 1000+并发粒子不卡顿 |
| **游戏性** | 双人对战，击杀连击，慢动作 |
| **可定制** | 9个游戏设置，实时调节 |

## 🎯 下次使用（几天后）

```bash
# 1. 进入项目目录
cd /home/blue/Rust_learning/homework/asteroids_opencode

# 2. 拉取最新代码（如果有远程仓库）
git pull

# 3. 重新构建
cargo build --release

# 4. 验证音频
ls assets/sounds/*.wav

# 5. 试运行
cargo run --release

# 6. 在设置中调整音量（演示用5-10%）
```

## 📚 详细文档

- **RELEASE_GUIDE.md** - 完整发布指南
- **README.md** / **README.zh-CN.md** - 项目说明
- **AUDIO_FADEOUT.md** - 音频技术文档
- **SHOOT_FADEOUT_FIX.md** - 音效修复详情

---

**准备时间**: 5分钟  
**演示时间**: 8-10分钟  
**技术栈**: Rust + Macroquad  
**代码质量**: 生产级

✅ 一切就绪，随时可以展示！
