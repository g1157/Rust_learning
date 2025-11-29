# hk11 - 分形图形与扩散模拟

生成多种分形图形并模拟奶滴扩散过程。

## 作业一：分形图形

### 生成的图形

| 图形 | 算法 | 输出文件 |
|------|------|----------|
| 分形树 | 递归分支 | `fractal_tree.png` |
| Barnsley 蕨 | IFS 迭代 | `barnsley_fern.png` |
| 龙形曲线 | 折纸分形 | `dragon_curve.png` |
| 列维 C 曲线 | L-System | `levy_c_curve.png` |
| 分形植物 | L-System | `fractal_plants.png` |
| 分形树变体 | 随机/风吹效果 | `fractal_tree_variants.png` |

### 算法说明

- **递归分形**: 每个分支产生两个子分支，角度和长度按比例缩小
- **IFS (迭代函数系统)**: 随机选择仿射变换迭代绘制点
- **L-System**: 符号重写系统，解释为绘图指令

## 作业二：奶滴扩散模拟

使用随机漫步模拟粒子扩散：
- 粒子从中心释放
- 随机方向移动
- 统计扩散半径随时间变化

### 输出

- `diffusion_*.png` - 扩散过程快照
- `diffusion_lnN_vs_t.png` - 粒子数衰减拟合图

## 模块结构

```
hk11/src/
├── main.rs           # 主程序
├── common.rs         # 公共组件
├── fractal_tree.rs   # 分形树
├── barnsley_fern.rs  # Barnsley 蕨
├── dragon_curve.rs   # 龙形曲线
├── fractal_plants.rs # L-System 植物
└── diffusion.rs      # 扩散模拟
```

## 运行

```bash
cargo run --release
```
