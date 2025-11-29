# Rust Learning

个人学习 Rust 语言的代码仓库，主要包含计算物理课程项目和 Rust 语法练习。

## 项目结构

```
Rust_learning/
├── homework/          # 计算物理课程作业
├── test/              # 实验性项目
├── yufa/              # Rust 语法学习
└── rustlings-zh-cn/   # Rustlings 中文版练习
```

---

## homework/ - 计算物理作业

物理模拟与数值计算项目，使用 Rust 实现各种物理系统的数值模拟。

| 项目 | 描述 | 关键技术 |
|------|------|----------|
| [guessing_game](homework/guessing_game) | 猜数字游戏 | 用户输入、随机数 |
| [hk3-1](homework/hk3-1) | Rectangle 结构体 | 结构体、方法 |
| [hk3-2](homework/hk3-2) | TrafficLight 借用 | 引用与借用 |
| [hk3-3](homework/hk3-3) | Self 类型 | 可变借用 |
| [hk3-4](homework/hk3-4) | 关联函数 | 构造器模式 |
| [hk3-5](homework/hk3-5) | 枚举与 match | 模式匹配 |
| [hk4](homework/hk4) | 放射性衰变模拟 | 蒙特卡洛方法 |
| [hk5](homework/hk5) | 炮弹轨迹模拟 | 欧拉法、空气阻力 |
| [hk6](homework/hk6) | 混沌摆模拟 | 分岔图、相空间 |
| [hk7-1](homework/hk7-1) | 台球模拟器 | 碰撞检测、HTML可视化 |
| [hk7-2](homework/hk7-2) | Space Shooter 游戏 | Macroquad、游戏开发 |
| [hk8-1](homework/hk8-1) | Hyperion 混沌自转 | 刚体动力学 |
| [hk10-1](homework/hk10-1) | 波动方程数值解 | 有限差分法 |
| [hk11](homework/hk11) | 分形与扩散 | IFS、Barnsley蕨 |
| [asteroids_opencode](homework/asteroids_opencode) | 小行星游戏 | 多人网络、WebAssembly |

---

## test/ - 实验性项目

各种物理模拟和游戏开发的实验项目。

### 入门项目

| 项目 | 描述 |
|------|------|
| [hello_cargo](test/hello_cargo) | Cargo 项目模板 |
| [hello_rust](test/hello_rust) | Hello World |
| [hello_world](test/hello_world) | 纯 rustc 编译 |

### 轨道力学模拟

| 项目 | 描述 | 物理模型 |
|------|------|----------|
| [jupiter-earth](test/jupiter-earth) | 木星-地球轨道 | Euler-Cromer 方法 |
| [mercury-precession](test/mercury-precession) | 水星进动 | 广义相对论修正 |
| [orbital-sim](test/orbital-sim) | 广义引力轨道 | 可变引力指数 β |
| [three-body-chaos](test/three-body-chaos) | 三体问题混沌 | Velocity Verlet |

### 其他项目

| 项目 | 描述 |
|------|------|
| [my-game](test/my-game) | 2D 射击游戏（Macroquad） |
| [shc](test/shc) | 薛定谔方程求解器 |

---

## yufa/ - Rust 语法学习

Rust 语言基础语法的学习笔记和代码示例。

| 项目 | 主题 | 内容 |
|------|------|------|
| [branches](yufa/branches) | 控制流 | if/else、loop、while、for |
| [data_type](yufa/data_type) | 数据类型 | 标量、元组、数组 |
| [fnc](yufa/fnc) | 函数 | 参数、返回值、表达式 |
| [hashmap](yufa/hashmap) | HashMap | 键值对集合 |
| [result](yufa/result) | 错误处理 | Result、unwrap、expect |
| [string](yufa/string) | 字符串 | String、&str、UTF-8 |
| [vector](yufa/vector) | Vector | 动态数组、遍历 |

---

## 快速开始

### 环境要求

- Rust 1.70+
- Cargo

### 运行项目

```bash
# 进入任意项目目录
cd homework/hk6

# 编译并运行
cargo run --release
```

### 常用命令

```bash
cargo build          # 编译
cargo run            # 运行
cargo test           # 测试
cargo clippy         # 代码检查
cargo fmt            # 格式化
```

---

## 学习资源

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)

---

## 许可证

本仓库仅供学习和研究使用。

---

> **Note**: 本仓库的 README 文档在 AI (Claude) 辅助下完成整理和编写。
