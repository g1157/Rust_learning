# 发布和展示指南

## 📦 发布前准备清单

### 1. 代码质量检查 ✅

```bash
# 确保所有检查通过
cargo fmt --check          # 代码格式检查
cargo clippy -- -D warnings # 代码质量检查（0警告）
cargo test                 # 运行所有测试（36/36通过）
cargo build --release      # 发布版本构建
```

**当前状态**:
- ✅ 格式: 符合 rustfmt 标准
- ✅ Clippy: 0 警告
- ✅ 测试: 36/36 通过
- ✅ 编译: Debug + Release 通过

### 2. 音频资源准备 🔊

**重要**: 音频文件默认不包含在 git 仓库中（见 `.gitignore`）

#### 选项 A: 包含处理好的音频（推荐用于展示）

```bash
# 1. 修改 .gitignore，允许提交处理过的音频
echo "# 允许提交处理过的音频文件" >> .gitignore
echo "!assets/sounds/shoot.wav" >> .gitignore
echo "!assets/sounds/powerup.wav" >> .gitignore

# 2. 添加音频文件到 git
git add -f assets/sounds/shoot.wav
git add -f assets/sounds/powerup.wav
git add assets/sounds/README.md
```

#### 选项 B: 提供下载链接（推荐用于开源）

在 `assets/sounds/README.md` 中添加下载说明：

```markdown
## 获取音频文件

由于版权和文件大小原因，音频文件未包含在仓库中。

### 方法 1: 使用你自己的音频
将以下文件放入 `assets/sounds/` 目录：
- shoot.wav - 射击音效
- powerup.wav - 道具拾取音效

### 方法 2: 使用免费资源
推荐网站:
- OpenGameArt.org
- Freesound.org
- Zapsplat.com

### 音频处理
获得音频后，运行淡出处理脚本：
```bash
python3 fix_shoot_fadeout.py
```
```

### 3. 字体资源准备 🔤

字体文件同样默认不包含在 git 中。

#### 选项 A: 系统字体（推荐）

```bash
# 运行字体设置脚本（Linux/macOS）
./setup_fonts.sh
```

#### 选项 B: 包含字体文件

```bash
# 如果你有字体文件的使用权限
git add -f assets/fonts/DejaVuSans.ttf
git add -f assets/fonts/Ubuntu-R.ttf
```

### 4. 文档完整性检查 📚

确保以下文档存在且最新：

```bash
# 核心文档
✅ README.md              # 英文项目说明
✅ README.zh-CN.md        # 中文项目说明
✅ AGENTS.md              # 开发规范

# 音频文档
✅ AUDIO_FADEOUT.md       # 音频淡出指南
✅ SHOOT_FADEOUT_FIX.md   # 射击音效修复
✅ assets/sounds/README.md # 音频资源说明

# 设置指南
✅ SETUP_ASSETS.md        # 资源设置指南
✅ FONT_SYSTEM.md         # 字体系统说明

# 发布指南
✅ RELEASE_GUIDE.md       # 本文件
```

## 🚀 发布流程

### 方案 A: GitHub 发布（推荐）

#### 1. 提交所有更改

```bash
# 添加所有修改的文件
git add -A

# 提交（使用有意义的提交信息）
git commit -m "feat: 完善音频系统，默认音量调整为1%

- 音量默认值从3%降至1%，提升新手体验
- 更新所有文档（README、音频指南等）
- 射击音效250ms超长淡出，完全消除杂音
- 36个测试全部通过，0 clippy警告"

# 推送到远程仓库
git push origin main
```

#### 2. 创建 GitHub Release

```bash
# 方法1: 使用 GitHub 网页
# 1. 访问 https://github.com/你的用户名/asteroids_opencode
# 2. 点击 "Releases" -> "Create a new release"
# 3. 填写版本号和说明（见下方模板）

# 方法2: 使用 gh 命令行
gh release create v1.0.0 \
  --title "Asteroids v1.0.0 - 完整音频系统" \
  --notes "$(cat << 'NOTES'
## 🎮 Asteroids 小行星游戏 v1.0.0

### ✨ 主要特性

- 🎯 双人本地多人游戏（生存模式 + 对战模式）
- 🔊 完善的音频系统（1%默认音量，250ms淡出处理）
- ⚡ QuadTree 空间分区优化（O(n log n)碰撞检测）
- 💫 粒子效果系统（爆炸、推进器、尾迹）
- 🎨 可调游戏设置（速度、音量、字体等）
- 📊 性能监控面板（F3切换）

### 📦 安装和运行

#### 前置要求
- Rust 1.70+
- Cargo

#### 快速开始
\`\`\`bash
# 克隆仓库
git clone https://github.com/你的用户名/asteroids_opencode
cd asteroids_opencode

# 安装依赖并运行
cargo run --release
\`\`\`

### 🔊 音频设置

**重要**: 首次运行需要设置音频文件。

详见 [SETUP_ASSETS.md](SETUP_ASSETS.md)

### 📚 文档

- [README.md](README.md) - 英文说明
- [README.zh-CN.md](README.zh-CN.md) - 中文说明
- [AUDIO_FADEOUT.md](AUDIO_FADEOUT.md) - 音频系统详解

### ✅ 测试状态

- 单元测试: 36/36 通过
- Clippy: 0 警告
- 代码行数: ~3,200 行

### 🙏 致谢

感谢 Macroquad 团队和 Rust 社区！
NOTES
)"
```

#### 3. Release 说明模板

```markdown
## 🎮 Asteroids v1.0.0

### 新功能
- ✅ 完整的音频系统（1%默认音量）
- ✅ 专业音频淡出处理（250ms/150ms）
- ✅ 可调游戏设置（9个选项）
- ✅ QuadTree 碰撞优化
- ✅ 粒子效果系统
- ✅ 性能监控面板

### 游戏模式
- 生存模式：合作清除小行星波次
- 对战模式：多回合夺旗战（Best of 3/5）

### 下载和运行

**方法1: 从源码构建（推荐）**
```bash
git clone https://github.com/你的用户名/asteroids_opencode
cd asteroids_opencode
cargo run --release
```

**方法2: 下载预编译二进制（待补充）**

### 首次运行设置

1. 安装音频文件（详见 SETUP_ASSETS.md）
2. 运行 `cargo run --release`
3. 在设置中调整音量（默认1%）

### 已知问题

- 音频文件需要手动准备（版权原因未包含）
- 字体使用系统字体（需运行 setup_fonts.sh）

### 技术栈

- Rust 2024 Edition
- Macroquad 0.4
- 36个单元测试，100%通过
```

### 方案 B: 本地演示准备

#### 1. 创建演示包

```bash
# 创建演示目录
mkdir -p asteroids_demo
cd asteroids_demo

# 复制项目文件（排除 target 和 .git）
rsync -av --exclude 'target' --exclude '.git' \
  /home/blue/Rust_learning/homework/asteroids_opencode/ .

# 构建发布版本
cargo build --release

# 复制二进制文件到根目录（方便运行）
cp target/release/asteroids_opencode ./asteroids

# 创建启动脚本
cat > run.sh << 'SCRIPT'
#!/bin/bash
echo "🎮 Asteroids 小行星游戏"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "检查音频文件..."

if [ ! -f "assets/sounds/shoot.wav" ]; then
    echo "⚠️  警告: 未找到音频文件"
    echo "请先运行: python3 fix_shoot_fadeout.py"
    echo ""
fi

echo "启动游戏..."
./asteroids
SCRIPT

chmod +x run.sh

# 创建 README
cat > DEMO_README.md << 'README'
# Asteroids 演示包

## 快速开始

### 1. 准备音频（首次运行）
```bash
python3 fix_shoot_fadeout.py
```

### 2. 运行游戏
```bash
./run.sh
# 或直接运行
./asteroids
```

## 游戏操作

### 玩家1: W/A/D 移动，J/F 射击
### 玩家2: ↑/←/→ 移动，1 射击

### 设置: 主菜单 -> Settings
- 调整音量（默认1%）
- 调整速度倍数
- 开关震动/慢动作等

## 完整文档

见 README.md 和 README.zh-CN.md
README

echo "✅ 演示包准备完成！"
echo "位置: $(pwd)"
```

#### 2. 打包压缩

```bash
# 创建压缩包（包含所有必要文件）
cd ..
tar czf asteroids_demo_v1.0.0.tar.gz asteroids_demo/ \
  --exclude='target' \
  --exclude='.git'

# 或使用 zip（Windows友好）
zip -r asteroids_demo_v1.0.0.zip asteroids_demo/ \
  -x "*/target/*" "*/.git/*"

echo "✅ 演示包: asteroids_demo_v1.0.0.tar.gz"
```

## 🎬 现场演示准备

### 1. 演示检查清单

**运行前检查**:
```bash
# ✅ 音频文件存在
ls assets/sounds/*.wav

# ✅ 编译成功
cargo build --release

# ✅ 测试通过
cargo test --quiet

# ✅ 音量合适（演示时可能需要调高）
# 在游戏设置中调整到 5%-10%
```

### 2. 演示流程建议

**第一部分: 生存模式（3-5分钟）**
1. 启动游戏，选择 Survival Mode
2. 展示基本操作（移动、射击）
3. 展示粒子效果（爆炸、推进器）
4. 展示护盾道具拾取
5. 展示波次系统（清空所有小行星）

**第二部分: 对战模式（3-5分钟）**
1. 返回主菜单，选择 Duel Mode
2. 展示夺旗机制
3. 展示击杀连击系统
4. 展示慢动作效果
5. 展示回合制系统

**第三部分: 设置系统（2分钟）**
1. 打开 Settings
2. 展示音量调节
3. 展示速度调节
4. 展示其他开关选项
5. 按 F3 展示性能面板

### 3. 演示话术参考

```
"这是一个用 Rust 和 Macroquad 开发的小行星游戏。

主要亮点：

1. 音频系统：
   - 使用专业淡出处理，完全消除杂音
   - 射击音效用了 250ms 超长淡出，占音效时长的 66%
   - 默认音量1%，可以在设置中调节

2. 性能优化：
   - QuadTree 空间分区，碰撞检测从 O(n²) 优化到 O(n log n)
   - 支持 1000+ 粒子特效不卡顿
   - 按 F3 可以看实时性能数据

3. 代码质量：
   - 36 个单元测试，100% 通过
   - Clippy 0 警告
   - 完整的文档系统

4. 游戏性：
   - 双人本地合作/对战
   - 多种武器类型（普通/散射/穿透）
   - 击杀连击系统，慢动作效果
   - 完全可定制的游戏设置
"
```

## 📝 几天后展示准备

### 快速恢复检查清单

```bash
# 1. 拉取最新代码
git pull origin main

# 2. 重新构建
cargo build --release

# 3. 验证音频
ls assets/sounds/*.wav
python3 fix_shoot_fadeout.py  # 如果音频丢失

# 4. 运行测试
cargo test

# 5. 试运行
cargo run --release

# 6. 调整音量（演示用）
# 在游戏设置中调到 5-10%
```

### 备份重要文件

```bash
# 备份音频文件（以防丢失）
cp -r assets/sounds /tmp/asteroids_sounds_backup

# 备份发布版本二进制
cp target/release/asteroids_opencode ./asteroids_v1.0.0_backup
```

## 🌐 在线展示（可选）

### 选项: 部署到 itch.io

Macroquad 支持编译到 WASM，可以部署到网页：

```bash
# 安装 WASM 目标
rustup target add wasm32-unknown-unknown

# 构建 WASM 版本
cargo build --release --target wasm32-unknown-unknown

# 生成网页包装
# （需要额外配置，见 Macroquad 文档）
```

## ⚠️ 常见问题

### 问题1: 音频文件丢失

```bash
# 从备份恢复
cp -r /tmp/asteroids_sounds_backup/* assets/sounds/

# 或重新处理
python3 fix_shoot_fadeout.py
```

### 问题2: 编译失败

```bash
# 清理并重新构建
cargo clean
cargo build --release
```

### 问题3: 音效有杂音

```bash
# 重新运行淡出脚本
python3 fix_shoot_fadeout.py

# 检查音量设置（应该是1%）
# 在游戏设置中确认
```

### 问题4: 字体缺失

```bash
# Linux/macOS
./setup_fonts.sh

# 或手动复制系统字体
cp /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf assets/fonts/
```

## 📧 分享给他人

### GitHub 链接

```
项目地址: https://github.com/你的用户名/asteroids_opencode
演示视频: [待录制]
文档: 见 README.md
```

### 本地分享

```bash
# 打包整个项目（排除构建产物）
tar czf asteroids_share.tar.gz \
  asteroids_opencode/ \
  --exclude='target' \
  --exclude='.git'

# 分享给朋友
# 他们只需：
# 1. 解压
# 2. cargo run --release
# 3. 享受游戏！
```

## ✅ 最终检查清单

发布前确认：

- [ ] 所有测试通过（cargo test）
- [ ] Clippy 无警告（cargo clippy）
- [ ] 代码已格式化（cargo fmt）
- [ ] 音频文件已处理（fix_shoot_fadeout.py）
- [ ] README 文档完整
- [ ] 提交信息清晰
- [ ] Release notes 准备好
- [ ] 演示流程熟悉
- [ ] 音量已调整（演示用）
- [ ] 备份已创建

准备就绪！🚀
