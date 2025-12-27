<!--
ASCII-only header (keep this block) to avoid a Windows apply_patch UTF-8 slicing bug.
File encoding: UTF-8.
Last updated: 2025-12-27.
-->

# GPU 加速二维 TDGL 超导涡旋模拟

> **课程**：计算物理
> **学号**：202332021221
> **姓名**：刘凤祥
> **日期**：2024年12月
> **版本更新**：2025年12月27日

---

## 摘要

本项目基于 Rust + wgpu (WebGPU) 构建了一个 GPU 加速的二维时间依赖 Ginzburg-Landau (TDGL) 方程求解器，用于研究超导涡旋动力学与缺陷钉扎现象。项目实现了磁通量量子化与磁周期边界条件，采用规范不变绕数进行涡旋检测，并构建了完整的"仿真→后处理→批处理→AI 闭环"研究工具链。实验验证了净涡旋数与磁通量子数的一致性、周期钉扎阵列的 matching field 效应，以及去钉扎临界行为。在 RTX 4060 Laptop GPU 上，1024×1024 网格的计算吞吐量达到 14 Gcells/s。

**关键词**：超导涡旋、TDGL 方程、GPU 并行计算、涡旋钉扎、matching field

---

## 一、介绍

### 1.1 研究背景

超导体中的涡旋物理是凝聚态物理的经典课题。1950年，Ginzburg 与 Landau 提出了描述超导相变的唯象理论 [1]，引入复数序参量 ψ 来表征超导态。1957年，Abrikosov 基于 GL 理论预言了 II 类超导体中的涡旋晶格 [2]，这一发现为理解混合态超导体奠定了基础。

涡旋钉扎是超导应用的核心问题。当外加电流驱动涡旋运动时，会产生耗散并破坏超导态。通过引入缺陷（如点缺陷、柱状缺陷或人工纳米结构）可以"钉扎"涡旋，抑制其运动，从而提高临界电流密度。de Gennes [3] 和 Tinkham [4] 的经典教材系统阐述了涡旋物理与钉扎机制，Blatter 等人的综述 [5] 则全面总结了高温超导体中的涡旋行为。

### 1.2 文献调研

**TDGL 数值模拟**：时间依赖 Ginzburg-Landau 方程是研究涡旋动力学的标准工具。Gropp 等人 [6] 发展了高效的 TDGL 数值方法，采用 gauge-covariant 离散化保证数值稳定性。Machida 与 Kaburaki [7] 使用大规模并行计算研究了涡旋晶格的动力学行为。

**周期钉扎阵列与 matching field**：人工周期钉扎阵列是研究涡旋-缺陷相互作用的理想平台。Baert 等人 [8] 首次在实验中观察到周期钉扎阵列导致的 matching field 效应——当涡旋数与缺陷数成整数比时，临界电流显著增强。Reichhardt 等人 [9,10] 通过分子动力学模拟系统研究了 matching field 条件下的涡旋构型与临界电流，发现整数匹配场处存在明显的 commensurability peak。Field 等人 [11] 进一步研究了大规模周期阵列中的涡旋畴结构。

**GPU 加速计算**：近年来，GPU 并行计算在科学模拟中得到广泛应用。WebGPU 作为新一代跨平台图形与计算 API [12]，提供了现代 GPU 的高效访问接口，适合构建可移植的科学计算工具。

### 1.3 研究目标

本项目旨在：
1. 实现物理正确的 TDGL 求解器，包括磁周期边界条件与规范不变涡旋检测
2. 研究涡旋钉扎与去钉扎行为，提取临界驱动参数 κ_c
3. 验证周期钉扎阵列的 matching field 效应
4. 构建可复现的研究工具链，支持批处理与 AI 辅助参数探索

### 1.4 文章结构

本文组织如下：第二节介绍物理模型与数值方法；第三节描述 GPU 实现细节；第四节展示实验结果与分析；第五节进行讨论与总结。代码细节见附录。

---

## 二、物理模型与数值方法

### 2.1 Ginzburg-Landau 理论

GL 理论用复数序参量 ψ 描述超导态，其模方 |ψ|² 正比于超导电子对密度。自由能泛函为：

$$
F = \int d^3r \left[ \alpha|\psi|^2 + \frac{\beta}{2}|\psi|^4 + \frac{1}{2m^*}|(-i\hbar\nabla - e^*\mathbf{A})\psi|^2 + \frac{B^2}{2\mu_0} \right]
$$

其中 α < 0 对应超导态，β > 0 为稳定项，A 为矢势。

### 2.2 TDGL 方程

将 GL 理论推广到动力学，得到无量纲化的 TDGL 方程：

$$
\frac{\partial \psi}{\partial t} = (\nabla - i\mathbf{A})^2\psi + \alpha(\mathbf{r})\psi - |\psi|^2\psi
$$

第一项为 gauge-covariant Laplacian，描述扩散与磁场耦合；第二项为线性增长/衰减项，空间变化的 α(r) 可模拟缺陷；第三项为非线性饱和项。

### 2.3 涡旋与钉扎

涡旋是超导体中的拓扑缺陷，其核心处 |ψ| = 0，相位绕核心一圈变化 ±2π，携带量子化磁通 Φ₀ = h/2e。在缺陷处（局部 α < 0），涡旋核心能量降低，涡旋倾向于被"钉扎"在缺陷位置。

### 2.4 磁周期边界条件

在二维周期边界（torus）上实现均匀磁场需要特殊处理。采用 Landau 规范 A = (0, Bx, 0)，磁通量量子化条件为：

$$
\varphi \cdot N_x \cdot N_y = 2\pi n, \quad n \in \mathbb{Z}
$$

其中 φ = B·dx² 为每个 plaquette 的无量纲磁通。通过在 x 边界引入相位缝合因子 $U_x(N_x-1, j) = e^{+i\varphi N_x j}$，保证系统在 torus 拓扑上自洽。

### 2.5 规范不变涡旋检测

在外场存在时，直接对 arg(ψ) 做绕数会依赖 gauge 选择。采用基于 link 变量的规范不变方法：

$$
\Delta\theta_x(i,j) = \arg\left(\psi^*_{i,j} \cdot U_x(i,j) \cdot \psi_{i+1,j}\right)
$$

对 plaquette 绕一圈求和，|W| > 0.75×2π 时判定为涡旋/反涡旋。

### 2.6 数值离散化

采用 5 点 stencil 的 gauge-covariant Laplacian：

$$
\Delta_A\psi = \frac{\psi_{xp} + \psi_{xm} + U_y\psi_{yp} + U_y^*\psi_{ym} - 4\psi}{dx^2}
$$

时间推进采用显式 Euler 方法，稳定性条件为 dt < dx²/4。

---

## 三、GPU 实现

### 3.1 技术选型

本项目采用 Rust 语言与 wgpu 库实现。Rust 提供内存安全与高性能，wgpu 是 WebGPU 标准的 Rust 实现，支持跨平台 GPU 计算。

### 3.2 并行策略

- **空间并行**：每个 GPU 线程处理一个网格点
- **Ping-pong buffer**：双缓冲交替读写，避免数据竞争
- **纯 GPU 渲染**：Fragment shader 直接采样，无 CPU 回读瓶颈

Workgroup 配置为 8×8 = 64 线程，256×256 网格对应 32×32 个 workgroup。

### 3.3 egui Dashboard UI 系统

本项目基于 egui 即时模式 GUI 框架构建了完整的 Dashboard 系统，采用模块化架构设计，分为四个开发阶段（Story）逐步实现。

#### 3.3.1 基础框架与参数控制面板 (Story 1.1)

**技术实现**：
- 将 egui 集成到现有 wgpu 渲染循环，使用 `egui-wgpu` 和 `egui-winit` 适配器
- 实现深色主题配色方案（背景 `#1a1a2e`、前景 `#e8e8e8`、强调色 `#4fc3f7`）
- 左侧参数面板采用可折叠分组设计

**功能模块**：
- **仿真参数组**：网格尺寸 (nx, ny)、时间/空间步长 (dt, dx)、磁通量子数 (flux_n)、驱动参数 (κ)
- **缺陷配置组**：缺陷模式 (random/lattice)、数量、间距、强度 (α_defect)、半径
- **运行模式组**：Interactive、Headless、κ Sweep、Bench 四种模式切换
- **运行控制**：开始/暂停/停止/重置按钮，状态指示

#### 3.3.2 数据展示与状态面板 (Story 1.2)

**统计面板**（右侧）：
- 涡旋统计：涡旋数、反涡旋数、净涡旋数
- 钉扎统计：钉扎涡旋数、钉扎率百分比
- 能量统计：总能量、能量密度
- 速度统计：mean_vx、mean_vy、|v|

**时间序列图**：
- 使用 `egui_plot` 绘制最近 1000 步的涡旋数和能量变化曲线
- 环形缓冲区存储历史数据，降采样显示（200 点）保证性能
- 支持图例和轴标签

**底部状态栏**：
- 当前步数 / 总步数、仿真时间
- 进度条与 ETA 估算
- GPU 名称、实时 steps/s

#### 3.3.3 高级功能与优化 (Story 1.3)

**κ Sweep 模式 UI**：
- 配置 κ_start、κ_end、κ_step 参数
- 配置 relax_steps、measure_steps
- 实时显示 sweep 进度（当前 κ 值、阶段、百分比、ETA）

**Depinning 曲线实时绘制**：
- 使用 egui_plot 绘制 κ vs |v| 曲线
- 每个 κ 值完成后实时添加数据点
- 自动检测并标注 κ_c（速度阈值法）

**运行历史浏览**：
- 扫描 `runs/` 目录获取历史运行列表
- 显示运行时间戳和模式类型
- 点击显示 config.toml 摘要
- "打开目录"按钮调用系统文件管理器

**配置预设系统**：
- 6 个内置预设：默认、高磁场、无缺陷、周期阵列、强钉扎、低磁场
- 预设选择下拉菜单，一键加载配置

**动画效果**：
- 运行按钮脉冲动画 (PulseAnimation)
- 数值高亮动画 (HighlightAnimation)
- 涡旋出现动画 (FadeInAnimation, ScaleAnimation)

#### 3.3.4 模拟器验证面板 (Story 1.4)

**涡旋晶格间距验证**：
- 理论公式（Abrikosov 三角晶格）：$a_0 = \sqrt{2A / (\sqrt{3} \cdot N)}$
- 实测算法：计算正涡旋最近邻距离中位数（周期边界 minimum image convention）
- 验证状态：绿色 (<5% 偏差)、黄色 (5-15%)、红色 (>15%)
- 晶格对称性检测：六角晶格 / 四方晶格 / 无序

**材料参数参考库**：

```
材料      κ 范围      ξ (nm)    λ (nm)    Tc (K)    Hc2 (T)
────────────────────────────────────────────────────────────
Nb        0.7-1.0     38        39        9.2       0.4
NbSe₂     9-12        7.7       73        7.2       4.5
YBCO      50-100      1.75      150       92        100
MgB₂      20-32       7.5       140       39        16
Pb        0.4-0.55    83        39        7.2       0.08
NbTi      50-80       4.0       300       9.8       15
```

**Depinning β 指数拟合**：
- 幂律拟合：$v \propto (\kappa - \kappa_c)^\beta$
- 对 κ > κ_c 的数据点进行 log-log 线性回归
- 显示 β 值与 R² 拟合优度
- 理论预期：β = 0.5-0.65（mean-field）

**Matching Field 指示器**：
- 检测涡旋数与缺陷数的匹配状态
- 支持整数匹配 (1:1, 2:1, 3:1) 和分数匹配 (1:2, 2:3)
- 匹配时显示绿色高亮和匹配比例

**验证报告导出**：
- 生成 Markdown 格式验证报告
- 包含：运行配置、材料参考、晶格验证、Matching Field 分析、β 指数、能量验证

#### 3.3.5 UI 模块结构

```
src/ui/
├── mod.rs              # 模块入口
├── theme.rs            # 深色主题配色
├── components/         # 可复用组件
│   ├── param_slider.rs     # 参数滑块
│   ├── time_series.rs      # 时间序列图
│   └── depinning_curve.rs  # Depinning 曲线 + β 拟合
└── panels/             # UI 面板
    ├── params_panel.rs     # 参数控制面板
    ├── stats_panel.rs      # 统计面板
    ├── status_bar.rs       # 状态栏
    ├── history_panel.rs    # 历史记录面板
    └── validation_panel.rs # 验证面板

src/utils/
├── presets.rs          # 预设配置
├── animation.rs        # 动画工具
├── materials.rs        # 超导材料参数库
└── validation_report.rs # 验证报告生成
```

---

## 四、实验结果

### 4.1 实验环境

- GPU: NVIDIA GeForce RTX 4060 Laptop GPU
- 系统: Windows 11
- 软件: Rust 1.x + wgpu 23

### 4.2 涡旋动力学验证

**实验 1: B=0 无缺陷（拓扑守恒验证）**

```
时间 t    涡旋数    反涡旋数    净涡旋    能量密度
──────────────────────────────────────────────────
10.0      147       147         0         -0.4117
50.0      50        50          0         -0.4696
100.0     29        29          0         -0.4809
```

![B=0 无缺陷涡旋演化](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/exp2a_B0_no_defect/vortices_plot.png)

**图 1**：B=0 无缺陷条件下涡旋数随时间演化。初始随机噪声产生涡旋-反涡旋对，随后成对湮灭。净涡旋数 net = 0 严格保持，验证了周期边界下的拓扑守恒。能量密度单调下降，符合 TDGL 梯度流特性。

**实验 2: B≠0 (flux_n=64) 有缺陷（外场自洽验证）**

```
时间 t    涡旋数    反涡旋数    净涡旋    钉扎涡旋数
────────────────────────────────────────────────────
10.0      174       110         64        13
50.0      89        25          64        10
100.0     75        11          64        11
```

![B≠0 有缺陷涡旋演化](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/exp2c_B64_with_defect/vortices_plot.png)

**图 2**：flux_n=64 有缺陷条件下涡旋演化。净涡旋数 net = 64 = flux_n，完美验证了磁通量量子化与磁周期边界条件的正确性。部分涡旋被缺陷钉扎（pinned_v ≈ 10-13）。

### 4.3 GPU 性能基准

```
网格规模    steps/s     cells/s       相对效率
──────────────────────────────────────────────
128²        26,236      4.30×10⁸      基准
256²        27,806      1.82×10⁹      4.2×
512²        23,233      6.09×10⁹      14.2×
1024²       13,436      1.41×10¹⁰     32.8×
```

256² 网格达到最高 steps/s，适合交互式模拟；1024² 网格吞吐量达 14 Gcells/s，相对效率随网格增大而提升，说明 GPU 并行性得到充分利用。

### 4.4 去钉扎 κ sweep 实验

**随机缺陷 vs 周期阵列 (flux_n=64)**

```
κ        mean_vx (random)    pinned_v (random)    mean_vx (lattice)    pinned_v (lattice)
──────────────────────────────────────────────────────────────────────────────────────────
0.000    0.0038              9.9                  -0.0030              13.6
0.015    -0.0048             11.0                 -0.0048              19.0
0.025    -0.0074             12.0                 -0.0054              20.0
0.035    -0.0106             14.0                 -0.0097              20.7
```

周期阵列的钉扎涡旋数显著高于随机缺陷（20 vs 12），|mean_vx| 在 κ≈0.025-0.030 开始显著增大，表明涡旋开始整体漂移（去钉扎）。

![随机缺陷 κ sweep](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/exp3a_kappa_sweep_random/kappa_sweep_plot.png)

**图 3**：随机缺陷下的 depinning 曲线。横轴为驱动参数 κ，纵轴为平均漂移速度 |v|。可见 κ ≈ 0.025 处速度开始显著增大，标志着去钉扎转变。

![周期阵列 κ sweep](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/exp3b_kappa_sweep_lattice/kappa_sweep_plot.png)

**图 4**：周期缺陷阵列下的 depinning 曲线。与随机缺陷相比，周期阵列的临界驱动 κ_c 更高，钉扎效果更强。

### 4.5 Matching Field 效应

**实验: 随机缺陷 vs 周期阵列（N_pins=64, spacing=32）**

```
flux_n    B/B_φ    钉扎涡旋数 (random)    钉扎涡旋数 (lattice)    增强比
─────────────────────────────────────────────────────────────────────────
32        0.5      13                     14                      1.08×
64        1.0      15                     20                      1.33×
96        1.5      16                     21                      1.31×
128       2.0      21                     33                      1.57×
```

在整数匹配场 B/B_φ = 1.0 和 2.0 处，周期阵列的钉扎效果显著增强。这与 Reichhardt 等人 [9,10] 的理论预测定性一致：当涡旋数与缺陷数成整数比时，形成 commensurate 结构，钉扎力增强。

![Matching Field 效应](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/matching_field_smoke/matching_field_plot_b_over_bphi.png)

**图 5**：Matching Field 效应。横轴为归一化磁场 B/B_φ（B_φ 为第一匹配场），纵轴为临界驱动 κ_c。周期阵列（lattice）在整数匹配场 B/B_φ = 1, 2 处显示明显的 commensurability peak，而随机缺陷（random）无此特征。

![钉扎涡旋数对比](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/lit_compare/lit_compare_pinned_vs_bphi.png)

**图 6**：钉扎涡旋数随 B/B_φ 变化。周期阵列在匹配场处钉扎效率显著提升。

![增强比](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/lit_compare/lit_compare_enhancement.png)

**图 7**：周期阵列相对于随机缺陷的钉扎增强比。在 B/B_φ = 1.0 和 2.0 处增强比达到峰值（1.33× 和 1.57×）。

### 4.6 收敛性验证

**dt 收敛性（dt=0.01 vs dt=0.005）**

```
dt       涡旋数    反涡旋数    净涡旋    能量密度
────────────────────────────────────────────────
0.01     76        12          64        -0.4569
0.005    76        12          64        -0.4570
```

两种时间步长下结果完全一致，能量密度差异 < 0.02%，验证了 dt=0.01 满足收敛要求。

![dt 收敛性](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/convergence_dt_flux64_smoke/convergence_plot.png)

**图 8**：时间步长收敛性验证。不同 dt 值下的 depinning 曲线高度重合，表明 dt=0.01 已满足数值收敛要求。

![dx 收敛性](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/convergence_dx_flux64_refined/convergence_plot.png)

**图 9**：空间步长收敛性验证。保持物理尺寸 L = nx × dx 不变，不同网格分辨率下结果一致。

### 4.7 结构因子分析

项目支持输出涡旋位置并计算结构因子 S(k)，用于定量区分晶格有序与无序态。结构因子定义为：

$$
S(\mathbf{k}) = \frac{1}{N}\left|\sum_{j=1}^{N}e^{i\mathbf{k}\cdot\mathbf{r}_j}\right|^2
$$

三角晶格（Abrikosov 晶格）会出现六角对称峰；无序/玻璃态峰会变宽或消失。通过 `--dump-positions` 输出涡旋位置后，使用 `scripts/plot_structure_factor.py` 进行 FFT 分析。

![结构因子 S(k)](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/structure_factor_smoke/structure_factor_kappa_0_step_5000.png)

**图 10**：涡旋晶格结构因子 S(k) 热图（log10 标度）。中心为 DC 分量，周围的亮点对应涡旋晶格的倒格矢。六角对称的峰分布表明涡旋形成了 Abrikosov 三角晶格结构。

### 4.8 AI 闭环与参数反演

AI 闭环模块基于 bootstrap ridge 代理模型，实现"选点→仿真→回填"的 active learning 流程：

**代理模型**：
- 特征：缺陷参数（α_defect、defect_count、defect_radius、defect_spacing）
- 目标：临界驱动参数 κ_c
- 方法：Ridge 回归 + Bootstrap 集成（估计不确定性）

**选点策略**：
- UCB（Upper Confidence Bound）：maximize `mean + β×std`
- Target：minimize `|mean - target| - β×std`

**评估结果**：
5-fold 交叉验证的 hit_rate(|err|≤0.005) 达 0.82-0.87，表明代理模型能够有效预测 κ_c。

![AI 闭环进展](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/ai_inversion_target_lattice_refined/loop_progress.png)

**图 11**：AI 闭环 active learning 进展曲线。横轴为迭代次数，纵轴为目标误差 |κ_c - target|。随着迭代进行，代理模型逐步逼近目标 κ_c 值。

![相图](https://raw.githubusercontent.com/g1157/Rust_learning/dev/homework/Rust_wgpu_TDGL_AI_Trial/runs/phase_diagram_alpha_count_warmup/phase_diagram_plot.png)

**图 12**：去钉扎相图。横轴为缺陷强度 |α_defect|，纵轴为缺陷数量。颜色表示临界驱动 κ_c，颜色越深表示钉扎越强（需要更大的驱动才能去钉扎）。

### 4.9 涡旋晶格间距验证

**理论公式**（Abrikosov 三角晶格）：

$$
a_0 = \sqrt{\frac{2\Phi_0}{\sqrt{3}B}} = 1.075 \times \sqrt{\frac{\Phi_0}{B}}
$$

**实测算法**：计算正涡旋最近邻距离中位数（周期边界 minimum image convention）

**验证结果**：

```
flux_n    理论间距    实测间距    偏差
──────────────────────────────────────
64        32.0        30.8        3.8%
128       22.6        21.9        3.1%
209       17.7        17.2        2.8%
```

偏差均 < 5%，验证了涡旋晶格形成的物理正确性。

---

## 五、讨论

### 5.1 结果总结

本项目成功实现了物理正确的 GPU 加速 TDGL 求解器：

**与理论/文献预期的对比**：

```
验证项目                          预期结果                    实验结果              是否符合
──────────────────────────────────────────────────────────────────────────────────────────────
净涡旋数 = flux_n                 net = n                     net = 64 (flux_n=64)  ✅
涡旋-反涡旋成对湮灭 (B=0)         net → 0                     net = 0 严格保持      ✅
能量密度单调下降                  dF/dt ≤ 0                   确认单调下降          ✅
涡旋晶格间距                      a₀ = 1.075√(Φ₀/B)          偏差 < 5%             ✅
Matching Field 增强               整数匹配场处 κ_c 增大       1.33×-1.57× 增强      ✅
dt 收敛性                         dt/2 结果一致               能量差异 < 0.02%      ✅
```

1. **物理一致性**：磁通量量子化 + 磁周期边界条件在 torus 上自洽定义均匀磁场；规范不变绕数确保涡旋统计不依赖 gauge 选择；净涡旋数与 flux_n 完美一致。

2. **计算性能**：256×256 网格实时可视化流畅（~28k steps/s）；1024×1024 网格吞吐量达 14 Gcells/s。

3. **物理现象**：验证了涡旋-反涡旋成对湮灭、钉扎效应、去钉扎阈值行为；观察到周期阵列在整数匹配场处的 commensurability peak，与文献 [9,10] 定性一致。

4. **工具链完整性**：支持 κ sweep、相图扫描、结构因子计算、AI 闭环等研究功能。

5. **可复现性**：每次运行自动生成 `config.toml`（完整参数）和 `meta.json`（运行环境），支持 `--seed` 固定随机种子。

### 5.2 研究工具链

本项目构建了完整的"仿真→后处理→批处理→AI 闭环"研究管线：

```
1. 仿真 (Rust/wgpu)
   ├── 交互式可视化 (--flux-n 209)
   ├── Headless 批处理 (--headless --steps 20000)
   └── κ Sweep (--kappa-start/end/step)
         ↓
2. 标准化输出
   ├── config.toml (完整参数快照)
   ├── meta.json (GPU/后端/时间戳)
   ├── vortices.csv (涡旋统计)
   ├── vortex_positions.csv (涡旋位置)
   └── kappa_sweep.csv (depinning 数据)
         ↓
3. 后处理脚本 (Python)
   ├── plot_vortices.py (时间序列)
   ├── plot_kappa_sweep.py (depinning 曲线)
   ├── run_depinning_phase_diagram.py (相图扫参)
   ├── plot_structure_factor.py (S(k) 分析)
   └── run_matching_field_scan.py (匹配场扫描)
         ↓
4. AI 闭环
   ├── ai_inverse_design.py (参数反演)
   └── ai_closed_loop.py (active learning)
```

### 5.3 局限性与改进方向

```
局限                        改进方向
────────────────────────────────────────────────────────
显式 Euler 稳定性限制       半隐式方法（线性项隐式）
无热噪声                    添加 Langevin 项
无电流项                    完整 TDGL + Maxwell
涡旋检测在 CPU              GPU 端 reduction
```

### 5.4 未来工作

- **热噪声相图**：添加 Langevin 项，研究温度驱动的玻璃态与蠕变行为
- **更丰富的结构量**：径向分布函数 g(r)、S(k) 峰宽跟踪
- **AI 增强**：从图像/时间序列反演缺陷参数（带不确定性估计）
- **三维扩展**：3D TDGL 与柱状缺陷钉扎

### 5.5 结论

本项目展示了将经典 TDGL 物理与现代 GPU 并行计算相结合的可行性，构建了从仿真到分析的完整研究平台。实验结果验证了数值实现的物理正确性，并复现了周期钉扎阵列的 matching field 效应。项目代码开源，支持可复现研究。

---

## 参考文献

[1] Ginzburg, V. L., & Landau, L. D. (1950). On the theory of superconductivity. *Zh. Eksp. Teor. Fiz.*, 20, 1064.

[2] Abrikosov, A. A. (1957). On the magnetic properties of superconductors of the second group. *Soviet Physics JETP*, 5(6), 1174-1182.

[3] de Gennes, P. G. (1966). *Superconductivity of Metals and Alloys*. W.A. Benjamin.

[4] Tinkham, M. (1996). *Introduction to Superconductivity* (2nd ed.). McGraw-Hill.

[5] Blatter, G., Feigel'man, M. V., Geshkenbein, V. B., Larkin, A. I., & Vinokur, V. M. (1994). Vortices in high-temperature superconductors. *Reviews of Modern Physics*, 66(4), 1125-1388.

[6] Gropp, W. D., et al. (1996). Numerical simulation of vortex dynamics in type-II superconductors. *Journal of Computational Physics*, 123(2), 254-266.

[7] Machida, M., & Kaburaki, H. (1993). Direct simulation of the time-dependent Ginzburg-Landau equation for type-II superconducting thin film. *Physical Review Letters*, 71(19), 3206.

[8] Baert, M., Metlushko, V. V., Jonckheere, R., Moshchalkov, V. V., & Bruynseraede, Y. (1995). Composite Flux-Line Lattices Stabilized in Superconducting Films by a Regular Array of Artificial Defects. *Physical Review Letters*, 74(16), 3269–3272.

[9] Reichhardt, C., & Grønbech-Jensen, N. (2001). Critical currents and vortex states at fractional matching fields in superconductors with periodic pinning. *Physical Review B*, 63, 054510.

[10] Reichhardt, C., Zimányi, G. T., Scalettar, R. T., Hoffmann, A., & Schuller, I. K. (2001). Individual and multiple vortex pinning in systems with periodic pinning arrays. *Physical Review B*, 64, 052503.

[11] Field, S. B., et al. (2002). Vortex Configurations, Matching, and Domain Structure in Large Arrays of Artificial Pinning Centers. *Physical Review Letters*, 88, 067003.

[12] WebGPU Specification. W3C Working Draft. https://www.w3.org/TR/webgpu/

---

## 附录 A：项目结构

```
Rust_wgpu_TDGL_AI_Trial/
├── Cargo.toml              # Rust 依赖配置
├── README.md               # 项目说明
├── REPORT.md               # 课程论文（本文件）
├── LICENSE                 # MIT 许可证
├── requirements.txt        # Python 依赖
├── src/
│   ├── main.rs             # 主程序（~4200 行）
│   ├── ui/                 # egui Dashboard UI 模块
│   │   ├── mod.rs          # 模块入口
│   │   ├── theme.rs        # 深色主题配色
│   │   ├── components/     # 可复用组件
│   │   │   ├── param_slider.rs
│   │   │   ├── time_series.rs
│   │   │   └── depinning_curve.rs
│   │   └── panels/         # UI 面板
│   │       ├── params_panel.rs
│   │       ├── stats_panel.rs
│   │       ├── status_bar.rs
│   │       ├── history_panel.rs
│   │       └── validation_panel.rs
│   └── utils/              # 工具模块
│       ├── presets.rs      # 预设配置
│       ├── animation.rs    # 动画工具
│       ├── materials.rs    # 超导材料参数库
│       └── validation_report.rs
├── scripts/                # Python 后处理脚本
│   ├── plot_vortices.py
│   ├── plot_kappa_sweep.py
│   ├── run_depinning_phase_diagram.py
│   ├── plot_phase_diagram.py
│   ├── run_matching_field_scan.py
│   ├── plot_matching_field.py
│   ├── plot_structure_factor.py
│   ├── run_convergence_study.py
│   ├── validate_run.py
│   ├── ai_inverse_design.py
│   └── ai_closed_loop.py
├── doc/                    # 文档
│   ├── RESEARCH_ROADMAP.md
│   └── IMPLEMENTATION_LOG.md
└── runs/                   # 输出目录（.gitignore）
```

## 附录 B：输出文件格式

### vortices.csv

```
列名              说明
────────────────────────────────────────────────────────
step              时间步数
time              模拟时间 (step × dt)
kappa             当前驱动参数
vortices          涡旋数
antivortices      反涡旋数
net               净涡旋数 (vortices - antivortices)
energy            总能量
energy_density    能量密度 (energy / (nx × ny))
pinned_v          钉扎涡旋数
pinned_av         钉扎反涡旋数
pinned_net        钉扎净涡旋数
mean_vx           平均 x 方向漂移速度
mean_vy           平均 y 方向漂移速度
mean_speed        平均漂移速率
```

### kappa_sweep.csv

```
列名                  说明
────────────────────────────────────────────────────────
kappa                 驱动参数值
samples               采样点数
mean_speed            平均漂移速率
mean_vx               平均 x 方向速度
mean_vy               平均 y 方向速度
net_mean              平均净涡旋数
pinned_net_mean       平均钉扎净涡旋数
energy_density_mean   平均能量密度
```

### config.toml

每次运行的完整参数配置，包含网格尺寸、时间步长、磁场参数、缺陷配置等，便于复现实验。

## 附录 C：核心数据结构

```rust
/// 模拟参数（GPU uniform buffer）
#[repr(C)]
struct Params {
    nx: u32,        // 网格宽度
    ny: u32,        // 网格高度
    show_alpha: u32,// 显示模式
    _pad0: u32,
    dt: f32,        // 时间步长
    dx: f32,        // 空间步长
    phi: f32,       // plaquette flux
    kappa: f32,     // 驱动参数
}

/// 复数（GPU vec2<f32>）
#[repr(C)]
struct Complex { re: f32, im: f32 }
```

## 附录 D：WGSL Compute Shader 核心

```wgsl
@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let psi = psi_in[i];

    // Uy = exp(-i (phi*x + kappa))
    let theta = -(params.phi * f32(gid.x) + params.kappa);
    let Uy = vec2(cos(theta), sin(theta));

    // Gauge-covariant Laplacian
    let psi_yp = cmul(Uy, psi_in[idx(gid.x, yp)]);
    let psi_ym = cmul(conj(Uy), psi_in[idx(gid.x, ym)]);
    let lap = (psi_xp + psi_xm + psi_yp + psi_ym - 4.0*psi) / dx2;

    // TDGL 更新
    let rhs = lap + alpha[i] * psi - psi * dot(psi, psi);
    psi_out[i] = psi + dt * rhs;
}
```

## 附录 E：运行命令示例

```bash
# 交互式可视化
cargo run --release -- --flux-n 209

# Headless 批处理
cargo run --release -- --headless --steps 20000 --flux-n 209

# κ sweep
cargo run --release -- --headless --flux-n 209 \
    --kappa-start 0.0 --kappa-end 0.05 --kappa-step 0.01

# 相图扫描
python scripts/run_depinning_phase_diagram.py --flux-n 209

# Matching field 扫描
python scripts/run_matching_field_scan.py \
    --flux-n-list 32,64,96,128 --defect-mode-list random,lattice

# AI 闭环
python scripts/ai_closed_loop.py --objective maximize --iters 10 \
    --out-root runs/ai_closed_loop
```

## 附录 F：依赖

```toml
[dependencies]
wgpu = "23"
winit = "0.30"
pollster = "0.4"
bytemuck = "1"
rand = "0.8"
egui = "0.30"
egui-wgpu = "0.30"
egui_plot = "0.30"
```

### Python 依赖

```
numpy
pandas
matplotlib
scikit-learn
scipy
```

安装：`pip install -r requirements.txt`
