# 超越平方反比律的行星轨道模拟器

使用纯 Rust 标准库实现的广义引力定律轨道模拟器。

## 物理模型

本模拟器实现了广义引力定律：

```
F = -GMm / r^(2+β)
```

其中 β 是可调节参数：
- **β = 0.0**: 标准牛顿引力（平方反比定律）- 闭合椭圆轨道
- **β > 0**: 引力随距离衰减更快 - 轨道进动或螺旋坠入
- **β < 0**: 引力随距离衰减更慢 - 轨道更稳定或逃逸

## 数值方法

使用 **欧拉-克罗默方法**（Euler-Cromer）进行数值积分：
1. 先更新速度：`v_new = v + a * dt`
2. 再用新速度更新位置：`x_new = x + v_new * dt`

这种方法比标准欧拉法对轨道运动的能量守恒更好。

## 编译和运行

### 编译
```bash
cargo build --release
```

### 运行不同 β 值的模拟
```bash
# 标准牛顿引力（β = 0）
cargo run --release -- 0.0

# 广义相对论效应模拟（β = 0.1）
cargo run --release -- 0.1

# 强引力偏离（β = 0.5）
cargo run --release -- 0.5

# 弱引力（β = -0.5）
cargo run --release -- -0.5

# 极强引力（β = 1.0）
cargo run --release -- 1.0
```

## 输出文件

每次运行会生成两个文件：
1. **SVG 图像文件**: `orbit_beta_X_X.svg` - 可在浏览器中查看的矢量图
2. **CSV 数据文件**: `orbit_beta_X_X.csv` - 包含位置、速度、距离等数据

## 示例输出

```
=== Orbital Simulation with Generalized Gravity ===
Force law: F = -GMm / r^(2+β)
β = 0

Initial energy: -1.973921e-5
Running simulation for 3.00 years (30000 steps)...
Progress: 100.0% - Complete!
Final energy: -1.973921e-5
Energy drift: 0.0000%
```

还会在终端显示 ASCII 艺术轨道图。

## 项目特点

- ✅ **零依赖**: 仅使用 Rust 标准库
- ✅ **能量守恒**: 欧拉-克罗默方法保证良好的能量守恒
- ✅ **可视化**: SVG 矢量图 + ASCII 终端图
- ✅ **数据导出**: CSV 格式，便于进一步分析
- ✅ **高性能**: Release 模式编译，快速模拟

## 物理参数

- 时间步长: `dt = 0.0001` 年
- 模拟时长: 3 个轨道周期
- 初始位置: 1 AU（天文单位）
- 初始速度: 2π AU/年（接近圆轨道）
- 天文单位制: AU, year, solar mass

## 代码结构

```
src/main.rs
├── Vec2         - 二维向量结构体
├── Planet       - 行星结构体
├── Simulation   - 轨道模拟器
│   ├── gravity_acceleration()  - 计算引力加速度
│   ├── step()                  - 欧拉-克罗默单步积分
│   ├── run()                   - 运行模拟
│   ├── total_energy()          - 计算总能量
│   ├── save_svg()              - 生成 SVG 图像
│   ├── print_ascii_orbit()     - 终端 ASCII 图
│   └── save_csv()              - 保存 CSV 数据
└── main()       - 主程序入口
```

## 实验建议

尝试不同的 β 值观察轨道变化：
- β = 0.0: 标准椭圆轨道
- β = 0.1: 轻微进动（类似水星近日点进动）
- β = 0.5: 明显进动，玫瑰花瓣形轨道
- β = 1.0: 快速螺旋坠入
- β = -0.5: 轨道膨胀
