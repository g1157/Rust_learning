# 作业十一：分形图形与扩散模拟

> **课程**：计算物理  
> **学号**：[学号]  
> **姓名**：[姓名]  
> **日期**：2024年11月

---

## 一、实验目的

1. 掌握几何/递归类分形图形的生成算法，理解分形的自相似性质
2. 通过随机漫步模拟奶滴扩散过程，验证粒子数随时间的指数衰减规律 $N(t) = N_0 e^{-t/\tau}$
3. 学习使用 Rust 语言进行科学计算与图形可视化

---

## 二、实验原理

### 2.1 分形几何

分形（Fractal）是具有**自相似性**的几何图形，即局部与整体在某种意义上相似。常见的生成方法包括：

| 方法 | 原理 | 代表图形 |
|------|------|----------|
| **递归分解** | 将图形分解为相似的子图形 | 分形树、科赫雪花 |
| **迭代函数系统 (IFS)** | 随机选择仿射变换迭代 | Barnsley 蕨类 |
| **L-System** | 字符串重写 + 海龟绘图 | 分形植物 |
| **折纸规则** | 模拟纸张对折展开 | 龙形曲线 |

### 2.2 随机漫步与扩散

粒子在容器中做随机漫步，每步等概率向上/下/左/右移动一格。当粒子到达容器边缘的孔洞时逃逸。

设初始粒子数为 $N_0$，逃逸满足一阶动力学：

$$\frac{dN}{dt} = -\frac{N}{\tau}$$

解得：

$$N(t) = N_0 \cdot e^{-t/\tau}$$

其中 $\tau$ 为特征逃逸时间。取对数得：

$$\ln\frac{N}{N_0} = -\frac{t}{\tau}$$

因此 $\ln(N/N_0)$ 与 $t$ 应呈线性关系，斜率为 $-1/\tau$。

---

## 三、实验内容与结果

### 3.1 分形树 (Fractal Tree)

#### 算法原理

采用**递归二叉分支**策略：从树干开始，每个枝条末端分裂为两个子枝条，角度偏移 $\pm\theta$，长度缩放为原来的 $r$ 倍。

```
设当前枝条：起点 (x, y)，方向 angle，长度 L
终点：(x + L·cos(angle), y - L·sin(angle))
左子枝：方向 angle + θ，长度 L·r
右子枝：方向 angle - θ，长度 L·r
```

#### 核心代码

```rust
fn draw_branch(img: &mut RgbImage, x: f64, y: f64, angle: f64, 
               length: f64, depth: u32, max_depth: u32) {
    if depth == 0 || length < 1.0 {
        return;
    }
    
    // 计算终点
    let end_x = x + angle.cos() * length;
    let end_y = y - angle.sin() * length;
    
    // 颜色渐变：树干棕色 → 树叶绿色
    let t = depth as f64 / max_depth as f64;
    let color = lerp_color(LEAF_GREEN, TRUNK_BROWN, t);
    
    draw_line(img, &Line::new(Point::new(x, y), Point::new(end_x, end_y)), color);
    
    // 递归绘制左右分支
    let new_length = length * 0.7;
    draw_branch(img, end_x, end_y, angle + 0.5, new_length, depth - 1, max_depth);
    draw_branch(img, end_x, end_y, angle - 0.5, new_length, depth - 1, max_depth);
}
```

#### 参数设置

- 最大递归深度：12 层
- 分支角度：$\theta = 30°$
- 长度缩放因子：$r = 0.7$

#### 生成结果

![分形树](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/fractal_tree.png)

**分析**：分形树展现了明显的自相似结构，每个分支都是整体的缩小版。颜色从树干的棕色渐变到树叶的绿色，增强了视觉层次感。共有 $2^{12} = 4096$ 个末端分支。

---

### 3.2 Barnsley 蕨类 (Barnsley Fern)

#### 算法原理

采用**迭代函数系统 (IFS)**，使用 4 个仿射变换，按概率随机选择：

$$\begin{pmatrix} x' \\ y' \end{pmatrix} = \begin{pmatrix} a & b \\ c & d \end{pmatrix} \begin{pmatrix} x \\ y \end{pmatrix} + \begin{pmatrix} e \\ f \end{pmatrix}$$

| 变换 | 概率 | 作用 | 参数 |
|:----:|:----:|:----:|:----:|
| $f_1$ | 1% | 茎干 | $a=0, b=0, c=0, d=0.16$ |
| $f_2$ | 85% | 主叶片 | $a=0.85, b=0.04, c=-0.04, d=0.85$ |
| $f_3$ | 7% | 左侧小叶 | $a=0.2, b=-0.26, c=0.23, d=0.22$ |
| $f_4$ | 7% | 右侧小叶 | $a=-0.15, b=0.28, c=0.26, d=0.24$ |

#### 核心代码

```rust
fn barnsley_fern_iteration(x: f64, y: f64, rng: &mut impl Rng) -> (f64, f64) {
    let r: f64 = rng.r#gen();
    
    if r < 0.01 {
        // f1: 茎干
        (0.0, 0.16 * y)
    } else if r < 0.86 {
        // f2: 主叶片（自相似缩放+旋转）
        (0.85 * x + 0.04 * y, -0.04 * x + 0.85 * y + 1.6)
    } else if r < 0.93 {
        // f3: 左侧小叶
        (0.2 * x - 0.26 * y, 0.23 * x + 0.22 * y + 1.6)
    } else {
        // f4: 右侧小叶
        (-0.15 * x + 0.28 * y, 0.26 * x + 0.24 * y + 0.44)
    }
}
```

#### 生成结果

![Barnsley蕨类](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/barnsley_fern.png)

**分析**：经过 500,000 次迭代，点集收敛到蕨类植物的吸引子上。图像呈现出惊人的自然植物形态，每片小叶都是整体的缩小复制品，完美展示了 IFS 生成自然形态的能力。

---

### 3.3 龙形曲线 (Dragon Curve)

#### 算法原理

龙形曲线模拟**纸张对折后展开**的形状。定义转向序列：

- 初始：空序列
- 每次迭代：在中间插入 $R$（右转），然后将前半部分翻转并取反后追加

$$\emptyset \to R \to RRL \to RRLRRLL \to RRLRRLLRRRLLRLL \to \cdots$$

根据转向序列，从起点出发，每步前进固定距离，遇 $R$ 右转 90°，遇 $L$ 左转 90°。

#### 核心代码

```rust
fn generate_dragon_turns(iterations: u32) -> Vec<bool> {
    let mut turns = Vec::new();
    
    for _ in 0..iterations {
        let mut new_turns = turns.clone();
        new_turns.push(true);  // 右转
        
        // 翻转并取反前半部分
        for &turn in turns.iter().rev() {
            new_turns.push(!turn);
        }
        turns = new_turns;
    }
    turns
}
```

#### 生成结果

![龙形曲线](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/dragon_curve.png)

**分析**：15 次迭代生成 $2^{15} = 32768$ 条线段。曲线采用彩虹渐变着色，深蓝背景增强对比度。龙形曲线具有 4 重旋转对称性，是一种空间填充曲线的近似。

---

### 3.4 列维 C 曲线 (Lévy C Curve)

#### 算法原理

将每条线段替换为以其为斜边的**等腰直角三角形**的两条直角边：

```
原线段：A -------- B

替换后：    M
           /\
          /  \
         /    \
        A      B
```

其中 $M$ 是 $AB$ 中点向垂直方向偏移 $|AB|/2$ 的位置。

#### 核心代码

```rust
fn levy_c_recursive(img: &mut RgbImage, x1: f64, y1: f64, x2: f64, y2: f64, depth: u32) {
    if depth == 0 {
        draw_line(img, &Line::new(Point::new(x1, y1), Point::new(x2, y2)), color);
        return;
    }
    
    let dx = x2 - x1;
    let dy = y2 - y1;
    
    // 新顶点：中点 + 垂直偏移
    let mx = (x1 + x2) / 2.0 + dy / 2.0;
    let my = (y1 + y2) / 2.0 - dx / 2.0;
    
    levy_c_recursive(img, x1, y1, mx, my, depth - 1);
    levy_c_recursive(img, mx, my, x2, y2, depth - 1);
}
```

#### 生成结果

![列维C曲线](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/levy_c_curve.png)

**分析**：16 层递归生成 $2^{16} = 65536$ 条线段。列维 C 曲线具有对称美感，其分形维数约为 2，接近于平面填充。

---

### 3.5 L-System 分形植物

#### 算法原理

**L-System**（Lindenmayer 系统）是一种字符串重写系统：

1. **公理（Axiom）**：初始字符串
2. **产生式规则**：字符替换规则
3. **海龟解释器**：将字符串转换为图形

常用符号：
- `F`：向前移动并画线
- `+`/`-`：左转/右转
- `[`/`]`：保存/恢复状态（用于分支）

#### 四种植物的规则

| 植物 | 公理 | 规则 | 角度 |
|:----:|:----:|:----:|:----:|
| 分形草 | F | F → F[+F]F[-F]F | 25.7° |
| 灌木 | F | F → FF-[-F+F+F]+[+F-F-F] | 22.5° |
| 蕨类 | X | X → F+[[X]-X]-F[-FX]+X, F → FF | 25° |
| 花朵 | F | F → F[+F]F[-F][F] | 20° |

#### 生成结果

![分形植物](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/fractal_plants.png)

**分析**：四种植物展示了 L-System 的强大表达能力。从左到右：分形草紧凑对称；灌木茂密舒展；蕨类呈现经典的羽状结构；花朵状植物向上伸展。L-System 用简单规则生成了复杂的自然形态。

---

### 3.6 分形树变体（随机与风吹效果）

#### 算法改进

在基础分形树上增加：

1. **随机分支数**：每个节点生成 2-4 个子分支
2. **随机角度扰动**：$\theta' = \theta + \epsilon$，$\epsilon \sim U(-0.1, 0.1)$
3. **风力因子**：$\theta' = \theta + w \cdot (1-t)$，其中 $w$ 为风力，$t$ 为深度比例

#### 核心代码

```rust
// 随机生成 2-4 个分支
let num_branches = rng.gen_range(2..=4);

for i in 0..num_branches {
    let branch_offset = (i as f64 - (num_branches - 1) as f64 / 2.0) * spread;
    let random_offset = (rng.r#gen::<f64>() - 0.5) * 0.2;
    let wind_offset = wind_factor * (1.0 - t);  // 风力对细枝影响更大
    
    let new_angle = angle + branch_offset + random_offset + wind_offset;
    let new_length = length * (0.6 + rng.r#gen::<f64>() * 0.2);
    
    draw_random_tree(img, end_x, end_y, new_angle, new_length, depth - 1, ...);
}
```

#### 生成结果

![分形树变体](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/fractal_tree_variants.png)

**分析**：从左到右展示了不同风力条件下的树木形态：
- **树1**：无风，对称生长
- **树2**：微风向右（$w=0.15$）
- **树3**：强风向右（$w=0.3$），明显倾斜
- **树4**：微风向左（$w=-0.1$）
- **树5**：微风向右的大树

随机性使每棵树都独一无二，更接近自然界的真实树木。

---

### 3.7 奶滴扩散模拟

#### 实验设置

| 参数 | 值 |
|:----:|:----:|
| 容器尺寸 | $50 \times 50$ 格子 |
| 孔洞位置 | 右边墙，$y \in [20, 30)$ |
| 孔洞长度 | 10 格 |
| 初始粒子数 | $N_0 = 200$ |
| 初始分布 | 中心 $10 \times 10$ 区域 |
| 模拟时长 | $10^6$ 时间步 |

#### 核心代码

```rust
pub fn step(&mut self) {
    let mut rng = rand::thread_rng();
    
    for particle in &mut self.particles {
        if !particle.active { continue; }
        
        // 随机选择方向：上下左右
        let direction = rng.r#gen::<u8>() % 4;
        let (dx, dy) = match direction {
            0 => (1, 0),   // 右
            1 => (-1, 0),  // 左
            2 => (0, 1),   // 下
            _ => (0, -1),  // 上
        };
        
        let new_x = particle.x + dx;
        let new_y = particle.y + dy;
        
        // 检查是否通过孔洞逃逸
        if new_x >= width && new_y >= hole_start && new_y < hole_end {
            particle.active = false;
            continue;
        }
        
        // 边界检查（碰壁不移动）
        if new_x >= 0 && new_x < width && new_y >= 0 && new_y < height {
            particle.x = new_x;
            particle.y = new_y;
        }
    }
    self.time += 1;
}
```

#### 扩散过程可视化

| $t = 0$（初始） | $t = 10^4$ |
|:---------------:|:----------:|
| ![t=0](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/diffusion_t0.png) | ![t=1e4](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/diffusion_t1e4.png) |

| $t = 10^5$ | $t = 10^6$ |
|:----------:|:----------:|
| ![t=1e5](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/diffusion_t1e5.png) | ![t=1e6](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/diffusion_t1e6.png) |

**观察**：
- $t=0$：粒子集中在中心区域
- $t=10^4$：粒子扩散开来，部分已逃逸
- $t=10^5$：大部分粒子已逃逸
- $t=10^6$：几乎所有粒子都已逃逸

#### 实验数据

| 时刻 $t$ | 粒子数 $N$ | $\ln(N/N_0)$ |
|:--------:|:----------:|:------------:|
| 0 | 200 | 0.00 |
| 10,000 | 52 | -1.35 |
| 20,000 | 17 | -2.47 |
| 30,000 | 9 | -3.10 |
| 40,000 | 3 | -4.20 |
| 50,000 | 2 | -4.61 |
| 100,000 | 0 | — |

#### 数据拟合与分析

对 $\ln(N/N_0)$ 与 $t$ 进行最小二乘线性拟合：

$$\ln\frac{N}{N_0} = kt$$

使用公式：

$$k = \frac{\sum t_i \ln(N_i/N_0)}{\sum t_i^2}$$

**拟合结果**：

$$k = -1.16 \times 10^{-4}, \quad \tau = -\frac{1}{k} \approx 8621 \text{ 时间步}$$

**半衰期**：

$$t_{1/2} = \tau \ln 2 \approx 5975 \text{ 时间步}$$

#### 拟合图

![ln(N/N0) vs t](https://raw.githubusercontent.com/g1157/Rust_learning/main/homework/hk11/diffusion_lnN_vs_t.png)

**分析**：
- 红色圆点为实验数据，蓝色直线为理论拟合线 $\ln(N/N_0) = -t/\tau$
- 数据点基本落在理论直线上，验证了指数衰减规律
- 后期数据点偏离较大，这是由于粒子数太少导致的统计涨落

---

## 四、结论

1. **分形图形**：成功实现了 6 种几何/递归类分形，包括分形树、Barnsley 蕨类、龙形曲线、列维 C 曲线、L-System 分形植物和随机分形树。这些算法展示了分形的自相似性和用简单规则生成复杂图形的能力。

2. **扩散模拟**：随机漫步模拟验证了粒子数的指数衰减规律 $N(t) = N_0 e^{-t/\tau}$。拟合得到特征时间 $\tau \approx 8621$ 时间步，半衰期 $t_{1/2} \approx 5975$ 时间步。

3. **编程实践**：使用 Rust 语言实现了模块化的代码结构，利用 `image` crate 进行图像生成，`plotters` crate 进行数据可视化，展示了 Rust 在科学计算领域的应用潜力。

---

## 五、项目结构

```
src/
├── main.rs            # 主入口
├── common.rs          # 公共模块（Point, Line, 绘图函数）
├── fractal_tree.rs    # 分形树算法
├── barnsley_fern.rs   # Barnsley 蕨类算法
├── dragon_curve.rs    # 龙形曲线
├── fractal_plants.rs  # 列维C曲线、L-System 植物、分形树变体
└── diffusion.rs       # 奶滴扩散模拟
```

## 六、运行方式

```bash
cargo run --release
```

## 七、依赖

```toml
[dependencies]
image = "0.25"
rand = "0.8"
plotters = "0.3"
```

---

## 参考文献

1. Mandelbrot, B. B. (1982). *The Fractal Geometry of Nature*. W. H. Freeman.
2. Barnsley, M. F. (1988). *Fractals Everywhere*. Academic Press.
3. Prusinkiewicz, P., & Lindenmayer, A. (1990). *The Algorithmic Beauty of Plants*. Springer.
