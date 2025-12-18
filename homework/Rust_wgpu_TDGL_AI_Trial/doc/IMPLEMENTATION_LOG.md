<!--
ASCII-only header (keep this block) to avoid a Windows apply_patch UTF-8 slicing bug.
File encoding: UTF-8.
Last updated: 2025-12-18.
Synced with: kappa drive+sweep, kappa initial relax, kappa_c extraction scripts, configurable defects, out-dir, vortex position dump, pinned/velocity observables, AI inversion baseline, AI closed loop runner.
Doc note: normalized exponent notation (use ^ and e-notation instead of superscripts for console-friendliness).
Doc note: convergence+finite-size studies added (nx/ny flags + scripts/run_convergence_study.py + scripts/validate_run.py).
Doc note: refined convergence runs + AI inversion accuracy evaluation added (see runs/*_refined and runs/phase_diagram_ai_eval_128).
Doc note: default --out-dir is runs/<mode>_<unix_ms> (pass --out-dir . for legacy cwd output).
Doc note: repo hygiene: LICENSE + requirements.txt + .gitignore (target/, runs/).
Pad: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
Pad2: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-->

# TDGL 项目实施日志
工程新颖性：基于 Rust + wgpu/WebGPU 的 GPU TDGL 端到端平台，实现 compute+render 融合、无 CPU 回读的实时可视化，并给出跨网格规模的吞吐基准（你这组 cells/s 数据非常亮眼）。
## 阶段 0：wgpu compute 骨架

### 完成时间
2024-12-17

### 实现内容
1. 修复 `Cargo.toml`：edition 从 2024 改为 2021，添加依赖
2. 初始化 wgpu 实例/设备/队列
3. 创建 ping-pong storage buffer
4. 实现最小 compute shader (`out = in + 1.0`)
5. 验证 GPU 数据流正确性

### 关键代码
- `Cargo.toml`：wgpu 23, pollster, bytemuck, anyhow, env_logger, log
- `src/main.rs`：wgpu 初始化 + compute pipeline + buffer 回读验证

### 测试结果
```
使用适配器: "NVIDIA GeForce RTX 4060 Laptop GPU"
验证通过！256 个元素全部正确
阶段 0 完成：GPU compute 骨架验证通过
```

### Codex Review 反馈
- 代码结构清晰，满足阶段 0 需求
- wgpu API 使用正确
- 小建议：`input_buffer` 不需要 `COPY_SRC`

---

## 阶段 1：TDGL 更新核

### 开始时间
2024-12-17

### 实现内容
1. 2D 网格索引与周期边界（WGSL）
2. 复数运算（vec2<f32> 表示实部和虚部）
3. 5 点 stencil 拉普拉斯算子
4. 显式 Euler 时间推进
5. TDGL 方程实现

### 数值参数
- 网格：256×256
- dt = 0.01（满足稳定性条件 dt < dx^2/4 = 0.25）
- dx = 1.0
- alpha = 1.0（均匀场）
- 边界条件：周期边界
- 初始条件：随机噪声

### TDGL 方程
```
∂ψ/∂t = ∇^2ψ + α(r)ψ - |ψ|^2ψ
```

离散化（显式 Euler）：
```
ψ_next = ψ + dt * (laplacian(ψ) + α*ψ - |ψ|^2*ψ)
```

5 点 stencil 拉普拉斯：
```
∇^2ψ ≈ (ψ[x+1,y] + ψ[x-1,y] + ψ[x,y+1] + ψ[x,y-1] - 4*ψ[x,y]) / dx^2
```

### 关键文件变更
- `Cargo.toml`：添加 rand 依赖
- `src/main.rs`：TDGL compute shader + 随机初态

### 测试结果
```
GPU TDGL 单步更新完成
  网格: 256x256
  dt=0.01, dx=1, alpha=1
  GPU |psi| 均值: 0.074401
  GPU |psi| 最大: 0.137803
运行 CPU 参考实现...
  CPU |psi| 均值: 0.074401
  CPU |psi| 最大: 0.137803
CPU/GPU 对比结果:
  最大差异: 7.45e-9
阶段 1 完成：TDGL 更新核验证通过，CPU/GPU 结果一致！
```

### Codex Review 反馈
- CPU/GPU 实现一致，无正确性问题
- 周期边界、拉普拉斯算子、复数运算均正确
- 建议：预计算 `inv_dx2 = 1.0/(dx*dx)` 提升性能
- 多步演化需要 ping-pong buffer 交换
- 稳定性条件已满足 (dt=0.01 < dx^2/4=0.25)

---

## 阶段 2：缺陷/钉扎势

### 完成时间
2024-12-17

### 实现内容
1. 添加 alpha 场 buffer（与 psi 同尺寸）
2. Compute shader 读取空间变化的 α(r)
3. 随机生成圆形缺陷点
4. 按 A 键切换显示 |ψ| / alpha 场

### 缺陷参数
- 默认 alpha = 1.0（超导区）
- 缺陷 alpha = -0.5（抑制超导）
- 缺陷半径 = 3 像素
- 缺陷数量 = 50 个

### 测试结果
```
GPU: "NVIDIA GeForce RTX 4060 Laptop GPU"
生成 50 个缺陷点
步数: 100, 200, ...
显示模式: alpha场 / |ψ| (按 A 切换)
```

### 物理效果
- 缺陷区域 |ψ| 被抑制（蓝色）
- 超导区域 |ψ| → 1（红色）
- 可观察到涡旋被缺陷钉扎

---

## 阶段 3：可视化

### 完成时间
2024-12-17

### 实现内容
1. 使用 winit 0.30 创建窗口（ApplicationHandler 模式）
2. GPU 端直接渲染 |ψ| 热力图（无 CPU 回读）
3. 实现多步 TDGL 演化（ping-pong buffer 交换）
4. 伪彩色映射（蓝-白-红）
5. 实时显示涡旋形成过程

### 关键技术
- 全屏三角形渲染（3 顶点覆盖整个屏幕）
- Fragment shader 直接采样 storage buffer
- 每帧执行 10 步 TDGL 更新
- ping-pong 双 bind group 交替使用

### 关键文件变更
- `Cargo.toml`：添加 winit 0.30 依赖
- `src/main.rs`：完整重写，包含 compute + render pipeline

### 测试结果
```
使用适配器: "NVIDIA GeForce RTX 4060 Laptop GPU"
步数: 100, 模拟时间: 1.00
步数: 200, 模拟时间: 2.00
...
步数: 4700, 模拟时间: 47.00
```

### 性能
- 每秒约 100 步（10 步/帧 × ~10 FPS with VSync）
- 256×256 网格实时渲染流畅

---

## 阶段 4：研究实验与扫参

### 完成时间
2024-12-17

### 实现内容
1. 添加 `--bench` 命令行参数
2. 无窗口基准测试模式
3. 多网格规模性能测试（128^2, 256^2, 512^2, 1024^2）
4. 预热阶段 + 精确计时

### 基准测试方法
- 每个网格规模运行 500 步
- 50 步预热（不计入统计）
- 使用 `device.poll(Wait)` 确保 GPU 完成
- 计算 steps/s 和 cells/s

### 测试结果（RTX 4060 Laptop GPU）
```
网格规模      steps/s         cells/s
----------------------------------------
    128^2     29832.8       4.89e8
    256^2     28410.1       1.86e9
    512^2     22309.5       5.85e9
   1024^2     13329.2       1.40e10
```

### 性能分析
- 小网格（128^2）受 dispatch 开销限制
- 大网格（1024^2）接近显存带宽瓶颈
- 256^2 是交互式模拟的最佳平衡点
- 1024^2 仍可达到 1.4e10 cells/s 吞吐量

### 使用方法
```bash
# 交互式可视化
cargo run

# 性能基准测试
cargo run -- --bench
```

---

## 阶段 4.1：涡旋检测

### 完成时间
2024-12-17

### 实现内容
1. 相位绕数（phase winding）算法
2. CPU 端涡旋检测（每 100 步采样）
3. Buffer 回读机制
4. CSV 输出 N_v(t) 曲线

### 算法原理
对每个网格单元，累加四边相位差（unwrap 到 (-π,π]），总和接近 ±2π 即为涡旋/反涡旋：
```rust
let sum = wrap_phase(p10 - p00) + wrap_phase(p11 - p10)
        + wrap_phase(p01 - p11) + wrap_phase(p00 - p01);
if sum > PI { vort += 1; }
else if sum < -PI { anti += 1; }
```

### 测试结果
```csv
step,time,vortices,antivortices,net
100,1.00,1425,1425,0
200,2.00,674,674,0
500,5.00,263,263,0
1000,10.00,151,151,0
2000,20.00,96,96,0
```

### 物理分析
- 初始随机噪声产生大量涡旋对（~1400 对）
- 涡旋-反涡旋成对湮灭，数量指数衰减
- net=0 符合周期边界下的拓扑守恒
- 最终稳态约 100 对涡旋（被缺陷钉扎）

---

## 项目完成总结

### 已实现功能
- ✅ wgpu compute 骨架
- ✅ TDGL 更新核（CPU/GPU 验证通过）
- ✅ 缺陷/钉扎势（空间变化 α 场）
- ✅ GPU 端实时可视化
- ✅ 性能基准测试
- ✅ 涡旋检测与统计

### 技术亮点
- 纯 GPU 渲染，无 CPU 回读（可视化）
- 定期采样涡旋检测（研究分析）
- ping-pong buffer 高效交替
- winit 0.30 ApplicationHandler 模式
- 全屏三角形渲染技术

### 物理验证
- CPU/GPU 结果最大差异：7.45e-9
- 可观察涡旋形成与缺陷钉扎现象
- 涡旋数 N_v(t) 曲线符合物理预期
- 稳定性条件满足（dt=0.01 < dx^2/4=0.25）

### 输出文件
- `vortices.csv`：涡旋统计数据

---

## 阶段 4.2：外磁场（Gauge-covariant TDGL）

### 完成时间
2024-12-17

### 实现内容
1. Gauge-covariant Laplacian（link 变量）
2. Landau gauge: A = (0, Bx, 0)
3. 复数乘法函数 cmul/conj
4. Params 结构体添加 B 字段

### 物理方程
```
∂ψ/∂t = (∇ - iA)^2ψ + α(r)ψ - |ψ|^2ψ
```

离散化（link 变量）：
```
Uy(x) = exp(-i B x dx)
Δ_A ψ = (ψ_xp + ψ_xm + Uy·ψ_yp + Uy*·ψ_ym - 4ψ) / dx^2
```

### 测试结果对比
| 条件 | 初始涡旋 | 稳态涡旋 |
|------|----------|----------|
| B=0 | ~1400 对 | ~100 对 |
| B=0.02 | ~3000 对 | ~400 对 |

### 物理分析
- 外磁场增加涡旋数量（磁通涡旋）
- 涡旋密度与磁场强度正相关
- 符合 Type-II 超导体物理预期

---

## 阶段 4.3：可视化脚本

### 完成时间
2024-12-17

### 实现内容
- Python 脚本 `scripts/plot_vortices.py`
- 绘制 N_v(t) 曲线
- 输出 `vortices_plot.png`

### 使用方法
```bash
python scripts/plot_vortices.py
```

---

## 阶段 4.4：磁周期边界 + 外场量子化 + 规范不变涡旋检测 + 能量诊断

### 完成时间
2025-12-17

### 实现内容
1. 外场参数改为 **plaquette flux**：`phi = B * dx^2`（uniform 传入 `phi`）
2. 支持外场输入：
   - `--flux-n <i32>`：总磁通量子数（推荐）
   - `--b <f32>`：目标外场强度（会自动量子化到最近的 `flux-n`）
   - `--dt <f32>`、`--dx <f32>`：修改步长
   - `--seed <u64>`：随机种子（复现实验；不提供则随机生成并在日志/CSV 中记录）
3. 引入 **磁周期边界（magnetic periodic BC）**：
   - `Uy(i)=exp(-i phi * i)`
   - `Ux(i,j)=1`（内部）
   - 仅在 `x=nx-1 -> 0` 的边界 hop：`Ux=exp(+i phi * nx * j)`
4. 涡旋检测升级为 **规范不变绕数（gauge-invariant winding）**（基于 link 的边相位增量）
5. 增加耗散/稳定性诊断：输出离散能量泛函 `energy` 与 `energy_density`
6. 更新绘图脚本：支持绘制 `net`，若存在则绘制 `energy_density`

### 关键关系（torus 上均匀磁场自洽）

磁通量量子化：

$$
\phi N_xN_y = 2\pi n,\quad n\in\mathbb Z
$$

因此：

$$
\phi = \frac{2\pi n}{N_xN_y},\quad B=\frac{\phi}{dx^2}
$$

### 输出变化

`vortices.csv` 列更新为：

```csv
step,time,vortices,antivortices,net,energy,energy_density
```

文件开头会以 `# ...` 注释行记录本次运行的 nx/ny、dt/dx、flux_n、phi/B、seed 与缺陷参数（便于复现实验）。

### 使用方法

```bash
# 交互式可视化（推荐用 flux-n）
cargo run -- --flux-n 209

# 或用目标 B（会自动量子化）
cargo run -- --b 0.02

# 基准测试
cargo run -- --bench --flux-n 209
```

---

## 阶段 4.5：Headless 模式（无窗口扫参/批处理）

### 完成时间
2025-12-17

### 实现内容
1. 增加运行模式 `--headless`：不创建窗口，直接运行 TDGL 并写出 `vortices.csv`
2. 增加参数：
   - `--steps <u64>`：总步数（默认 5000）
   - `--sample-period <u64>`：采样周期（默认 `VORTEX_SAMPLE_PERIOD`）
3. 采样逻辑与交互模式保持一致：每次采样时 copy 当前 `psi` → CPU，执行
   - `detect_vortices(...)`（规范不变绕数）
   - `energy_functional(...)`（能量泛函与能量密度）
4. `vortices.csv` 额外写入 headless 元信息：
   - `# mode=headless steps=... sample_period=...`

### 使用方法

```bash
# headless：适合扫参/批处理（输出 vortices.csv）
cargo run -- --headless --steps 20000 --sample-period 100 --flux-n 209 --seed 1234
```

---

## 阶段 4.6：驱动 κ + 钉扎/动力学观测量 + 可配置缺陷 + 输出目录

### 完成时间
2025-12-17

### 实现内容
1. 引入驱动参数 `--kappa <f32>`（相位扭曲/等效常量 Ay0）：`Uy <- exp(-i(phi*x + kappa))`
2. 缺陷参数可配置：
   - `--alpha-default`、`--alpha-defect`、`--defect-radius`、`--defect-count`
3. 观测量扩展（写入 `vortices.csv`）：
   - `pinned_v/pinned_av/pinned_net`：被钉扎涡旋计数（基于 α 场与 cell 重叠的近似判据）
   - `mean_vx/mean_vy/mean_speed`：基于“最近邻匹配”的平均漂移速度（用于 depinning 阈值曲线）
4. 输出目录 `--out-dir <path>`：每次运行可将输出写入独立目录（默认 `runs/<mode>_<unix_ms>`），便于扫参
5. 可选输出 `--dump-positions`：写出 `vortex_positions.csv`（`step,time,kappa,x_cell,y_cell,sign`），用于结构因子/轨迹后处理

### 使用方法

```bash
# depinning 单点：kappa!=0，输出到独立目录
cargo run -- --headless --steps 20000 --sample-period 100 --flux-n 209 --kappa 0.02 --out-dir runs/kappa_0.02

# 同时输出涡旋位置
cargo run -- --headless --steps 20000 --sample-period 100 --flux-n 209 --kappa 0.02 --dump-positions --out-dir runs/kappa_0.02
```

---

## 阶段 4.7：kappa sweep（depinning 曲线自动化）

### 完成时间
2025-12-17

### 实现内容
1. 新增 headless κ 扫参模式：`--kappa-start/--kappa-end/--kappa-step`（每个 κ 点先 relax 再 measure）
2. 生成 `kappa_sweep.csv`：汇总每个 κ 的 `mean_speed/mean_vx/mean_vy/net_mean/pinned_net_mean/energy_density_mean`
3. `vortices.csv` / `vortex_positions.csv` 每行新增 `kappa` 列，便于合并多段运行结果
4. 后处理脚本：`scripts/plot_kappa_sweep.py` 绘制 `mean_speed(kappa)`（depinning order parameter）

### 使用方法

```bash
# kappa sweep（生成 kappa_sweep.csv）
cargo run -- --headless --flux-n 209 --kappa-start 0.0 --kappa-end 0.05 --kappa-step 0.01 --kappa-relax-steps 2000 --kappa-measure-steps 5000 --sample-period 100 --out-dir runs/kappa_sweep

# 画 depinning 曲线
python scripts/plot_kappa_sweep.py runs/kappa_sweep/kappa_sweep.csv
```

---

## 阶段 4.8：实验可复现打包（config.toml + meta.json）与相图扫参脚本

### 完成时间
2025-12-18

### 实现内容
1. 每次 headless 运行自动在 `--out-dir` 下生成：
   - `config.toml`：本次运行的完整参数（便于复现）
   - `meta.json`：GPU/后端/argv/时间戳等运行环境信息
2. 新增自动扫参脚本 `scripts/run_depinning_phase_diagram.py`：
   - 批量扫描 `alpha_defect/defect_count/defect_radius`
   - 对每个点运行 κ sweep 并读取 `kappa_sweep.csv`
   - 按阈值 `mean_speed > epsilon` 提取 `kappa_c`
   - 汇总输出 `phase_diagram.csv`（后续可直接画热图/相图）

### 使用方法

```bash
# 生成相图数据（示例：只跑 3 个点用于 smoke test）
python scripts/run_depinning_phase_diagram.py --flux-n 209 --seed 1234 --out-root runs/phase_diagram_smoke --max-jobs 3
```

---

## 阶段 4.9：缺陷几何（随机 vs 周期阵列）—— matching field 的工程入口

### 完成时间
2025-12-18

### 实现内容
1. 新增缺陷几何模式：
   - `--defect-mode random`：随机圆形缺陷（默认）
   - `--defect-mode lattice`：周期方阵缺陷（square lattice）
2. 新增 `--defect-spacing <i32>`：周期缺陷阵列间距（cell），用于 commensurability/matching field 实验
3. CSV 元信息与 `config.toml` 同步记录 `defect_mode/defect_spacing`

### 使用方法

```bash
# random defects
cargo run -- --headless --steps 20000 --sample-period 100 --flux-n 209 --defect-mode random --defect-count 50 --defect-radius 3 --out-dir runs/defects_random

# periodic defect lattice (approx. 8x8 = 64 sites for spacing=32)
cargo run -- --headless --steps 20000 --sample-period 100 --flux-n 209 --defect-mode lattice --defect-spacing 32 --defect-radius 3 --out-dir runs/defects_lattice
```

---

## 阶段 4.10：kappa_c 提取脚本增强（order parameter + 方法）

### 完成时间
2025-12-18

### 实现内容
1. `scripts/plot_kappa_sweep.py` 支持：
   - `--order-parameter mean_speed|abs_mean_vx|abs_mean_vy|abs_mean_v`
   - `--kappa-c-method threshold|baseline_threshold|two_phase_fit`
2. `scripts/run_depinning_phase_diagram.py` 同步支持上述参数，并将选择写入 `phase_diagram.csv`（便于复现实验口径）
3. 新增 `--overwrite-summary`：覆盖写 `phase_diagram.csv`（避免多次追加造成混淆）

### 使用方法

```bash
# 画 depinning 曲线（推荐沿驱动导致的漂移方向：abs_mean_vx）
python scripts/plot_kappa_sweep.py runs/kappa_sweep/kappa_sweep.csv --order-parameter abs_mean_vx --kappa-c-method baseline_threshold --epsilon 1e-3

# 批量扫参并提取 kappa_c（示例：smoke test）
python scripts/run_depinning_phase_diagram.py --flux-n 209 --seed 1234 --order-parameter abs_mean_vx --kappa-c-method baseline_threshold --out-root runs/phase_diagram_smoke --max-jobs 3 --overwrite-summary
```

---

## 阶段 4.11：kappa sweep 初始热身（抑制 kappa_start 瞬态）

### 完成时间
2025-12-18

### 实现内容
1. Rust CLI 新增：`--kappa-initial-relax-steps <u64>`：仅对 sweep 的第一个 κ 点使用更长 relax（默认等于 `--kappa-relax-steps`）
2. `config.toml` 的 `[kappa_sweep]` 记录 `initial_relax_steps`
3. `kappa_sweep.csv`/`vortices.csv` 的 `# mode=...` 元信息记录 `initial_relax_steps`
4. `scripts/run_depinning_phase_diagram.py` 新增：`--initial-relax-steps`，自动传递给二进制（0 表示跟随 `--relax-steps`）

### 使用方法

```bash
# 推荐：只对 kappa_start 做更长热身，降低初始随机噪声带来的瞬态速度
cargo run -- --headless --flux-n 209 --kappa-start 0.0 --kappa-end 0.05 --kappa-step 0.01 --kappa-initial-relax-steps 20000 --kappa-relax-steps 2000 --kappa-measure-steps 5000 --sample-period 100 --out-dir runs/kappa_sweep
```

---

## 阶段 5.0：AI 反演/逆向设计（baseline）

### 完成时间
2025-12-18

### 实现内容
1. 新增 `scripts/ai_inverse_design.py`：
   - 读取 `phase_diagram.csv`（`status=ok` 且 `kappa_c` 可解析）
   - ridge 回归代理模型：`kappa_c ~= f(defect params)`
   - 给定目标 `kappa_c*`，在离散候选网格上搜索最接近的参数组合（最小闭环）

### 使用方法

```bash
# 训练
python scripts/ai_inverse_design.py train runs/phase_diagram_smoke/phase_diagram.csv

# 反演：用数据集中的取值组成搜索网格
python scripts/ai_inverse_design.py invert runs/phase_diagram_smoke/phase_diagram.csv --target 0.03 --search-from-data --top 10
```

---

## 阶段 5.1：匹配场（matching field）扫描（random vs lattice）

### 完成时间
2025-12-18

### 实现内容
1. 新增 `scripts/run_matching_field_scan.py`：
   - 扫 `flux_n`（等效外场 B）并对比 `defect_mode=random` vs `defect_mode=lattice`
   - 汇总输出 `matching_field.csv`（包含 `kappa_c` 与 `pinned_fraction_k0` 等字段）
2. 新增 `scripts/plot_matching_field.py`：
   - 绘制 `kappa_c(flux_n)`，可用 `--show-matching` 标注 `flux_n = m*N_pins`（周期阵列的匹配场位置）

### 使用方法

```bash
python scripts/run_matching_field_scan.py --flux-n-list 32,48,64,80,96 --defect-mode-list random,lattice --defect-spacing 32 --alpha-defect -0.5 --defect-radius 3 --defect-count 64 --kappa-start 0 --kappa-end 0.05 --kappa-step 0.01 --initial-relax-steps 20000 --relax-steps 2000 --measure-steps 5000 --sample-period 100 --order-parameter abs_mean_vx --kappa-c-method baseline_threshold --epsilon 1e-3 --out-root runs/matching_field_scan --overwrite-summary
python scripts/plot_matching_field.py runs/matching_field_scan/matching_field.csv --show-matching --no-show
```

---

## 阶段 5.2：结构因子 S(k) 后处理（vortex_positions.csv）

### 完成时间
2025-12-18

### 实现内容
1. 新增 `scripts/plot_structure_factor.py`：
   - 读取 `vortex_positions.csv`（由 `--dump-positions` 生成）
   - 在给定 `(kappa, step)` 上构造涡旋密度场并做 FFT，输出 2D 结构因子 `S(k)` 热图
   - 打印除 DC 之外的主峰位置/峰值，用于量化“有序/无序”（Abrikosov 晶格、匹配场等）
2. `scripts/run_matching_field_scan.py` 新增 `--dump-positions`：批量扫描时也可输出 `vortex_positions.csv` 供结构因子分析

### 使用方法

```bash
# 生成 vortex_positions.csv（headless 单点）
cargo run -- --headless --steps 20000 --sample-period 100 --flux-n 64 --seed 1234 --dump-positions --out-dir runs/structure_factor_demo

# 画结构因子（默认取最后一个 kappa 与最后一个 step）
python scripts/plot_structure_factor.py runs/structure_factor_demo/vortex_positions.csv --log10 --no-show
```

---

## 阶段 5.3：AI 闭环（active learning runner）

### 完成时间
2025-12-18

### 实现内容
1. 新增 `scripts/ai_closed_loop.py`：
   - 维护闭环数据集：`phase_diagram.csv`（与 `plot_phase_diagram.py` / `ai_inverse_design.py` 兼容）
   - 代理模型：bootstrap ridge（支持 `--degree`、`--lambda`、`--ensemble`）
   - 选点策略：UCB（maximize：`mean + beta*std`）或 target（`-|mean-target| + beta*std`）
   - 自动调用 Rust 二进制做 headless κ sweep，提取 `kappa_c` 并回填数据集
   - 输出 `loop_log.jsonl`（每次选点记录）与 `loop_progress.png`（best-so-far 曲线）
2. 支持：`--resume`（复用已有运行目录）、`--seed-dataset`（用已有 `phase_diagram.csv` 作为初始训练数据）、`--dry-run`

### 使用方法

```bash
# 注意：负数列表必须用 '=' 传参（argparse 会把 "-0.2,-0.5" 误判为新的选项）
python scripts/ai_closed_loop.py --build --objective maximize --iters 8 --init-random 4 --out-root runs/ai_closed_loop --flux-n-list=209 --seed-list=1234 --defect-mode-list=random --defect-spacing-list=32 --alpha-defect-list=-0.2,-0.5 --defect-radius-list=3 --defect-count-list=0,20,50,100 --kappa-start 0.0 --kappa-end 0.05 --kappa-step 0.01 --initial-relax-steps 20000 --relax-steps 2000 --measure-steps 5000 --sample-period 100 --order-parameter abs_mean_vx --kappa-c-method baseline_threshold --epsilon 1e-3
```

---

## 阶段 5.4：收敛性与有限尺寸效应（dt/dx + nx/ny）

### 完成时间
2025-12-18

### 实现内容
1. Rust 二进制新增 `--nx/--ny`：
   - 运行时可变网格尺寸（headless/交互模式一致）
   - GPU buffer 分配、dispatch workgroups、涡旋检测与 CSV/header 全部使用 `(nx, ny)` 动态尺寸
   - `config.toml` 记录实际 `nx/ny`
2. 新增 `scripts/run_convergence_study.py` + `scripts/plot_convergence_study.py`：
   - dt 收敛：dt vs dt/2（可选保持物理时间不变）
   - dx 收敛：保持 `L=nx*dx` 常数；支持 `--scale-defects-with-dx` 保持缺陷物理尺度不变
   - 有限尺寸效应：固定目标 B（用 `--b` 量子化），扫 `nx/ny`
   - 输出 `convergence_study.csv` + `convergence_plot.png`
3. 新增 `scripts/validate_run.py`：
   - 对 `vortices.csv` 做 schema 检查与基础 sanity checks（`net≈flux_n`、能量下降等）

### Smoke 示例输出

- `runs/convergence_dt_flux64_smoke/`：dt 收敛（`convergence_plot.png`）
- `runs/convergence_dx_smoke/`：dx 收敛（`convergence_plot.png`）
- `runs/finite_size_smoke/`：有限尺寸效应（`convergence_plot.png`）

---

## 阶段 5.5：可信度加固（refined 收敛 + AI 反演精度）
### 完成时间
2025-12-18

### 实施内容
1. refined 收敛/有限尺寸输出
   - dx 收敛（κ step=0.005）：`runs/convergence_dx_flux64_refined/`
   - 有限尺寸（κ step=0.005）：`runs/finite_size_refined/`
2. AI 反演精度评估（离线）
   - 数据集：`runs/phase_diagram_ai_eval_128/phase_diagram.csv`（45 点）
   - 工具：`scripts/evaluate_ai_inversion.py --fill-missing-with-kappa-end`
3. AI target 闭环示例（误差曲线）
   - `runs/ai_inversion_target_lattice_refined/`（objective=target，best_abs_err=0）
4. 工具链补强
   - `scripts/ai_closed_loop.py`：objective=target 时进度图改为 `|kappa_c-target|`；并传递 `--nx/--ny`
   - `scripts/run_depinning_phase_diagram.py`/`scripts/run_matching_field_scan.py`：传递 `--nx/--ny`
   - `scripts/validate_run.py`：增加 `phi=2πn/(nx*ny)` 与 `B=phi/dx^2` 的一致性检查
