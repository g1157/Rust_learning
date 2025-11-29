# hk8-1 - Hyperion 混沌自转模拟

模拟土卫七（Hyperion）的混沌自转，研究不同偏心率下的 Lyapunov 指数。

## 物理背景

Hyperion 是太阳系中已知具有混沌自转的天体。其自转受到与土星之间潮汐力矩的影响，在椭圆轨道上表现出对初始条件的极度敏感性。

## 功能特性

- Euler-Cromer 数值积分
- 多偏心率对比模拟
- Lyapunov 指数估计
- 混沌/非混沌轨道分类
- HTML 可视化输出

## 运动方程

角加速度公式 (4.24)：
```
dω/dt = -3GM/(r⁵I) × (x·sinθ - y·cosθ)(x·cosθ + y·sinθ)
```

## 运行

```bash
cargo run
```

## 输出文件

- `dtheta_*.html` - 角度差发散图
- `divergence_compare_*.html` - 对比图
- `lambda_vs_e.html` - Lyapunov 指数随偏心率变化
- `chaotic_group_*.html` - 混沌轨道组
- `nonchaotic_group_*.html` - 非混沌轨道组
