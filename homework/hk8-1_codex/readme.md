# hk8-1_codex - Hyperion 混沌模拟 (Codex 版本)

使用 clap 命令行参数解析的 Hyperion 混沌自转模拟器。

## 功能特性

- 完整的命令行接口 (CLI)
- 支持 Benettin 归一化方法估计 Lyapunov 指数
- 多偏心率批量模拟
- 可配置的模拟参数

## 命令行选项

```
--eccentricity <e>      单个偏心率 (默认 0.0)
--eccentricities <e1,e2,...>  多个偏心率
--duration <years>      模拟时长 (默认 2.0)
--dt <step>            时间步长 (默认 1e-4)
--theta-offset <rad>    初始角度偏移 (默认 0.01)
--renormalize          启用 Benettin 归一化
--delta-min <val>      Lyapunov 估计下限
--delta-max <val>      Lyapunov 估计上限
--output-dir <dir>     输出目录 (默认 plots)
--no-plots             跳过绘图
```

## Lyapunov 估计方法

1. **线性回归**: 对 ln|Δθ| vs t 进行线性拟合
2. **Benettin 归一化**: 当 Δθ 超过阈值时归一化并累计拉伸因子

## 运行示例

```bash
# 单个偏心率
cargo run -- --eccentricity 0.3

# 多个偏心率
cargo run -- --eccentricities 0.0,0.1,0.2,0.3,0.4,0.5

# 启用 Benettin 方法
cargo run -- --eccentricity 0.3 --renormalize
```

## 输出

- `plots/delta_e*.html` - Δθ(t) 曲线
- `plots/lyapunov_vs_eccentricity.html` - λ 随 e 变化
