# 作业十二：二维伊辛模型蒙特卡洛模拟

> **课程**：计算物理  
> **学号**：[202332021221]  
> **姓名**：[刘凤祥]  
> **日期**：2024年12月

---

## 一、实验目的

1. 通过 Metropolis 蒙特卡洛方法模拟二维伊辛模型，深入理解**相变与临界现象**
2. 估算临界指数 β 并比较正方形与三角形晶格的结果，验证临界指数的**普适性**
3. 在高温极限验证 M(H) ≈ tanh(H/T)，分析外场响应的偏差与温度的关系

---

## 二、实验原理

### 2.1 伊辛模型简介

伊辛模型（Ising Model）是统计物理中描述**铁磁性相变**的经典模型，由 Ernst Ising 于 1925 年提出。

**基本设定**：
- 系统由位于晶格格点上的**自旋**组成
- 每个自旋只能取 +1（向上）或 -1（向下）两个状态
- 相邻自旋之间存在**最近邻相互作用**

### 2.2 哈密顿量（能量函数）

系统的总能量由哈密顿量描述：

$$
H = -J \sum_{\langle i,j \rangle} s_i s_j - \mu h \sum_i s_i
$$

其中：
| 符号 | 含义 |
|:----:|:----:|
| J | 交换耦合常数（J > 0 表示铁磁耦合） |
| s_i = ±1 | 第 i 个格点的自旋 |
| ⟨i,j⟩ | 对所有最近邻自旋对求和 |
| h | 外加磁场强度 |
| μ | 磁矩 |

**物理意义**：
- 第一项：相邻自旋**同向排列**时能量更低（s_i × s_j = +1）
- 第二项：自旋沿外场方向排列时能量更低

### 2.3 磁化强度

系统的磁化强度定义为所有自旋的平均值：

$$
M = \frac{1}{N} \sum_i s_i
$$

- |M| = 1：完全有序（所有自旋同向）
- M = 0：完全无序（自旋随机取向）

### 2.4 相变与临界现象

伊辛模型存在一个**临界温度** T_c，系统行为在此发生**质变**：

| 温度区间 | 物理行为 | 相态 |
|:--------:|:--------:|:----:|
| T < T_c | 自旋倾向于平行排列，自发磁化 M ≠ 0 | 铁磁相 |
| T > T_c | 热涨落主导，自旋随机取向，M → 0 | 顺磁相 |
| T = T_c | 临界点，关联长度发散，涨落增强 | 临界态 |

#### 精确解的临界温度

对于二维晶格（设 J = k_B = 1）：

| 晶格类型 | 临界温度 T_c | 配位数 z |
|:--------:|:------------:|:--------:|
| 正方形 | 2/ln(1 + √2) ≈ 2.269 | 4 |
| 三角形 | 4/ln(3) ≈ 3.641 | 6 |

### 2.5 临界指数 β 与普适性

在临界温度附近，磁化强度遵循**幂律行为**：

$$
M \sim (T_c - T)^\beta \quad \text{当 } T \to T_c^-
$$

**二维伊辛模型的精确结果**：β = 1/8 = 0.125

**普适性（Universality）**：
- 临界指数只依赖于系统的**维度**和**对称性**
- 与晶格细节（正方形、三角形等）无关
- 这是相变理论的核心概念之一

### 2.6 蒙特卡洛方法：Metropolis 算法

#### 为什么需要蒙特卡洛？

对于 N 个自旋的系统，共有 2^N 种构型。即使 N = 32 × 32 = 1024，构型数也是 2^1024 ——远超宇宙原子数！

**解决方案**：**重要性抽样**——不遍历所有构型，而是按玻尔兹曼权重 exp(-βH) 抽样。

#### Metropolis 算法步骤

```
1. 随机选择一个自旋 s_i
2. 计算翻转该自旋导致的能量变化 ΔE
3. 接受准则：
   - 若 ΔE ≤ 0（能量降低）：接受翻转
   - 若 ΔE > 0：以概率 exp(-ΔE/T) 接受翻转
4. 重复 N 次 = 1 次"扫描"(sweep)
```

#### 能量变化的快速计算

翻转 s_i 时，能量变化为：

$$
\Delta E = 2 s_i \left( J \sum_{j \in \text{nn}(i)} s_j + \mu h \right)
$$

**关键优化**：只需查看 4 个（正方形）或 6 个（三角形）近邻，是 O(1) 操作！

### 2.7 估算临界指数 β 的方法

#### 方法一：M^(1/β*) vs T 线性度分析

由幂律 M = A(T_c - T)^β，有：

$$
M^{1/\beta} = A^{1/\beta}(T_c - T)
$$

因此 M^(1/β) 与 T 呈**线性关系**。

**算法**：扫描不同的 β* 值，计算 M^(1/β*) vs T 的线性拟合 R²，选择 R² 最大的 β* 作为最佳估计。

**优势**：无需预先知道 T_c

#### 方法二：log(M) vs log(T_c - T) 拟合

取对数：

$$
\log M = \beta \log(T_c - T) + \log A
$$

在 log-log 图上，**斜率即为 β**。

**注意**：需要已知 T_c 的估计值

### 2.8 高温极限与外场响应（作业二）

在极高温度下（T >> T_c），自旋间相互作用可忽略。单个自旋在外场中的平均值为：

$$
\langle s \rangle = \frac{e^{h/T} - e^{-h/T}}{e^{h/T} + e^{-h/T}} = \tanh\left(\frac{h}{T}\right)
$$

设 μ = k_B = 1，则：

$$
M_{\text{theory}} = \tanh\left(\frac{h}{T}\right)
$$

**温度依赖性**：

| 温度 | 行为 | 偏差 |
|:----:|:----:|:----:|
| T = 100 | 自旋相互作用几乎可忽略 | M(H) ≈ tanh(H/T) |
| T = 30, 10 | 相互作用开始显著 | 偏差增大 |
| T → T_c | 临界涨落主导 | 近似完全失效 |

---

## 三、晶格结构与实现

### 3.1 正方形晶格（Square Lattice）

- 每个自旋有 **4 个最近邻**
- 配位数 z = 4
- 周期性边界条件

```
  ↑ ↓ ↑ ↓
  ↓ ↑ ↓ ↑
  ↑ ↓ ↑ ↓
  ↓ ↑ ↓ ↑
```

### 3.2 三角形晶格（Triangular Lattice）

- 每个自旋有 **6 个最近邻**
- 配位数 z = 6
- 使用**偏移行（skewed row）**方法实现

```
    ↑   ↓   ↑
  ↓   ↑   ↓   ↑
    ↑   ↓   ↑
  ↓   ↑   ↓   ↑
```

#### 实现技巧

在正方网格上实现三角形拓扑：
- **偶数行**：额外连接右上、左下
- **奇数行**：额外连接左上、右下

```rust
let (diag1, diag2) = if row % 2 == 0 {
    // 偶数行：右上、左下
    let up_right = ((row + size - 1) % size) * size + (col + 1) % size;
    let down_left = ((row + 1) % size) * size + (col + size - 1) % size;
    (up_right, down_left)
} else {
    // 奇数行：左上、右下
    let up_left = ((row + size - 1) % size) * size + (col + size - 1) % size;
    let down_right = ((row + 1) % size) * size + (col + 1) % size;
    (up_left, down_right)
};
```

---

## 四、项目结构

```
hk12/
├── src/
│   ├── main.rs              # 主入口
│   ├── ising_critical.rs    # 作业一：临界指数 β 计算
│   └── ising_field.rs       # 作业二：高温外场响应
├── plots/                   # 生成的可视化图表
├── ising_critical_results.csv
├── ising_field_results.csv
├── README.md
├── REPORT.md
├── Cargo.toml
└── Cargo.lock
```

---

## 五、运行方式

### 5.1 依赖

- Rust 1.75+（建议）
- crates: `rand`, `plotters`, `clap`

### 5.2 编译

```bash
cargo build --release
```

### 5.3 运行作业一：临界指数计算

```bash
# 正方形晶格（默认生成可视化图表）
cargo run --release --bin ising_critical -- --lattice square --size 32

# 三角形晶格
cargo run --release --bin ising_critical -- --lattice triangular --size 32

# 自定义参数
cargo run --release --bin ising_critical -- --size 64 --sweeps 5000

# 禁用可视化
cargo run --release --bin ising_critical -- --no-plot
```

### 5.4 运行作业二：高温外场响应

```bash
# 默认参数（T = 100, 30, 10）
cargo run --release --bin ising_field -- --temperatures 100,30,10

# 自定义温度
cargo run --release --bin ising_field -- --temperatures 100,50,20,10,5

# 禁用可视化
cargo run --release --bin ising_field -- --no-plot
```

### 5.5 查看帮助

```bash
cargo run --release --bin ising_critical -- --help
cargo run --release --bin ising_field -- --help
```

---

## 六、实验结果

### 6.1 作业一：临界指数 β

| 晶格类型 | 估算 β | 理论值 | 相对误差 |
|:--------:|:------:|:------:|:--------:|
| 正方形 | ~0.12 | 0.125 | <6% |
| 三角形 | ~0.12 | 0.125 | <6% |

**结论**：两种晶格得到相同的临界指数，验证了**普适性**。

![磁化强度 vs 温度](plots/m_vs_t_square.png)

*图1: 正方形晶格磁化强度随温度变化，红线标注临界温度 Tc ≈ 2.27*

![log-log 拟合](plots/loglog_beta_square.png)

*图2: log(M) vs log(Tc-T) 图，斜率即为临界指数 β ≈ 0.12*

### 6.2 作业二：高温外场响应

| 温度 T | 平均绝对误差 | 与 tanh(H/T) 符合程度 |
|:------:|:------------:|:--------------------:|
| 100 | ~0.001 | 极好 |
| 30  | ~0.004 | 良好 |
| 10  | ~0.040 | 偏差显著 |

**结论**：温度降低时，与 tanh(H/T) 的偏差增大约 **40 倍**，验证了自旋相互作用的影响。

![外场响应](plots/m_vs_h_field.png)

*图3: 不同温度下 M(H) 与 tanh(H/T) 理论曲线对比*

![误差分析](plots/error_vs_temp.png)

*图4: 平均绝对误差随温度的变化*

---

## 七、可视化输出列表

程序会自动在 `plots/` 目录生成以下图表：

### 作业一图表

| 文件名 | 描述 |
|:------:|:----:|
| `m_vs_t_square.png` | 正方形晶格：磁化强度 M vs 温度 T 曲线 |
| `m_vs_t_triangular.png` | 三角形晶格：磁化强度 M vs 温度 T 曲线 |
| `loglog_beta_square.png` | log(M) vs log(Tc-T) 图（含拟合直线和 β 估计） |
| `loglog_beta_triangular.png` | 三角形晶格的 log-log 图 |
| `m_beta_star_square.png` | M^(1/β*) vs T 曲线（线性度分析） |
| `m_beta_star_triangular.png` | 三角形晶格的线性度分析 |

### 作业二图表

| 文件名 | 描述 |
|:------:|:----:|
| `m_vs_h_field.png` | M(H) vs H 曲线（多温度对比，含理论曲线） |
| `error_vs_temp.png` | 平均绝对误差随温度变化柱状图 |

---

## 八、代码实现亮点

### 8.1 近邻预计算

初始化时构建周期性边界条件下的近邻索引表，模拟阶段 O(1) 访问：

```rust
fn build_neighbors(size: usize, lattice_type: LatticeType) -> Vec<Vec<usize>> {
    // 预计算所有格点的近邻，避免运行时重复计算
    let mut neighbors = vec![Vec::new(); size * size];
    for row in 0..size {
        for col in 0..size {
            // 周期性边界条件
            let up = ((row + size - 1) % size) * size + col;
            let down = ((row + 1) % size) * size + col;
            // ... 
        }
    }
    neighbors
}
```

### 8.2 Metropolis 核心循环

```rust
fn metropolis_sweep(&mut self, beta: f64, rng: &mut StdRng) {
    let n = self.spins.len();
    for _ in 0..n {
        let idx = rng.gen_range(0..n);  // 随机选自旋
        let delta_e = self.energy_delta(idx);
        
        // Metropolis 接受准则
        if delta_e <= 0.0 || rng.gen::<f64>() < (-beta * delta_e).exp() {
            self.spins[idx] = -self.spins[idx];  // 翻转！
        }
    }
}
```

### 8.3 数据分析

- **线性回归**：最小二乘法计算斜率和 R²
- **温度窗口过滤**：只使用 0.85×T_c < T < 0.98×T_c 的数据，避免有限尺寸效应

---

## 九、结论

1. **临界指数 β**：通过 Metropolis 蒙特卡洛方法，估算得到 β ≈ 0.12，与 Onsager 精确解 β = 0.125 符合良好
2. **普适性验证**：正方形和三角形晶格得到相同的临界指数，验证了相变的普适性概念
3. **高温近似**：验证了 T >> T_c 时 M(H) ≈ tanh(H/T) 的近似，以及随温度降低偏差增大的规律
4. **代码实现**：近邻预计算、偏移行三角晶格、自动可视化等技术提升了代码效率和可用性

---

## 十、参考文献

1. Onsager, L. (1944). Crystal statistics. I. A two-dimensional model with an order-disorder transition. *Physical Review*, 65(3-4), 117.
2. Metropolis, N., et al. (1953). Equation of state calculations by fast computing machines. *The Journal of Chemical Physics*, 21(6), 1087-1092.
3. Newman, M. E. J., & Barkema, G. T. (1999). *Monte Carlo Methods in Statistical Physics*. Oxford University Press.
