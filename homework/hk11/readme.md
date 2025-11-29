# 作业十一：分形图形与扩散模拟

## 任务要求

**作业一：几何/递归类分形图形**
- 自由绘制分形图形，越复杂的越好
- 几何/递归类分形（线、三角形、树等）——非"复平面像素迭代"方法的分形

**作业二：奶滴扩散模拟**
- 进行奶滴扩散的随机漫步模拟
- 50×50 容器格子，边缘长度为 10 个单位的孔
- 验证粒子数随时间符合 exp(-t/τ) 变化规律

---

## 作业一完成情况 ✅

### 实现的分形

| 分形 | 算法类型 | 特点 |
|------|----------|------|
| **分形树 (Fractal Tree)** | 递归 | 二叉分支，棕→绿渐变着色 |
| **Barnsley 蕨类 (Barnsley Fern)** | IFS 迭代 | 模拟真实蕨类植物，自相似结构 |

### 生成结果

| 分形树 | Barnsley 蕨类 |
|--------|---------------|
| ![分形树](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/fractal_tree.png) | ![Barnsley 蕨类](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/barnsley_fern.png) |

### 算法说明

#### 1. 分形树 (Fractal Tree)

**原理**：递归二叉分支

```
           起点
            │
            │ length
            │
           终点
          ╱    ╲
      左分支   右分支
     angle-30° angle+30°
     长度×0.7  长度×0.7
```

**参数**：
- 最大深度：12 层（2^12 = 4096 个末端分支）
- 分支角度：30°
- 长度缩放：0.7 倍

**着色**：按深度渐变，树干棕色 (139,69,19) → 树叶绿色 (34,139,34)

#### 2. Barnsley 蕨类 (Barnsley Fern)

**原理**：迭代函数系统 (IFS)

使用 4 个仿射变换，按概率随机选择：

| 变换 | 概率 | 作用 |
|------|------|------|
| f1 | 1% | 茎干 |
| f2 | 85% | 主叶片（自相似缩放+旋转）|
| f3 | 7% | 左侧小叶 |
| f4 | 7% | 右侧小叶 |

**仿射变换公式**：
```
x' = a*x + b*y + e
y' = c*x + d*y + f
```

**迭代次数**：500,000 次

---

## 作业二完成情况 ✅

### 模拟参数

- **容器尺寸**：50×50 格子
- **孔洞位置**：右边墙，y∈[20,30)，长度 10 格
- **初始粒子数**：200
- **初始分布**：中心 10×10 区域
- **模拟时长**：10^6 时间步

### 模拟结果

| 时刻 t | 粒子数 N | ln(N/N₀) |
|--------|----------|----------|
| 0 | 200 | 0.00 |
| 10^4 | 50 | -1.39 |
| 2×10^4 | 15 | -2.59 |
| 3×10^4 | 5 | -3.69 |
| 4×10^4 | 1 | -5.30 |
| 10^5 | 0 | - |

### 验证 exp(-t/τ) 规律

根据最小二乘法拟合结果：
- **斜率 k = -0.000131**
- **特征时间 τ = -1/k ≈ 7,630 时间步**
- **半衰期 t₁/₂ = τ·ln(2) ≈ 5,287 时间步**

ln(N/N₀) 与 t 呈线性关系，斜率 ≈ -1/τ，**验证了粒子数符合指数衰减规律**：

$$N(t) = N_0 \cdot e^{-t/\tau}$$

### ln(N/N₀) vs t 拟合图

![ln(N/N₀) vs t](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/diffusion_lnN_vs_t.png)

- **红色圆点**：实验数据点
- **蓝色直线**：理论拟合线 ln(N/N₀) = -t/τ

### 扩散过程可视化

| t=0 (初始) | t=10^4 |
|------------|--------|
| ![t=0](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/diffusion_t0.png) | ![t=1e4](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/diffusion_t1e4.png) |

| t=10^5 | t=10^6 |
|--------|--------|
| ![t=1e5](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/diffusion_t1e5.png) | ![t=1e6](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/diffusion_t1e6.png) |

---

## 项目结构

```
src/
  main.rs           # 主入口
  common.rs         # 公共模块（Point, Line, 绘图函数）
  fractal_tree.rs   # 分形树算法
  barnsley_fern.rs  # Barnsley 蕨类算法
  diffusion.rs      # 奶滴扩散模拟

输出文件：
  fractal_tree.png        # 分形树
  barnsley_fern.png       # Barnsley 蕨类
  diffusion_t0.png        # 扩散初始状态
  diffusion_t1e4.png      # t=10^4
  diffusion_t1e5.png      # t=10^5
  diffusion_t1e6.png      # t=10^6
  diffusion_lnN_vs_t.png  # ln(N/N₀) vs t 拟合图
```

## 运行方式

```bash
cargo run --release
```

## 代码设计亮点

### 模块化设计

`common.rs` 抽取公共组件，供所有模块复用：

```rust
// 几何结构
pub struct Point { x: f64, y: f64 }
pub struct Line { start: Point, end: Point }

// 绘图工具
pub fn draw_line(img, line, color)    // Bresenham 画线
pub fn draw_pixel(img, x, y, color)   // 单像素绘制
pub fn lerp_color(c1, c2, t)          // 颜色插值
pub fn create_image(w, h, bg)         // 创建图像
```

### 三种不同的算法范式

| 模块 | 算法类型 | 核心思想 |
|------|----------|----------|
| fractal_tree | 递归 | 分治：大问题分解为相似的小问题 |
| barnsley_fern | IFS 迭代 | 随机选择变换，收敛到吸引子 |
| diffusion | 蒙特卡洛 | 随机漫步模拟物理过程 |

---

## 依赖

```toml
[dependencies]
image = "0.25"
rand = "0.8"
```
