# TDGL GPU Research Roadmap (Deep Analysis + Next Steps)

<!--
ASCII-only header (keep this block) to avoid a Windows apply_patch UTF-8 slicing bug.
Once tooling is fixed, this header can be simplified.
File encoding: UTF-8 (Chinese content starts below).
Last updated: 2025-12-18.
Synced with: flux quantization/MPBC, gauge-invariant winding, energy diagnostics, headless mode, kappa drive+sweep, pinned/velocity observables.
Doc note: AI inversion baseline + AI closed-loop runner implemented (scripts/ai_inverse_design.py, scripts/ai_closed_loop.py).
Doc note: default --out-dir is runs/<mode>_<unix_ms> (pass --out-dir . for legacy cwd output).
Doc note: repo hygiene: LICENSE + requirements.txt + .gitignore (target/, runs/).
Pad: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
Pad2: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
-------------------------------------------------------------------------------
-->

# TDGL GPU 平台：深度分析与研究路线图

> 面向：把当前项目从“能跑能看”升级为“能回答明确科学问题、可量化、可复现”的研究型工作  
> 适用仓库：`Rust_wgpu_TDGL_AI_Trial`  
> 关联文档：`README.md`、`REPORT.md`、`doc/IMPLEMENTATION_LOG.md`、`doc/Rust_wgpu_TDGL_AI_Trial_Doc.md`

---

## 0. 一句话结论（你下一步最该做什么）

当前工程已经具备“研究平台”雏形：GPU 并行 TDGL + 实时可视化 + 缺陷场 + 采样统计 + benchmark。要把它提升到**物理可信 + 有创新点**，最关键的第一步是把 **外磁场 + 周期边界**做成**全局一致**（磁通量量子化/磁周期边界），并把涡旋检测升级为**规范不变（gauge-invariant）**版本；随后围绕“去钉扎阈值/相图”“缺陷几何设计”“热噪声玻璃态”建立一套可复现实验与指标体系，最终可以自然扩展到 **AI 逆问题/逆向设计**。

---

## 1. 当前项目能力与架构（现状盘点）

### 1.1 已具备的研究能力

- **物理模型**：二维 gauge-covariant TDGL（Landau gauge）+ 空间变化缺陷/钉扎势 `α(r)`。
- **数值实现**：5 点 stencil 的协变拉普拉斯 + 显式 Euler 时间推进。
- **GPU 端到端链路**：compute shader 更新 `ψ`（ping-pong）→ fragment shader 直接采样渲染 `|ψ|` / `α`（无 CPU 每帧回读）。
- **统计与导出**：周期性将 `ψ` 回读到 CPU，使用相位绕数算法计数涡旋/反涡旋，输出 `vortices.csv`。
- **性能基准**：`--bench` 无窗口模式对不同网格规模测 steps/s 与 cells/s。

### 1.2 参数一览（便于写实验表）

| 类别 | 参数 | 含义 | 当前默认值（代码） |
|---|---|---|---|
| 网格 | `NX, NY` | 网格尺寸 | 256×256 |
| 时间 | `dt` | 时间步长 | 0.01 |
| 空间 | `dx` | 空间步长 | 1.0 |
| 外场 | `B` | 外磁场强度（Landau gauge） | 0.02 |
| 缺陷 | `α_default` | 超导区材料参数 | 1.0 |
| 缺陷 | `α_defect` | 缺陷区材料参数 | -0.5 |
| 缺陷 | `count/radius` | 缺陷数量/半径 | 50 / 3 |
| GPU | `workgroup` | workgroup size | 8×8 |
| 采样 | `period` | 涡旋检测周期 | 100 steps |

> 上表建议在你后续写 `config.toml` 时一字不落地保留（见第 7 节）。

### 1.3 数据流/控制流（建议你后续所有改动都不破坏它）

```
        ┌───────────────┐
        │   Params/α(r)  │  (CPU 初始化/配置)
        └───────┬───────┘
                │ write_buffer
┌───────────────▼────────────────┐
│  GPU Compute: TDGL update step  │  ping-pong ψA/ψB
└───────────────┬────────────────┘
                │
                │ (可选：每 N 步 copy 一次)
                │
      ┌─────────▼─────────┐
      │  GPU Readback Buf   │  map_async + poll
      └─────────┬─────────┘
                │
┌───────────────▼────────────────┐
│ CPU: vortex detect + CSV log    │
└───────────────┬────────────────┘
                │
┌───────────────▼────────────────┐
│ GPU Render: sample ψ/α → colormap│  交互切换显示
└─────────────────────────────────┘
```

后续研究功能（驱动项、热噪声、追踪、结构因子、GPU 端统计）建议尽量沿这条管线“插拔式”加入，避免把工程改成不可维护的一坨。

---

## 2. 物理模型与离散化（写论文/报告要用）

### 2.1 当前无量纲化 TDGL（含矢势）

$$
\partial_t \psi = (\nabla - i\mathbf A)^2\psi + \alpha(\mathbf r)\psi - |\psi|^2\psi
$$

- $\psi$：复数序参（超导序参量）
- $\alpha(\mathbf r)$：材料/缺陷场（钉扎通过局部降低 $\alpha$ 实现）
- $\mathbf A$：外磁场对应矢势（Landau gauge：$\mathbf A=(0,Bx,0)$）

### 2.2 离散化要点（协变拉普拉斯 + link 变量）

在格点上用 link 变量保证规范协变（以 $y$ 方向为例）：

$$
U_y(x)=e^{-i B x\,\Delta x}
$$

$$
(\nabla-iA)^2\psi \approx
\frac{\psi_{x+}+\psi_{x-}+U_y\psi_{y+}+U_y^*\psi_{y-}-4\psi}{(\Delta x)^2}
$$

时间推进（显式 Euler）：

$$
\psi^{n+1}=\psi^n+\Delta t\;F(\psi^n)
$$

---

## 3. “深水区”问题（决定你是否真正做对了物理）

下面三项是最建议优先解决的“研究可信度门槛”。做完它们，你的项目会从“课程作业”直接上升到“能写出像样科研小论文”的可信度。

### 3.1 外磁场 + 周期边界的全局一致性（磁通量量子化 / 磁周期边界）

#### 3.1.1 为什么这是硬门槛

你目前使用 Landau gauge：$A_y=Bx$，但在周期边界上：

$$
A_y(x+L_x)=A_y(x)+B L_x
$$

这意味着 $A$ 本身不周期；要在环面（torus）上定义均匀磁场，需要：

1. **磁通量量子化**（最省事的修复方式），或  
2. **磁周期边界条件（Magnetic periodic / twisted BC）**（更一般、更“正统”）。

否则会出现“局部看起来有 B，但全局拓扑量不对”的现象，典型症状就是：在 $B\neq 0$ 的情况下，你统计到的净涡旋数 `net=vort-antivort` 仍长期接近 0（这在很多初学实现里都会出现）。

#### 3.1.2 方案 A（推荐起步）：磁通量量子化，把 B 选成“合法值”

在离散环面上，总磁通满足：

$$
\Phi = B\,(N_x\Delta x)(N_y\Delta x)=2\pi n,\;\; n\in \mathbb Z
$$

因此：

$$
B=\frac{2\pi n}{N_x N_y (\Delta x)^2}
$$

以默认 $N_x=N_y=256,\Delta x=1$ 为例：
- 你现在的 $B=0.02$ 对应 $n\approx 208.6$（非整数）
- 取 $n=209$ 得 $B\approx 0.0200376$
- 取 $n=208$ 得 $B\approx 0.0199418$

**验证标准**：在无缺陷或弱缺陷下，长时间演化后应满足

$$
N_{\mathrm{net}} \equiv N_v - N_{av} \approx n
$$

并且涡旋密度满足 $n_v \approx B/(2\pi)$（无量纲）。

> 实操建议：后续扫外场用整数 `n` 做自变量（见附录 A），这样实验更“干净”，结论更好写。

#### 3.1.3 方案 B：磁周期边界（更一般，适合后续做驱动/扭曲）

在 Landau gauge 的 lattice 实现里，通常需要在边界 link 上补一个“缝合相位”来保持 plaquette 磁通一致。一个常见构造：

- 内部：$U_x(x,y)=1$，$U_y(x,y)=e^{-iB x\Delta x}$
- **仅在 $x=L_x-\Delta x$ 的边界**设置：

$$
U_x(L_x-\Delta x, y)=e^{+iB L_x\, y}
$$

直观理解：Landau gauge 在 $x$ 方向绕一圈会多出一个 gauge 变换，相当于波函数要在边界“扭一下”才能把环面缝合起来。

**验证标准 1（局部）**：计算每个 plaquette 的 gauge-invariant 磁通

$$
\phi_{x,y}=\arg\left(U_x(x,y)\,U_y(x+\Delta x,y)\,U_x^*(x,y+\Delta x)\,U_y^*(x,y)\right)
$$

应当 $\phi_{x,y}\approx -B(\Delta x)^2$ 且空间上均匀。

**验证标准 2（全局）**：总磁通满足 $\sum\phi_{x,y}=2\pi n$，并且 $N_{\mathrm{net}}\approx n$。

#### 3.1.4 离散实现对照（推荐用“plaquette flux”写清单位）

为了避免 $\Delta x \neq 1$ 时单位混乱，建议明确区分：

- 物理磁场（连续）$B_{\text{phys}}$
- 格点上每个 plaquette 的无量纲磁通（更适合写进代码）

$$
\varphi \equiv B_{\text{phys}}(\Delta x)^2
$$

在索引空间 $(i,j)$（$i=0..N_x-1,\;j=0..N_y-1$）上，经典的 torus 均匀磁场 link 构造可写为：

- $U_y(i)=\exp(-i\,\varphi\,i)$
- $U_x(i,j)=1$（内部）
- $U_x(N_x-1,j)=\exp(+i\,\varphi\,N_x\,j)$（只在 $x$ 边界补缝合相位）

磁通量量子化条件变为：

$$
\varphi\,N_xN_y=2\pi n
$$

也就是：

$$
\varphi(n)=\frac{2\pi n}{N_xN_y}
$$

> 这条写法的好处：当你做网格收敛（改 $\Delta x$）时，代码仍可用同一个 $\varphi$ 表示“同一总磁通”；对应的 $B_{\text{phys}}=\varphi/(\Delta x)^2$。

**协变拉普拉斯在 x 方向的边界项（实现提示）**  
对于 $D_x\psi$，应使用

$$
\psi_{x+}=U_x(i,j)\,\psi(i+1,j),\;\;
\psi_{x-}=U_x^*(i-1,j)\,\psi(i-1,j)
$$

其中 $i\pm 1$ 需要做周期 wrap；当 wrap 发生时，$U_x$ 就不再恒等于 1（正是上面的边界补相位在起作用）。

---

### 3.2 有矢势时的涡旋检测：必须用规范不变（gauge-invariant）绕数

#### 3.2.1 为什么当前“相位绕数”在 A≠0 时不够严格

直接对 $\theta=\arg(\psi)$ 做绕数，本质上在数值上依赖一个“选定的 gauge”。当 $A\neq 0$ 时，更稳妥的做法是基于 link 变量计算边相位差，等价于离散的 $\nabla\theta-\mathbf A$。

#### 3.2.2 规范不变绕数（建议作为研究结果的标准做法）

对每个网格元（四角点 $\psi_{00},\psi_{10},\psi_{11},\psi_{01}$），定义边上的 gauge-invariant 相位差：

$$
\Delta\theta_x(x,y)=\arg\left(\psi^*(x,y)\,U_x(x,y)\,\psi(x+\Delta x,y)\right)
$$

$$
\Delta\theta_y(x,y)=\arg\left(\psi^*(x,y)\,U_y(x,y)\,\psi(x,y+\Delta x)\right)
$$

然后绕一圈求和并做 unwrap：

$$
W=\mathrm{wrap}\left(
\Delta\theta_x(x,y)+\Delta\theta_y(x+\Delta x,y)-\Delta\theta_x(x,y+\Delta x)-\Delta\theta_y(x,y)
\right)
$$

判断 $W\approx \pm 2\pi$ 即为涡旋/反涡旋。

**建议输出的三个量**（研究报告更完整）：
- $N_v(t)$、$N_{av}(t)$、$N_{\mathrm{net}}(t)$
- 以及 $N_{\mathrm{net}}$ 与 $B(n)$ 的一致性（见 3.1）。

#### 3.2.3 实现伪代码（CPU 版，便于先对照验证）

> 核心：不用单独求 $\arg(\psi)$ 再差分，而是直接算边上的复数“并行运输”内积，再取 `atan2(im,re)`。

```text
for each cell (i,j):
  ψ00 = ψ(i,  j)
  ψ10 = ψ(i+1,j)
  ψ11 = ψ(i+1,j+1)
  ψ01 = ψ(i,  j+1)

  e0 = arg( conj(ψ00) * Ux(i,  j) * ψ10 )
  e1 = arg( conj(ψ10) * Uy(i+1,j) * ψ11 )
  e2 = arg( conj(ψ11) * conj(Ux(i, j+1)) * ψ01 )   # 注意方向与共轭
  e3 = arg( conj(ψ01) * conj(Uy(i, j))   * ψ00 )

  W = wrap(e0 + e1 + e2 + e3)
  if W > 0.75*2π: vort++
  if W < -0.75*2π: anti++
```

实现细节建议：

- `arg(z)` 用 `atan2(z.im, z.re)`，不需要归一化（只要不是精确 0，方向都定义良好）。
- `wrap` 把角度压回 $(-\pi,\pi]$，减少跨越 $\pm\pi$ 的跳变。
- 阈值用 `0.75*2π` 比 `π` 更抗噪，避免把数值波动当涡旋。

---

### 3.3 数值可信度：显式 Euler 只是起点，必须补“收敛性 + 耗散性”证据

#### 3.3.1 为什么要做收敛性（不是形式主义）

显式 Euler 的稳定性窗口很小（你目前 dt 满足 CFL 类条件并不代表“误差小”）。如果你报告里给出了阈值/相图，但没有 `dt/dx` 收敛性证据，结论在严格意义上是不可信的。

#### 3.3.2 需要补齐的两类证据（最低配）

1) **时间步收敛**  
固定一组物理参数（同一 `n`、同一缺陷），比较
- `dt` 与 `dt/2` 时稳态统计量的差异（例如稳态 $N_{\mathrm{net}}$、$F(t)$ 末值、结构因子主峰高度）

2) **网格/有限尺寸效应**  
比较 $256^2 \rightarrow 512^2$。为了“物理尺寸一致”，推荐保持 $L=N\Delta x$ 不变（即网格变细时同步调整 $\Delta x$）。

#### 3.3.3 耗散泛函单调性（强烈建议加入监测）

无噪声、无外驱的 TDGL 是梯度流，连续理论下自由能 $F$ 应随时间下降（数值误差会破坏，但应“大体单调”）。

一个常用无量纲能量泛函（示意）：

$$
F=\int d^2r\left(|(\nabla-iA)\psi|^2-\alpha|\psi|^2+\frac{1}{2}|\psi|^4\right)
$$

离散后用 link 变量构造 $|D\psi|^2$ 并每隔若干步统计一次 $F(t)$，能显著提升“物理可信度”。

#### 3.3.4 时间推进升级路线（按投入产出比排序）

1) **RK2 / Heun（中点法）**：实现简单，误差从一阶升到二阶，仍然显式；在“同等可信度”下可允许更大 dt。  
2) **半隐式（线性项隐式、非线性显式）**：把拉普拉斯项用少量 Jacobi/Gauss-Seidel 迭代近似求解，可显著放宽稳定性限制；非常适合 GPU（固定迭代次数）。  
3) **谱方法/ETD（更硬核）**：若你未来把线性部分搬到 FFT（周期边界天然适配），可以做到非常稳定且高效，但工程投入更大。

---

## 4. 观测量体系：把“看到涡旋”变成“测到涡旋”

下面给出一套从易到难的观测量清单，建议你按“研究问题”选 2–4 个做深入，而不是全部都做。

### 4.1 基础拓扑量（必做）

- **净涡旋数**：$N_{\mathrm{net}}=N_v-N_{av}$  
  用于验证磁通量量子化、外场正确性、周期拓扑守恒等。
- **涡旋密度**：$n_v=N_{\mathrm{net}}/(L_xL_y)$  
  与 $B/(2\pi)$ 的关系是外场自洽的重要检验。

### 4.2 结构量（决定你能否说“晶格/无序/玻璃”）

- **结构因子**（vortex positions 的 Fourier 统计）：

$$
S(\mathbf k)=\frac{1}{N}\left|\sum_{j=1}^{N}e^{i\mathbf k\cdot \mathbf r_j}\right|^2
$$

三角晶格会出现六角对称峰；无序/玻璃态峰会变宽或消失。

- **径向分布/相关函数** $g(r)$：描述局域有序与尺度。

### 4.3 动力学量（去钉扎与输运的核心）

- **涡旋位置追踪**：从每次检测到的涡旋中心 $\mathbf r_j(t)$ 构建轨迹  
  实现上可用“最近邻匹配 + 最大速度阈值”的简化追踪（足够用于课程级研究）。
- **平均漂移速度**：

$$
\langle v\rangle=\left\langle \frac{|\mathbf r(t+\Delta t)-\mathbf r(t)|}{\Delta t}\right\rangle
$$

- **速度-驱动力曲线** $\langle v\rangle(F)$ 或 $\langle v\rangle(J)$：定义去钉扎阈值 $F_c$（或 $J_c$）。

### 4.4 钉扎量（让“缺陷”变成可量化因果）

- **被钉扎比例**：定义缺陷区域集合 $\Omega_p$，统计涡旋在 $\Omega_p$ 内的比例
- **钉扎能/有效势**：可用 $|ψ|$ 的局部抑制或能量密度对比量化缺陷影响

---

## 5. 研究方向（创新性 + 可落地）与最小可交付成果

> 建议主线：先把 **外场自洽 + 规范不变检测 + 收敛/耗散证据**做扎实，然后围绕一个“相图问题”深入。

### 方向 A：外场自洽验证 + Abrikosov 晶格与结构因子（硬核可信）

**研究问题**：在周期环面上，外场 $B$ 与净涡旋数、涡旋晶格序之间是否一致？缺陷如何破坏/锁定晶格？

**创新点**：把工程实现提升到“拓扑/规范一致”，这是数值超导模拟的硬门槛；并用结构因子给出“有序/无序”的定量判据。

**最小交付**：
- 选择量子化的 $B(n)$，验证稳态 $N_{\mathrm{net}}\approx n$
- 无缺陷 vs 随机缺陷下对比 $S(\mathbf k)$ 峰形与峰位

**风险与对策**：
- 若长期难以形成晶格：检查耗散泛函、数值噪声、初值与退火策略（逐步升 B 或逐步降低噪声）。

### 方向 B：去钉扎（depinning）阈值曲线与动力学相图（论文感最强）

**研究问题**：缺陷强度/密度/相关长度如何改变临界驱动（等效临界电流）以及涡旋流动形态？

**关键是驱动项的实现**（按复杂度从低到高）：

1) **相位扭曲/等效常量矢势**（推荐起步）  
   在周期边界上施加相位 twist（或加入常量 $A_{\mathrm{drive}}$），产生持久超流，从而对涡旋产生驱动。

2) **显式电场/标势（更正统）**  
   在 TDGL 中引入标势 $\phi$ 或时间依赖矢势，形成更贴近输运的驱动框架（实现更复杂，但研究深度更高）。

**落地方案（推荐起步）：在 $U_y$ 中加入常量 twist $\kappa$（等效外加超流）**

把第 3.1.4 节的 plaquette flux 记为 $\varphi$，外场 link 为 $U_y(i)=e^{-i\varphi i}$。在周期环面上加入一个常量 $A_{y,0}$（或相位扭曲）等价于：

$$
U_y(i)\;\leftarrow\;\exp\left(-i(\varphi i+\kappa)\right)
$$

直观含义：$\kappa$ 相当于在 $y$ 方向叠加一个全局相位梯度/扭曲边界，从而产生持久超流；超流对涡旋产生洛伦兹力并驱动其运动（在很多数值文献中这是做 depinning 的常用最小实现）。

**建议的实验流程（最省事且结果最好写）**：

1. 固定缺陷参数与外场（用整数 `n`），先用 $\kappa=0$ 演化到稳态  
2. 以小步长扫描 $\kappa$：$\kappa_0,\kappa_0+\Delta\kappa,\dots$（每个点都给足够的松弛时间）  
3. 每个 $\kappa$ 点测量平均漂移速度 $\langle v\rangle$（见第 4.3 节）  
4. 定义阈值：当 $\langle v\rangle>v_{\min}$ 持续若干采样窗口后，记为 depinned；得到 $\kappa_c$（或换算成等效 $F_c/J_c$）

> 关键是“缓慢 ramp + 去掉瞬态”：否则你测到的是启动瞬态而不是稳态输运。

**观测量**：
- $\langle v\rangle(F)$、$F_c$（阈值）
- 流动形态：通道流/塑性流/整体滑移（可用轨迹可视化 + 速度分布）

**最小交付**：
- 做一个 2D 相图：`缺陷强度 × 缺陷密度 → F_c`
- 并给出一条典型的 $\langle v\rangle-F$ 曲线

### 方向 C：缺陷几何设计与“匹配场”（结构设计带来的反直觉现象）

**研究问题**：周期阵列/准周期阵列/条纹缺陷是否在特定 $B$（匹配场）显著提高钉扎能力？

**创新点**：把缺陷从“随机点”升级为“可设计结构”，形成明显可发表式结论：在匹配条件下 $J_c$ 提升、晶格锁定等。

**最小交付**：
- 同缺陷密度下：随机 vs 周期阵列对比 $F_c$ 与 $S(\mathbf k)$
- 扫 $B$（或等效 $n$）观察“峰”（matching peak）

### 方向 D：热噪声、玻璃态与蠕变（统计物理深度）

**研究问题**：温度噪声如何导致热激活解钉扎、蠕变律与结构无序化？

**实现提示**：
- 在 TDGL 更新中加入 Langevin 噪声 $\eta(\mathbf r,t)$  
  $\langle\eta\rangle=0,\;\langle\eta^*(r,t)\eta(r',t')\rangle\propto T\,\delta_{rr'}\delta_{tt'}$
- 做 $T$-sweep 得到“相图”：有序晶格 → 玻璃/液体

**最小交付**：
- $F_c(T)$ 或 $\langle v\rangle(T)$ 曲线
- $S(\mathbf k)$ 随 $T$ 的峰展宽

### 方向 E：AI 逆问题/逆向设计（最具“跨领域创新”潜力）

> 你已经有一个 GPU 端到端的“数据生成器”，非常适合做 AI 闭环。

**E1：参数反演（inverse problem）**  
输入：最终态或时间序列的 $|ψ|$ 图/涡旋统计；输出：$B$、缺陷密度、缺陷强度、温度等。  
价值：贴近实验图像/统计的参数估计与不确定性评估。

**E2：缺陷景观逆向设计（pinning landscape design）**  
目标：在给定缺陷预算（面积/数量/强度限制）下最大化 $F_c$（或等效 $J_c$）。  
方法可选：贝叶斯优化/进化策略/强化学习/代理模型（surrogate）+ 仿真闭环。

**最小交付**（可先做课程级版本）：
- 用扫参生成小数据集（几十到几百条）
- 训练一个轻量模型预测 $F_c$
- 用优化算法在模型上找候选缺陷分布，再回到 TDGL 仿真验证

**当前实现（baseline，可离线运行）**：
- `scripts/ai_inverse_design.py`：对 `phase_diagram.csv` 做 ridge 回归代理模型，并用离散网格搜索做反演/逆向设计（Stage 3 的最小闭环版本）

**数据集与基线建议（让 AI 部分更“像研究”）**：

- 输入表示：
  - `α(r)`：二维图像（或下采样到 64^2/128^2 以降低训练成本）
  - 外场：用整数 `n`（或用 $\varphi$）编码，避免“非法 B”
  - 温度/驱动：$T,\kappa$ 等作为标量输入
- 标签设计（推荐做 depinning）：
  - 用第 5 方向 B 的扫描得到 $\kappa_c$，作为监督学习目标
  - 或预测整条 $\langle v\rangle(\kappa)$ 曲线的关键参数（阈值/斜率）
- 模型基线：
  - CNN/ViT：`α(r)` 图像 + 拼接标量参数
  - 简单代理：先用手工特征（缺陷密度、相关长度、强度分布）+ MLP 作为对照
- 评估方式：
  - 留一法：固定某些 `n` 或缺陷几何作为 OOD 测试（检验泛化）
  - 物理一致性检查：预测的 $\kappa_c$ 随缺陷强度/密度应单调或近似单调（作为 sanity check）

---

## 6. 推荐里程碑（按“最小可发表/可写深度报告”的节奏）

### 阶段 1：可信度加固（优先级最高）

- [x] 把 $B$ 改为量子化的 $B(n)$（或实现磁周期边界 link 修正）
- [x] 涡旋检测改为 gauge-invariant 版本（基于 link）
- [x] 加入收敛性实验：`dt`、`dx`（至少 2 组对比；见 `scripts/run_convergence_study.py` + `runs/convergence_dt_flux64_smoke/`、`runs/convergence_dx_smoke/`）
- [x] 加入耗散泛函 $F(t)$ 的监测与输出（每 N 步一次）

**阶段产出**：`N_net ≈ n`、`F(t)` 下降、收敛图（这三张图能显著提高“可信度”）。

### 阶段 2：选一条主线做“相图”

二选一（建议优先 B，再扩展到 C）：
- [x] 先行准备：加入驱动 $\kappa$ 与漂移速度观测量（为 depinning 扫参提供 order parameter）
- [x] κ sweep 自动化：`--kappa-start/--kappa-end/--kappa-step` + `kappa_sweep.csv`
- [x] 相图扫参工具：`scripts/run_depinning_phase_diagram.py`（提取 `kappa_c` 汇总到 `phase_diagram.csv`）
- [x] `kappa_c` 提取策略：`--order-parameter`（推荐 `abs_mean_vx`） + `--kappa-c-method`（baseline_threshold/two_phase_fit）
- [x] 去钉扎阈值 $F_c(V_p,n_p,r_p)$ 相图（流程闭环：扫参→汇总→绘图；物理口径可继续迭代）
- [x] 周期缺陷阵列支持（square lattice）：`--defect-mode lattice` + `--defect-spacing`
- [x] 匹配场/缺陷几何对比（随机 vs 周期阵列）：`scripts/run_matching_field_scan.py` + `scripts/plot_matching_field.py`

**阶段产出**：一张相图 + 一张典型动力学曲线（$\langle v\rangle-F$ 或 $F_c-B$）。

### 阶段 3：创新扩展（可选，但非常加分）

- [ ] 热噪声相图（$T$-sweep）
- [x] 结构因子 S(k) 工具链：`--dump-positions` + `scripts/plot_structure_factor.py`
- [x] AI 反演或 AI 逆向设计闭环（baseline：`scripts/ai_inverse_design.py` + `phase_diagram.csv`）
- [x] AI 闭环（active learning）：`scripts/ai_closed_loop.py`（代理模型 + acquisition 选点 + 自动仿真回填）

---

## 7. 可复现性与实验自动化（建议写成“研究框架”）

建议每次实验输出一个目录，包含：

- `config.toml`：所有参数（Nx, Ny, dt, dx, n/ B, 缺陷参数, 驱动参数, 温度, seed…）
- `meta.json`：运行时间、GPU/后端信息、git commit hash、wgpu 版本等
- `vortices.csv`：统计数据
- `observables.csv`：能量/结构因子峰值/速度等
- `snapshots/`：可选，保存若干帧 `|ψ|` 图或涡旋位置

核心原则：**一次运行 = 一组可被别人复现的实验记录**。

**当前项目已实现（最小可复现闭环）**：
- [x] `--out-dir`：每次实验输出到独立目录（默认 `runs/<mode>_<unix_ms>`）
- [x] `config.toml`：完整参数快照（自动写入 out-dir）
- [x] `meta.json`：GPU/后端/argv/时间戳（自动写入 out-dir）
- [x] `vortices.csv`/`vortex_positions.csv`：带 `kappa` 列的统计与位置数据
- [x] `kappa_sweep.csv`：用于提取 `kappa_c` 与绘制 depinning 曲线

---

## 8. 性能与工程深度（把 benchmark 变成“可解释的剖析”）

你已经有吞吐 benchmark，建议进一步把它变成“研究结果”：

- **workgroup 扫描**：8×8、16×16、32×8… 对 steps/s 的影响（解释是算力还是带宽瓶颈）
- **减少 trig 成本**：`Uy=exp(-iBxΔx)` 目前每格点计算 `sin/cos`；可尝试预计算 `Uy[x]` 缓冲区（用带宽换算力），并用 benchmark 量化收益
- **统计搬到 GPU**：把涡旋检测（或至少计数）搬到 GPU，只回读计数/稀疏结果，显著减少 map/unmap 与带宽消耗

这些内容写进报告，会让“GPU 加速”章节从“跑得快”升级成“我知道为什么快、瓶颈在哪里、怎么优化”。

---

## 9. 验证清单（建议你每次做大改动都跑一遍）

> 这些不是“测试用例”，而是研究数值模拟里常用的 sanity checks。通过它们，你可以很快定位是“物理没做对”还是“代码实现 bug”。

- **(V1) B=0，无缺陷**：长期应满足 `N_net≈0`；涡旋-反涡旋对会湮灭并衰减到很低水平。
- **(V2) B=0，有缺陷**：`N_net≈0` 仍应成立，但稳态会残留一定数量的涡旋对并在缺陷处被钉扎。
- **(V3) B=量子化（n 为整数），无缺陷**：稳态 `N_net≈n`；结构因子应出现明显有序峰（理想情况下趋向 Abrikosov 晶格）。
- **(V4) B=量子化，有缺陷**：`N_net≈n` 仍应保持；缺陷会使峰展宽/产生锁定结构。
- **(V5) 收敛性**：把 `dt` 减半后，稳态统计量变化应显著变小；否则需要降低 dt 或升级时间推进。

---

## 附录 A：默认参数下的量子化 B 快速表（Nx=Ny=256, dx=1）

$$
B(n)=\frac{2\pi n}{65536}
$$

| n | B(n) |
|---:|---:|
| 200 | 0.0191747598 |
| 208 | 0.0199417502 |
| 209 | 0.0200376240 |
| 210 | 0.0201334978 |

> 建议后续外场 sweep 用 `n` 做自变量，而不是直接用任意小数的 `B`。

## 附录 B：离散能量泛函的一个可用版本（用于耗散性监测）

> 这不是唯一正确写法，但足够作为“数值耗散诊断”。关键是：梯度项必须用同一套 link 变量构造。

设 $\psi_{i,j}$ 为复数，$U_x,U_y$ 为 link，则可定义（示意）：

$$
F \approx \sum_{i,j}\Big(
|U_x(i,j)\psi_{i+1,j}-\psi_{i,j}|^2
+|U_y(i,j)\psi_{i,j+1}-\psi_{i,j}|^2
-\alpha_{i,j}|\psi_{i,j}|^2
+\frac{1}{2}|\psi_{i,j}|^4
\Big)
$$

其中 $i+1,j+1$ 都按周期 wrap。输出 $F(t)$（或单位面积能量密度 $F/(L_xL_y)$）可作为：

- 时间推进是否“过于激进”的诊断（若频繁上升，dt 可能太大）
- 驱动/噪声打开后，系统是否进入稳态的判据（$F$ 统计稳定）
