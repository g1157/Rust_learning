# hk6 - 混沌摆模拟

研究受驱动阻尼摆的混沌行为。

## 功能特性

- 三种运行模式：delta、attractor、bifurcation
- 相空间轨迹和庞加莱截面可视化
- 分岔图生成

## 物理模型

受驱动阻尼摆方程：

```
d²θ/dt² = -γ(dθ/dt) - (g/L)sin(θ) + F·cos(Ω·t)
```

参数：
- γ: 阻尼系数
- g/L: 重力参数
- F: 驱动力振幅
- Ω: 驱动频率

## 模块结构

```
hk6/src/
├── main.rs        # 主程序入口
├── pendulum.rs    # 摆动力学
├── attractor.rs   # 吸引子绘制
├── bifurcation.rs # 分岔图生成
└── delta_theta.rs # 角度差分析
```

## 运行

```bash
cargo run -- delta       # Δθ 分析
cargo run -- attractor   # 吸引子图
cargo run -- bifurcation # 分岔图
```

## 输出

生成 PNG 格式的分岔图和 CSV 数据文件。
