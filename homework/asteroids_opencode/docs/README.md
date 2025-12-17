# Asteroids OpenCode - 开发文档

这是 Asteroids OpenCode 项目的官方开发文档。

## 快速开始

### 环境要求
- Rust 1.70+
- Cargo

### 构建项目
```bash
cargo build --release
```

### 运行项目
```bash
cargo run --release
```

## 架构概述

### 模块结构
- `main.rs` - 游戏主循环和状态管理
- `player.rs` - 玩家逻辑和控制
- `asteroid.rs` - 小行星生成和行为
- `network.rs` - 在线多人功能
- `performance.rs` - 性能监控

### 游戏模式
- **Survival**: 经典生存模式
- **Duel**: 对战模式
- **TimeAttack**: 竞速模式
- **Online**: 在线多人模式

## 开发指南

### 代码规范
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 所有警告必须修复

### 测试
```bash
cargo test
```

### 性能监控
按 F3 键切换性能监控覆盖层。

### 架构决策
查看 [ADRs](./adr/) 了解重要架构决策。

### API文档
- [Rust API文档](https://your-repo.github.io/asteroids_opencode/api/)
- [性能分析指南](./perf.md)

## API 参考

### 主要结构体

#### Player
```rust
pub struct Player {
    pub ship: Ship,
    pub controls: Controls,
    pub lives: u32,
    pub score: Score,
    // ... 其他字段
}
```

#### GameState
```rust
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
    // ... 其他状态
}
```

## 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 发起 Pull Request

## 许可证

本项目采用 MIT 许可证。