# Doc Index

<!--
ASCII-only header (keep this block) to avoid a Windows apply_patch UTF-8 slicing bug.
Doc note: Scripts index includes AI inversion + closed-loop tooling.
Doc note: prefer ^2 / e-notation in text to avoid Windows console superscript issues.
Doc note: optional smoke outputs can be found under ../runs/*_smoke.
Doc note: refined outputs can be found under ../runs/*_refined and ../runs/phase_diagram_ai_eval_128.
Doc note: default --out-dir is ../runs/<mode>_<unix_ms> (pass --out-dir . for legacy cwd output).
Doc note: Python deps are listed in ../requirements.txt.
Pad: 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
-------------------------------------------------------------------------------
-->

## 文档入口

- [RESEARCH_ROADMAP.md](RESEARCH_ROADMAP.md) - 深度分析与研究路线图（建议从这里开始）
- [IMPLEMENTATION_LOG.md](IMPLEMENTATION_LOG.md) - 工程实施日志（阶段记录/性能数据/实现要点）
- [Rust_wgpu_TDGL_AI_Trial_Doc.md](Rust_wgpu_TDGL_AI_Trial_Doc.md) - 项目需求文档 + AI 工作流提示词

## 脚本

- `scripts/plot_vortices.py` - 从 `vortices.csv` 绘制时间序列曲线（支持 pinned/speed）
- `scripts/plot_kappa_sweep.py` - 从 `kappa_sweep.csv` 绘制 depinning 曲线（支持 `--order-parameter` 与 `--kappa-c-method`）
- `scripts/run_depinning_phase_diagram.py` - 自动扫参并提取 `kappa_c`（支持 `--order-parameter/--kappa-c-method/--initial-relax-steps`，输出 `phase_diagram.csv`）
- `scripts/plot_phase_diagram.py` - 从 `phase_diagram.csv` 绘制热图（相图）
- `scripts/run_convergence_study.py` - 收敛性(dt/dx)与有限尺寸效应：运行 κ sweep 并汇总到 `convergence_study.csv`
- `scripts/plot_convergence_study.py` - 从 `convergence_study.csv` 绘制 `kappa_c` 收敛/尺寸曲线
- `scripts/validate_run.py` - 单次运行输出自检（schema + sanity checks：net≈flux_n、能量下降等）
- `scripts/evaluate_ai_inversion.py` - AI 反演精度评估（离线：基于数据集做 inversion error 统计）
- `scripts/ai_inverse_design.py` - 基线 AI 反演/逆向设计：ridge 代理模型 + 离散网格搜索（输入 `phase_diagram.csv`）
- `scripts/ai_closed_loop.py` - AI 闭环：bootstrap ridge 代理模型 + acquisition 选点 + 自动仿真回填（输出 `phase_diagram.csv` + `loop_log.jsonl` + `loop_progress.png`）
- `scripts/run_matching_field_scan.py` - 匹配场（commensurability）扫描：扫 `flux_n` 对比 random vs lattice（输出 `matching_field.csv`）
- `scripts/plot_matching_field.py` - 从 `matching_field.csv` 绘制 `kappa_c(flux_n)` 曲线（可标注匹配场）
- `scripts/plot_structure_factor.py` - 从 `vortex_positions.csv` 计算/绘制结构因子 `S(k)`（需要运行时使用 `--dump-positions`）

## Smoke 输出（可选）

用于快速展示“仿真→CSV→脚本→图/指标→AI”的最小链路（可通过对应命令复现）：

- `runs/matching_field_smoke/`：`matching_field.csv` + `matching_field_plot.png`
- `runs/structure_factor_smoke/`：`vortex_positions.csv` + `structure_factor_kappa_0_step_5000.png`
- `runs/ai_closed_loop_smoke/`：`phase_diagram.csv` + `loop_log.jsonl` + `loop_progress.png`
- `runs/convergence_dt_flux64_smoke/`：`convergence_study.csv` + `convergence_plot.png`
- `runs/convergence_dx_smoke/`：`convergence_study.csv` + `convergence_plot.png`
- `runs/convergence_dx_flux64_refined/`：`convergence_study.csv` + `convergence_plot.png`（κ step=0.005）
- `runs/finite_size_smoke/`：`convergence_study.csv` + `convergence_plot.png`
- `runs/finite_size_refined/`：`convergence_study.csv` + `convergence_plot.png`（κ step=0.005）
- `runs/phase_diagram_ai_eval_128/`：AI 反演精度评估数据集（`phase_diagram.csv`）
- `runs/ai_inversion_target_smoke/`：目标反演闭环示例（`phase_diagram.csv` + `loop_log.jsonl` + `loop_progress.png`）
- `runs/ai_inversion_target_lattice_refined/`：目标反演闭环示例（`phase_diagram.csv` + `loop_log.jsonl` + `loop_progress.png`，best_abs_err=0）

## 顶层文档

- [../README.md](../README.md) - 项目简介与运行方式
- [../REPORT.md](../REPORT.md) - 课程论文（摘要/介绍/正文/讨论/参考文献）
