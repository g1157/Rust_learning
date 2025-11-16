# 太空射击游戏 (Space Shooter)

一个使用 Rust 和 macroquad 引擎开发的经典太空射击游戏。

## 🎮 游戏特性

- **经典太空射击玩法**: 控制飞船消灭从上方降落的敌人
- **三种敌人类型**: 小型、中型、大型敌人，不同速度和分数
- **生命值系统**: 玩家拥有3条生命，可以承受多次碰撞
- **无敌时间机制**: 受伤后获得2秒无敌保护，玩家飞船闪烁提示
- **动态难度系统**: 随分数提升难度（每100分提升1级），敌人生成更频繁、速度更快
- **完整UI显示**: 实时显示分数、生命值、难度等级
- **精美像素艺术**: 复古风格的游戏画面
- **动态星空背景**: 使用 GLSL 着色器实现的滚动星空效果
- **粒子效果系统**: 爆炸特效和视觉反馈
- **音效系统**: 背景音乐、激光射击和爆炸音效
- **高分记录**: 自动保存和显示最高分

## 📋 游戏说明

### 控制方式
- **方向键 ←/→**: 左右移动
- **方向键 ↑/↓**: 上下移动
- **空格键**: 发射激光
- **ESC 键**: 暂停游戏
- **暂停时按空格**: 继续游戏
- **游戏结束后按空格**: 返回主菜单

### 游戏规则
1. 消灭从屏幕上方降落的敌人获得分数
2. 敌人类型：
   - **小型敌人** (16x16): 速度快，分数 10
   - **中型敌人** (32x32): 速度中等，分数 20
   - **大型敌人** (48x48): 速度慢，分数 30
3. 玩家拥有 **3条生命**，与敌人碰撞损失1条
4. 受伤后获得 **2秒无敌时间**（飞船闪烁提示）
5. 每获得 **100分** 提升1级难度（最高10级）
6. 难度提升后：敌人生成更频繁、移动更快
7. 生命耗尽时游戏结束
8. 尽可能获取更高的分数并打破记录

## 🚀 运行游戏

### 前置要求
- Rust 工具链 (1.70+)
- macroquad 依赖的系统库

### 构建和运行

```bash
# 克隆仓库（如果还没有）
git clone <repository-url>
cd homework/hk7-2

# 开发模式运行
cargo run

# 发布模式运行（更好的性能）
cargo run --release

# 构建可执行文件
cargo build --release
# 可执行文件位于 target/release/hk7-2
```

### Linux 依赖
在 Linux 上运行可能需要安装以下库：
```bash
# Ubuntu/Debian
sudo apt install libasound2-dev libx11-dev libxcursor-dev libxi-dev libgl1-mesa-dev

# Fedora
sudo dnf install alsa-lib-devel libX11-devel libXcursor-devel libXi-devel mesa-libGL-devel
```

## 🎨 资源来源

所有资源均来自开源社区，使用 CC0 公共领域许可证：

### 游戏精灵
- **太空飞船像素艺术资源**
  - 作者: ansimuz
  - 许可证: CC0 Public Domain
  - 来源: https://opengameart.org/content/space-ship-shooter-pixel-art-assets

### 背景音乐
- **8-bit 太空射击音乐**
  - 作者: HydroGene
  - 许可证: CC0 Public Domain
  - 来源: https://opengameart.org/content/8-bit-epic-space-shooter-music

### 音效
- **科幻音效**
  - 作者: Kenney.nl
  - 许可证: CC0 Public Domain
  - 来源: https://opengameart.org/content/sci-fi-sounds

### UI 界面
- **科幻用户界面元素**
  - 作者: Buch
  - 许可证: CC0 Public Domain
  - 来源: https://opengameart.org/content/sci-fi-user-interface-elements

### 字体
- **AtariGames 字体**
  - 作者: Kieran
  - 许可证: Public Domain
  - 来源: https://nimblebeastscollective.itch.io/nb-pixel-font-bundle

### 星空着色器
- **星空教程**
  - 作者: Martijn Steinrucken (BigWings)
  - 许可证: CC BY-NC-SA 3.0
  - 来源: https://www.youtube.com/watch?v=rvDo9LvfoVE

## 📁 项目结构

```
hk7-2/
├── src/
│   ├── main.rs                 # 游戏主循环
│   └── starfield-shader.glsl   # 星空背景着色器
├── assets/                     # 游戏资源
│   ├── ship.png               # 玩家飞船
│   ├── enemy-small.png        # 小型敌人
│   ├── enemy-medium.png       # 中型敌人
│   ├── enemy-big.png          # 大型敌人
│   ├── laser-bolts.png        # 激光子弹
│   ├── explosion.png          # 爆炸效果
│   ├── *.wav                  # 音效文件
│   ├── *.ogg                  # 背景音乐
│   └── README.md              # 资源来源说明
├── Cargo.toml                 # 项目配置
└── README.md                  # 本文件
```

## 🛠️ 技术栈

- **语言**: Rust 2021 Edition
- **游戏引擎**: [macroquad](https://macroquad.rs/) 0.4
- **粒子系统**: macroquad-particles 0.2.2
- **音频**: macroquad 内置音频系统
- **图形**: OpenGL (通过 miniquad)

## 🔧 开发

### 代码检查
```bash
# 运行 Clippy 进行代码检查
cargo clippy

# 格式化代码
cargo fmt

# 运行测试
cargo test
```

### 调试模式
开发时使用 `cargo run` 会启用调试信息和更快的编译速度。

### 性能优化
使用 `cargo run --release` 可以获得最佳性能（约 3-10 倍提升）。

## ✅ 已实现功能

- [x] 添加更多敌人类型（中型、大型敌人）
- [x] 实现生命值系统（玩家3条命）
- [x] 添加难度递增系统
- [x] 完整UI显示（分数、生命、难度）
- [x] 无敌时间和视觉反馈

## 📝 TODO / 未来改进

- [ ] 实现道具系统（生命恢复、武器升级）
- [ ] 改进暂停菜单 UI
- [ ] 添加游戏配置文件
- [ ] Boss 战斗
- [ ] 多人模式
- [ ] 成就系统

## 📄 许可证

本项目代码使用 MIT 许可证。
游戏资源使用 CC0 公共领域许可证（详见 assets/README.md）。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

**享受游戏！** 🚀✨
