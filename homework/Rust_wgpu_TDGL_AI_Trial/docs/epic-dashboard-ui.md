# TDGL Simulator Dashboard - 棕地增强 Epic

> 生成时间: 2025-12-25
> 状态: Draft
> 来源规格: docs/front-end-spec.md

---

## Epic 目标

为 TDGL 超导涡旋模拟器添加基于 egui 的统一交互式 Dashboard，取代当前分散的 CLI 命令，提供参数调节、实时可视化和数据分析的一体化界面。

---

## Epic 描述

### 现有系统上下文

| 项目 | 说明 |
|------|------|
| 当前功能 | GPU 加速 TDGL 方程求解、涡旋检测、热力图渲染、κ sweep |
| 技术栈 | Rust + wgpu 23 + winit 0.30 + egui 0.30 (已添加依赖) |
| 集成点 | `src/main.rs` 中的渲染循环、`RunConfig` 参数结构、涡旋统计数据 |
| 现有模式 | Interactive / Headless / HeadlessKappaSweep / Bench |

### 增强详情

| 项目 | 说明 |
|------|------|
| 添加内容 | 三栏 Dashboard 布局（参数控制 / 可视化 / 数据状态） |
| 集成方式 | egui 叠加在现有 wgpu 渲染之上，共享 GPU 上下文 |
| 成功标准 | 新用户 5 分钟内能启动仿真；常用操作 ≤3 次点击 |

---

## 用户故事

### Story 1: 基础框架与参数控制面板

**用户故事：**

> 作为研究者，我希望通过图形界面调整仿真参数（flux_n、κ、缺陷配置等），以便快速迭代实验而无需修改命令行参数。

**验收标准：**

1. egui 成功集成到现有 wgpu 渲染循环
2. 左侧面板显示可折叠的参数组（仿真参数、缺陷配置、运行模式）
3. 参数变更实时绑定到 `RunConfig`
4. 运行控制按钮（开始/暂停/停止/重置）功能正常
5. 深色主题应用，符合 front-end-spec 配色方案
6. 现有热力图渲染不受影响

**技术要点：**

- 集成 `egui-wgpu` 和 `egui-winit`
- 创建 `src/ui/` 模块结构
- 实现 `ParamSlider`、`ParamInput` 等参数组件

**预估复杂度：** 中等 (2-3 天)

---

### Story 2: 数据展示与状态面板

**用户故事：**

> 作为研究者，我希望实时查看涡旋统计、能量变化和时间序列图，以便在仿真过程中监控系统状态。

**验收标准：**

1. 右侧面板显示实时统计（涡旋数、钉扎率、能量密度、速度）
2. 时间序列图（egui_plot）显示最近 1000 步的涡旋数和能量
3. 底部状态栏显示进度、ETA、GPU 信息、steps/s
4. 涡旋位置叠加显示在热力图上（红色涡旋、青色反涡旋）
5. 鼠标悬停显示坐标和 |ψ| 值
6. 数值变化时有视觉高亮反馈

**技术要点：**

- 添加 `egui_plot` 依赖
- 实现 `VortexOverlay` 组件
- 创建 `StatCard`、`TimeSeriesPlot` 组件

**预估复杂度：** 中等 (2-3 天)

---

### Story 3: 高级功能与优化

**用户故事：**

> 作为研究者，我希望通过 Dashboard 执行 κ Sweep 扫参并查看实时 depinning 曲线，以及浏览和复现历史运行配置。

**验收标准：**

1. κ Sweep 模式 UI：配置 sweep 参数、显示实时进度和曲线
2. Depinning 曲线实时绘制，自动标注 κ_c
3. 运行历史浏览：列出 `runs/` 目录下的历史运行
4. 配置预设系统：保存/加载当前参数配置
5. 动画效果：运行按钮脉冲、数值高亮、涡旋出现动画
6. 性能优化：UI 响应 < 16ms，不影响仿真帧率

**技术要点：**

- 实现 `DepinningCurve` 组件
- 添加 `directories` crate 用于配置存储
- 实现动画系统（`src/utils/animation.rs`）

**预估复杂度：** 高 (3-4 天)

---

## 兼容性要求

- [x] 现有 CLI 参数保持不变
- [x] Headless 模式不受 UI 影响
- [x] 输出文件格式（vortices.csv、config.toml）保持兼容
- [x] 性能影响最小（目标：仿真帧率 ≥ 60 FPS）

---

## 风险缓解

| 风险 | 缓解措施 | 回滚方案 |
|------|----------|----------|
| egui 与 wgpu 23 兼容性问题 | 使用 egui 0.30 (已验证兼容) | 回退到纯 CLI 模式 |
| UI 影响仿真性能 | 仿真与 UI 分离线程 | 禁用 UI 叠加 |
| 代码复杂度增加 | 模块化设计 (`src/ui/`) | 保留原 main.rs 结构 |

---

## 完成定义

- [ ] 所有 3 个故事完成并通过验收标准
- [ ] 现有功能（CLI、Headless、κ Sweep）回归测试通过
- [ ] 性能基准：256² 网格 ≥ 25,000 steps/s
- [ ] 文档更新：README.md、CLAUDE.md

---

## 验证清单

### 范围验证

- [x] Epic 可在 3 个故事内完成
- [x] 无需架构文档（遵循现有 wgpu + egui 模式）
- [x] 增强遵循现有模式（Rust + wgpu）
- [x] 集成复杂度可控

### 风险评估

- [x] 对现有系统风险低（UI 为叠加层）
- [x] 回滚方案可行（移除 egui 集成代码）
- [x] 测试覆盖现有功能
- [x] 团队熟悉 egui + wgpu 集成

---

## Story Manager 交接

请为此棕地 Epic 开发详细的用户故事。关键考虑：

- 这是对运行 **Rust + wgpu 23 + egui 0.30** 的现有系统的增强
- 集成点：`src/main.rs` 渲染循环、`RunConfig` 结构、涡旋统计数据
- 遵循的现有模式：wgpu compute/render pipeline、winit 事件循环
- 关键兼容性要求：CLI 参数、Headless 模式、输出文件格式
- 每个故事必须包含验证现有功能保持完整的测试

Epic 应在交付 **Dashboard UI 一体化界面** 的同时保持系统完整性。

---

## 附录：文件结构规划

```
src/
├── main.rs              # 入口 (修改: 集成 egui)
├── app.rs               # 应用状态 (新增)
├── sim/                 # 仿真引擎 (新增模块)
│   ├── mod.rs
│   ├── engine.rs
│   ├── params.rs
│   └── stats.rs
├── ui/                  # UI 层 (新增)
│   ├── mod.rs
│   ├── theme.rs
│   ├── panels/
│   │   ├── mod.rs
│   │   ├── params_panel.rs
│   │   └── stats_panel.rs
│   └── components/
│       ├── mod.rs
│       ├── param_slider.rs
│       ├── stat_card.rs
│       └── time_series.rs
├── viz/                 # 可视化 (新增)
│   ├── mod.rs
│   ├── heatmap.rs
│   ├── vortex_overlay.rs
│   └── colormap.rs
└── utils/               # 工具 (新增)
    ├── mod.rs
    ├── animation.rs
    └── history.rs
```

---

*此文档由 BMad Product Owner (Sarah) 生成*
