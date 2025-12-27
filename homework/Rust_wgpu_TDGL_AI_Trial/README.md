<!--
ASCII-only header (keep this block) to avoid a Windows apply_patch UTF-8 slicing bug.
File encoding: UTF-8.
Last updated: 2025-12-27.
 Synced with: CLI flags (--flux-n/--b/--dt/--dx), magnetic periodic BC, gauge-invariant winding, energy diagnostics, drive kappa, pinned/velocity observables, out-dir, optional vortex position dump, kappa sweep, analysis scripts.
Doc note: vortices.csv now includes energy columns.
Doc note: add --seed for reproducible init/defects; vortices.csv has comment metadata lines.
Doc note: vortices.csv/vortex_positions.csv now include a per-row kappa column.
Doc note: kappa sweep writes kappa_sweep.csv (see Output Files).
Doc note: kappa sweep supports --kappa-initial-relax-steps (warm-up for the first kappa point).
Doc note: AI inversion baseline script: scripts/ai_inverse_design.py (ridge surrogate + grid-search inversion).
Doc note: AI closed loop runner script: scripts/ai_closed_loop.py (bootstrap surrogate + acquisition + simulation backfill).
Doc note: matching field scan scripts: scripts/run_matching_field_scan.py + scripts/plot_matching_field.py.
Doc note: structure factor (S(k)) script: scripts/plot_structure_factor.py (requires --dump-positions).
Doc note: README refreshed with full project summary + research workflow.
Doc note: reproducible smoke outputs live under runs/*_smoke (optional).
Doc note: refined runs: runs/convergence_dx_flux64_refined, runs/finite_size_refined, runs/phase_diagram_ai_eval_128, runs/ai_inversion_target_lattice_refined.
Doc note: ai_closed_loop objective=target progress plot shows |kappa_c-target|.
Doc note: default --out-dir is runs/<mode>_<unix_ms> (pass --out-dir . for legacy cwd output).
Doc note: Python deps are listed in requirements.txt (pip install -r requirements.txt).
Doc note: LICENSE file added (MIT).
Pad: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
Pad2: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-->

# Rust + wgpu TDGL 超导涡旋模拟

课程期末作业论文见 `REPORT.md`（摘要/介绍/正文/讨论/参考文献）。

基于 Rust + wgpu(WebGPU) 的 GPU 加速二维 TDGL 方程求解器，用于研究超导涡旋与钉扎现象（交互可视化 + headless 批处理 + 数据/脚本/AI 闭环）。

## 功能特性

- GPU 并行求解 TDGL 方程（compute shader）
- 实时可视化 |ψ| 热力图（无 CPU 回读）
- **egui Dashboard UI**：交互式参数控制、实时统计、时间序列图表
- 空间变化的钉扎势 α(r)
- 缺陷几何：随机缺陷 vs 周期缺陷阵列（matching field）
- 涡旋检测与统计（规范不变绕数 / gauge-invariant winding）
- 能量泛函诊断输出（energy / energy_density）
- κ 驱动与 κ sweep（去钉扎曲线、自动提取 κ_c）
- 批处理脚本：相图 / matching field / 结构因子 S(k)
- AI 工具链：反演（baseline）+ 闭环 active learning
- 材料预设库（NbSe2、YBCO、MgB2 等超导材料参数）
- 仿真验证报告生成
- 性能基准测试

## 项目总结（研究闭环）

本项目已经形成“研究型”闭环管线：

1. **仿真**（Rust/wgpu）：交互或 `--headless` 批处理，支持缺陷/外场/驱动 κ 与 κ sweep。
2. **标准化输出**：每次运行目录包含 `config.toml`、`meta.json`、`vortices.csv`（可选 `vortex_positions.csv` / `kappa_sweep.csv`）。
3. **后处理脚本**（Python）：从 CSV 自动提取 κ_c、画相图、画 matching field、计算结构因子 S(k)。
4. **AI 闭环**：用已有数据训练轻量代理模型，自动选点→回仿真→回填数据集（`scripts/ai_closed_loop.py`）。

## 物理模型

Gauge-covariant TDGL 方程（含矢势）：

```
∂ψ/∂t = (∇ - iA)^2 ψ + α(r)ψ - |ψ|^2 ψ
```

- ψ：复数序参量（超导序参）
- A：矢势（Landau gauge: A = (0, Bx, 0)）
- B：外磁场强度
- α(r)：材料参数场（缺陷处 α < 0）
- 边界条件：磁周期边界（torus 上均匀磁场自洽，B 自动量子化为整数磁通 n）
- 时间推进：显式 Euler + link 变量

## 快速开始

```bash
# 交互式可视化
cargo run -- --flux-n 209

# 固定随机种子（可复现）
cargo run -- --flux-n 209 --seed 1234

# 或指定目标 B（会自动量子化到最近的整数磁通 n）
cargo run -- --b 0.02

# 查看全部参数
cargo run -- --help

# 性能基准测试
cargo run -- --bench --flux-n 209

# Headless sweep (no window, writes vortices.csv)
cargo run -- --headless --steps 20000 --sample-period 100 --flux-n 209 --seed 1234

# Depinning / drive scan point (kappa) + keep outputs separated
cargo run -- --headless --steps 20000 --sample-period 100 --flux-n 209 --kappa 0.02 --out-dir runs/kappa_0.02

# Also dump vortex positions (for structure factor / tracking)
cargo run -- --headless --steps 20000 --sample-period 100 --flux-n 209 --kappa 0.02 --dump-positions --out-dir runs/kappa_0.02

# Kappa sweep (depinning curve), writes kappa_sweep.csv
cargo run -- --headless --flux-n 209 --kappa-start 0.0 --kappa-end 0.05 --kappa-step 0.01 --kappa-initial-relax-steps 20000 --kappa-relax-steps 2000 --kappa-measure-steps 5000 --sample-period 100 --out-dir runs/kappa_sweep

# Plot results
python scripts/plot_vortices.py runs/kappa_0.02/vortices.csv
python scripts/plot_kappa_sweep.py runs/kappa_sweep/kappa_sweep.csv --order-parameter abs_mean_vx --kappa-c-method baseline_threshold --epsilon 1e-3
```

### 交互控制

- `A` 键：切换显示 |ψ| / α 场
- `D` 键：切换 Dashboard UI 显示/隐藏
- 关闭窗口退出

### Dashboard UI 功能

交互式 egui Dashboard 提供以下面板：

- **参数面板**：实时调整 κ、dt、材料预设等参数
- **统计面板**：显示涡旋计数、能量密度、平均速度等
- **历史面板**：时间序列图表（涡旋数、能量、速度）
- **验证面板**：物理一致性检查与验证报告
- **状态栏**：仿真步数、FPS、GPU 信息

## 输出文件

- `config.toml`：本次运行的全部参数（写入 `--out-dir`），便于复现实验
- `meta.json`：运行环境信息（GPU/后端/argv/时间戳，写入 `--out-dir`）
- `vortices.csv`：涡旋统计与诊断数据（计数 + 能量 + 钉扎/动力学观测量）
  - 列：`step,time,kappa,vortices,antivortices,net,energy,energy_density,pinned_v,pinned_av,pinned_net,mean_vx,mean_vy,mean_speed`
  - 文件开头会以 `# ...` 注释行记录本次运行的 nx/ny、dt/dx、flux_n、phi/kappa/B、seed 与缺陷参数（便于复现实验）
- `vortex_positions.csv`（可选）：`--dump-positions` 时输出每次采样的涡旋/反涡旋位置（`step,time,kappa,x_cell,y_cell,sign`），用于结构因子/轨迹后处理
- `kappa_sweep.csv`（kappa 扫参模式）：当提供 `--kappa-start/--kappa-end/--kappa-step` 时输出，用于绘制 depinning 曲线
  - 列：`kappa,samples,mean_speed,mean_vx,mean_vy,net_mean,pinned_net_mean,energy_density_mean`

### 自动化扫参（相图）

```bash
python scripts/run_depinning_phase_diagram.py --flux-n 209 --seed 1234 --order-parameter abs_mean_vx --kappa-c-method baseline_threshold --initial-relax-steps 20000 --out-root runs/phase_diagram_smoke --max-jobs 3
python scripts/plot_phase_diagram.py runs/phase_diagram_smoke/phase_diagram.csv --no-show
```

### AI 反演 / 逆向设计（baseline）

```bash
# 训练一个轻量代理模型：kappa_c ~= f(defect params)
python scripts/ai_inverse_design.py train runs/phase_diagram_smoke/phase_diagram.csv

# 给定目标 kappa_c，搜索参数组合（离散网格）
python scripts/ai_inverse_design.py invert runs/phase_diagram_smoke/phase_diagram.csv --target 0.03 --search-from-data --top 10
```

### AI 闭环（active learning）

```bash
# 注意：负数列表必须用 '=' 传参（argparse 会把 "-0.2,-0.5" 误判为新的选项）
python scripts/ai_closed_loop.py --build --objective maximize --iters 6 --init-random 3 --out-root runs/ai_closed_loop_smoke --flux-n-list=209 --seed-list=1234 --defect-mode-list=random --defect-spacing-list=32 --alpha-defect-list=-0.2,-0.5 --defect-radius-list=3 --defect-count-list=0,50 --kappa-start 0.0 --kappa-end 0.05 --kappa-step 0.01 --initial-relax-steps 20000 --relax-steps 2000 --measure-steps 5000 --sample-period 100 --order-parameter abs_mean_vx --kappa-c-method baseline_threshold --epsilon 1e-3

# Target inversion (objective=target): loop_progress.png plots |kappa_c-target|.
python scripts/ai_closed_loop.py --objective target --target 0.025 --iters 6 --init-random 3 --out-root runs/ai_inversion_target_lattice_refined --nx 256 --ny 256 --flux-n-list 64 --seed-list 1234 --defect-mode-list lattice --defect-spacing-list 24,32,40 --alpha-defect-list=-0.2,-0.5,-1.0 --defect-radius-list 3 --defect-count-list 0 --kappa-start 0.0 --kappa-end 0.05 --kappa-step 0.005 --initial-relax-steps 10000 --relax-steps 1000 --measure-steps 2000 --sample-period 100 --order-parameter abs_mean_vx --kappa-c-method baseline_threshold --epsilon 1e-3 --baseline-points 2

# Offline inversion accuracy evaluation (k-fold); fill missing kappa_c with kappa_end (censored).
python scripts/evaluate_ai_inversion.py runs/phase_diagram_ai_eval_128/phase_diagram.csv --fill-missing-with-kappa-end --kfold 5 --delta 0.005
```

输出：
- `phase_diagram.csv`：追加评估结果（与相图脚本兼容）
- `loop_log.jsonl`：每次选点的预测/不确定性/acquisition/命令/耗时
- `loop_progress.png`：闭环进展曲线（objective=maximize：best kappa_c；objective=target：best |kappa_c-target|）

### 匹配场（matching field）对比：random vs lattice

```bash
python scripts/run_matching_field_scan.py --flux-n-list 32,48,64,80,96 --defect-mode-list random,lattice --defect-spacing 32 --alpha-defect -0.5 --defect-radius 3 --defect-count 64 --kappa-start 0 --kappa-end 0.05 --kappa-step 0.01 --initial-relax-steps 20000 --relax-steps 2000 --measure-steps 5000 --sample-period 100 --order-parameter abs_mean_vx --kappa-c-method baseline_threshold --epsilon 1e-3 --out-root runs/matching_field_scan --overwrite-summary
python scripts/plot_matching_field.py runs/matching_field_scan/matching_field.csv --show-matching --no-show
```

### 结构因子 S(k)（Abrikosov 晶格/有序性定量）

```bash
# 生成 vortex_positions.csv（结构因子输入）
cargo run -- --headless --steps 20000 --sample-period 100 --flux-n 64 --seed 1234 --dump-positions --out-dir runs/structure_factor_demo

# 计算并画 2D 结构因子 S(k)
python scripts/plot_structure_factor.py runs/structure_factor_demo/vortex_positions.csv --log10 --no-show
```

## 数值参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| NX, NY | 256 | 网格尺寸 |
| dt | 0.01 | 时间步长（< dx^2/4 = 0.25 稳定性条件）|
| dx | 1.0 | 空间步长 |
| α_default | 1.0 | 超导区材料参数 |
| α_defect | -0.5 | 缺陷区材料参数 |
| flux_n | ~209 | 总磁通量子数（推荐直接用 `--flux-n` 指定） |
| B_target | 0.02 | 目标外场强度（用 `--b` 指定，会量子化到最近 `flux_n`） |
| 缺陷数量 | 50 | 随机圆形缺陷 |
| 缺陷半径 | 3 | 像素 |
| 缺陷模式 | random | `--defect-mode random|lattice` |
| 阵列间距 | 32 | `--defect-spacing`（仅 lattice 模式） |

外场量子化关系（torus 上均匀磁场自洽）：

$$
\phi = \frac{2\pi n}{N_xN_y},\quad B=\frac{\phi}{dx^2}
$$

## 性能数据

RTX 4060 Laptop GPU 基准测试：

| 网格规模 | steps/s | cells/s |
|----------|---------|---------|
| 128^2 | 29,833 | 4.89e8 |
| 256^2 | 28,410 | 1.86e9 |
| 512^2 | 22,310 | 5.85e9 |
| 1024^2 | 13,329 | 1.40e10 |

## 技术实现

- **GPU 框架**：wgpu 23 (WebGPU)
- **窗口**：winit 0.30 (ApplicationHandler)
- **渲染**：全屏三角形 + fragment shader 采样
- **计算**：ping-pong buffer 交替更新
- **外场/边界**：磁通量量子化 + 磁周期边界（x 边界 hop 的 `Ux` 缝合相位）
- **涡旋检测**：CPU 端规范不变绕数（基于 link），定期采样

## 项目结构

```
Rust_wgpu_TDGL_AI_Trial/
├── Cargo.toml
├── README.md
├── REPORT.md
├── CLAUDE.md
├── LICENSE
├── requirements.txt
├── src/
│   ├── main.rs              # 主程序（compute + render + vortex detection）
│   ├── ui/                  # egui Dashboard UI 模块
│   │   ├── mod.rs
│   │   ├── theme.rs         # UI 主题配置
│   │   ├── components/      # 可复用 UI 组件
│   │   │   ├── depinning_curve.rs   # Depinning 曲线图表
│   │   │   ├── param_slider.rs      # 参数滑块
│   │   │   └── time_series.rs       # 时间序列图表
│   │   └── panels/          # Dashboard 面板
│   │       ├── params_panel.rs      # 参数控制面板
│   │       ├── stats_panel.rs       # 统计显示面板
│   │       ├── history_panel.rs     # 历史数据面板
│   │       ├── validation_panel.rs  # 验证面板
│   │       └── status_bar.rs        # 状态栏
│   └── utils/               # 工具模块
│       ├── animation.rs     # 动画工具
│       ├── materials.rs     # 材料预设库
│       ├── presets.rs       # 仿真预设
│       └── validation_report.rs  # 验证报告生成
├── scripts/
│   ├── plot_vortices.py               # vortices.csv 时间序列
│   ├── plot_kappa_sweep.py            # kappa_sweep.csv depinning 曲线 + κ_c
│   ├── run_depinning_phase_diagram.py # 自动扫参/提取 κ_c -> phase_diagram.csv
│   ├── plot_phase_diagram.py          # phase_diagram.csv 热图
│   ├── run_convergence_study.py       # 收敛性(dt/dx)与有限尺寸效应（kappa sweep 汇总）
│   ├── plot_convergence_study.py      # convergence_study.csv -> 曲线
│   ├── plot_lit_compare.py            # 文献对比图
│   ├── validate_run.py                # 单次运行输出自检（schema + sanity checks）
│   ├── evaluate_ai_inversion.py       # AI 反演精度评估（离线）
│   ├── run_matching_field_scan.py     # matching field: scan flux_n (random vs lattice)
│   ├── plot_matching_field.py         # matching_field.csv 曲线
│   ├── plot_structure_factor.py       # vortex_positions.csv -> 2D S(k)
│   ├── ai_inverse_design.py           # AI 反演/逆向设计（baseline）
│   └── ai_closed_loop.py              # AI 闭环（active learning）
├── docs/
│   ├── epic-dashboard-ui.md           # Dashboard UI Epic 文档
│   ├── front-end-spec.md              # 前端规格说明
│   └── stories/                       # 用户故事
├── doc/
│   ├── README.md                 # 文档索引
│   ├── IMPLEMENTATION_LOG.md     # 实施日志
│   ├── RESEARCH_ROADMAP.md       # 深度分析与研究路线图
│   └── Rust_wgpu_TDGL_AI_Trial_Doc.md  # 需求文档/AI 工作流
├── runs/                 # 本地输出目录（默认 --out-dir；.gitignore 忽略）
├── vortices.csv         # legacy：用 --out-dir . 生成（示例/旧行为）
└── vortices_plot.png    # legacy：用 --out-dir . 生成（示例/旧行为）
```

## 依赖

### Rust

```toml
wgpu = "23"
winit = "0.30"
egui = "0.30"
egui-wgpu = "0.30"
egui-winit = "0.30"
pollster = "0.4"
bytemuck = { version = "1", features = ["derive"] }
rand = "0.8"
env_logger = "0.11"
log = "0.4"
```

### Python（后处理脚本）

脚本尽量保持“离线可跑”，常用依赖：

- `numpy`
- `pandas`
- `matplotlib`

建议用 `pip install -r requirements.txt` 安装。

## 物理验证

- 规范一致性：磁通量量子化（`--flux-n`）+ 磁周期边界 + gauge-invariant winding（有外场时仍可正确统计净涡旋）。
- 诊断完备：`vortices.csv` 含能量/能量密度与钉扎/速度观测量；κ sweep 输出 `kappa_sweep.csv` 便于提取 κ_c。
- 可复现实验：`--seed` + `--out-dir` + `config.toml/meta.json`（同一参数可重跑/可追溯）。
- 研究示例：matching field smoke 显示周期缺陷阵列在 `flux_n≈N_pins` 附近 κ_c 更高（见 `scripts/run_matching_field_scan.py` + `scripts/plot_matching_field.py`）。
- smoke 输出目录（可选，用于演示脚本链路）：`runs/matching_field_smoke/`、`runs/structure_factor_smoke/`、`runs/ai_closed_loop_smoke/`、`runs/convergence_dt_flux64_smoke/`、`runs/convergence_dx_smoke/`、`runs/convergence_dx_flux64_refined/`、`runs/finite_size_smoke/`、`runs/finite_size_refined/`、`runs/ai_inversion_target_smoke/`、`runs/ai_inversion_target_lattice_refined/`、`runs/phase_diagram_ai_eval_128/`。
- 单次运行自检：`scripts/validate_run.py` 对 `vortices.csv` 做 schema + 基础物理 sanity checks（net≈flux_n、能量下降等）。

## 报告

详细的物理背景、数值方法和实验结果请参阅 [REPORT.md](REPORT.md)。

## Roadmap（进度）

- [x] 外场自洽：磁通量量子化（`--flux-n`）+ 磁周期边界（torus 上均匀外场）
- [x] 规范不变涡旋统计：gauge-invariant winding（外场下仍可统计净涡旋）
- [x] depinning：驱动 κ + κ sweep + κ_c 自动提取（`kappa_sweep.csv`）
- [x] 相图自动化：扫参→汇总 `phase_diagram.csv` → 热图脚本
- [x] matching field：random vs lattice（commensurability peak）
- [x] 结构因子 S(k)：`vortex_positions.csv` → FFT 热图
- [x] AI 反演 baseline + AI 闭环 active learning（自动选点→回仿真→回填）
- [x] 收敛性实验：dt/dx（至少 dt vs dt/2）与有限尺寸效应
- [x] **egui Dashboard UI**：交互式参数控制、实时统计、时间序列图表
- [x] **材料预设库**：NbSe2、YBCO、MgB2 等超导材料参数
- [x] **仿真验证面板**：物理一致性检查与验证报告
- [ ] 热噪声（Langevin）与 T-sweep 相图（玻璃态/蠕变）
- [ ] 更丰富的结构量：g(r)、S(k) 峰宽/主峰跟踪（与 matching field 联动）
- [ ] AI 反演增强：从图像/时间序列反演缺陷参数（带不确定性）

更完整的研究路线与创新方向见 [doc/RESEARCH_ROADMAP.md](doc/RESEARCH_ROADMAP.md)。

## 许可证

MIT (see `LICENSE`)
