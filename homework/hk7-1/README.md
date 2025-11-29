# hk7-1 - 台球模拟器

二维台球系统的动力学模拟，研究不同边界形状下的混沌行为。

## 功能特性

- 多种边界形状：圆形、椭圆、圆角三角形、星形、肾形
- 轨迹可视化和相空间图
- 庞加莱截面分析

## 支持的边界形状

| 形状 | 参数 | 描述 |
|------|------|------|
| circle | - | 单位圆 |
| ellipse | a=1.3, b=0.8 | 椭圆 |
| triangle | k=0.35 | 圆角三角形 |
| star | 5瓣 | 五角星形 |
| bean | - | 肾形/心豆形 |

## 物理模型

- 理想弹性碰撞（镜面反射）
- 欧拉积分推进运动
- 二分法精确定位碰撞点

## 运行

```bash
cargo run -- circle    # 圆形边界
cargo run -- ellipse   # 椭圆边界
cargo run -- star      # 星形边界
```

## 输出

- `{shape}_trajectory.html` - 运动轨迹
- `{shape}_phase_space.html` - 相空间图
- `{shape}_attractor.html` - 庞加莱截面
