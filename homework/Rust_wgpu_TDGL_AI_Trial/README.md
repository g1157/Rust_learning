# Rust + wgpu TDGL 超导涡旋模拟

基于 Rust + wgpu(WebGPU) 的 GPU 加速二维 TDGL 方程求解器，用于研究超导涡旋与钉扎现象。

## 功能特性

- GPU 并行求解 TDGL 方程（compute shader）
- 实时可视化 |ψ| 热力图（无 CPU 回读）
- 空间变化的钉扎势 α(r)
- 涡旋检测与统计（相位绕数算法）
- 性能基准测试

## 物理模型

Gauge-covariant TDGL 方程（含矢势）：

```
∂ψ/∂t = (∇ - iA)²ψ + α(r)ψ - |ψ|²ψ
```

- ψ：复数序参量（超导序参）
- A：矢势（Landau gauge: A = (0, Bx, 0)）
- B：外磁场强度
- α(r)：材料参数场（缺陷处 α < 0）
- 边界条件：周期边界
- 时间推进：显式 Euler + link 变量

## 快速开始

```bash
# 交互式可视化
cargo run

# 性能基准测试
cargo run -- --bench
```

### 交互控制

- `A` 键：切换显示 |ψ| / α 场
- 关闭窗口退出

## 输出文件

- `vortices.csv`：涡旋统计数据（step, time, vortices, antivortices, net）

## 数值参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| NX, NY | 256 | 网格尺寸 |
| dt | 0.01 | 时间步长（< dx²/4 = 0.25 稳定性条件）|
| dx | 1.0 | 空间步长 |
| α_default | 1.0 | 超导区材料参数 |
| α_defect | -0.5 | 缺陷区材料参数 |
| B | 0.02 | 外磁场强度 |
| 缺陷数量 | 50 | 随机圆形缺陷 |
| 缺陷半径 | 3 | 像素 |

## 性能数据

RTX 4060 Laptop GPU 基准测试：

| 网格规模 | steps/s | cells/s |
|----------|---------|---------|
| 128² | 29,833 | 4.89×10⁸ |
| 256² | 28,410 | 1.86×10⁹ |
| 512² | 22,310 | 5.85×10⁹ |
| 1024² | 13,329 | 1.40×10¹⁰ |

## 技术实现

- **GPU 框架**：wgpu 23 (WebGPU)
- **窗口**：winit 0.30 (ApplicationHandler)
- **渲染**：全屏三角形 + fragment shader 采样
- **计算**：ping-pong buffer 交替更新
- **涡旋检测**：CPU 端相位绕数算法，定期采样

## 项目结构

```
Rust_wgpu_TDGL_AI_Trial/
├── Cargo.toml
├── README.md
├── src/
│   └── main.rs          # 主程序（compute + render + vortex detection）
├── scripts/
│   └── plot_vortices.py # 涡旋统计可视化脚本
├── doc/
│   ├── IMPLEMENTATION_LOG.md    # 实施日志
│   └── Rust_wgpu_TDGL_AI_Trial_Doc.md  # 需求文档
├── vortices.csv         # 运行时生成
└── vortices_plot.png    # 可视化图表
```

## 依赖

```toml
wgpu = "23"
winit = "0.30"
pollster = "0.4"
bytemuck = { version = "1", features = ["derive"] }
rand = "0.8"
env_logger = "0.11"
log = "0.4"
```

## 物理验证

- CPU/GPU 单步结果最大差异：7.45×10⁻⁹
- 涡旋数 N_v(t) 符合弛豫动力学（指数衰减）
- net = 0 符合周期边界拓扑守恒

## 报告

详细的物理背景、数值方法和实验结果请参阅 [REPORT.md](REPORT.md)。

## 后续计划

- [ ] 去钉扎阈值实验（扫缺陷强度/密度）
- [ ] dt/dx 收敛性验证
- [ ] 涡旋位置可视化叠加
- [ ] 参数配置文件支持

## 许可证

MIT
