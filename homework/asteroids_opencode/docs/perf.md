# 性能分析指南

## 概述

本项目集成了性能监控和分析工具，帮助开发者识别和解决性能问题。

## 性能监控

### 实时监控
运行游戏时按 `F3` 键切换性能监控覆盖层，显示：
- FPS（当前/平均/最低）
- 帧时间（毫秒）
- 实体数量统计
- Hitch检测（卡顿计数）
- 内存使用估算
- 网络延迟（在线模式）

### 性能预算
项目目标性能指标：
- **平均FPS**: ≥55
- **平均帧时间**: ≤18.2ms
- **最大帧时间**: ≤33.3ms
- **Hitch计数**: <3（每300帧）

## 性能分析工具

### puffin集成
项目支持puffin性能分析器，用于详细的CPU性能分析。

#### 启用分析
```bash
cargo run --features profiling
```

#### 分析步骤
1. 启动带profiling的构建
2. 运行游戏进行分析
3. 连接puffin viewer查看结果

#### puffin viewer
```bash
# 安装puffin viewer
cargo install puffin_viewer

# 运行viewer
puffin_viewer
```

### CI性能测试
CI流水线包含性能冒烟测试：
```bash
cargo run --release --features profiling -- --frames 300 --dump-metrics perf.json
```

## 性能优化技巧

### 常见瓶颈
1. **实体迭代**: 避免在热路径中遍历大量实体
2. **内存分配**: 减少临时对象创建
3. **碰撞检测**: 使用空间分区优化
4. **渲染调用**: 批量渲染减少draw calls

### 优化工具
- 使用 `profile_scope!` 宏标记关键代码段
- 分析 `perf.json` 输出识别热点
- 使用 `cargo flamegraph` 生成火焰图

### 内存优化
- 预分配Vec容量
- 重用对象池
- 避免字符串分配

## 基准测试

### 运行基准
```bash
# 基础性能测试
cargo run --release -- --frames 600 --entities 1000

# 压力测试
cargo run --release -- --frames 600 --entities 5000

# 网络性能测试
cargo run --release -- --frames 600 --network-test
```

### 基准指标
- **1000实体**: FPS ≥58, 内存 <50MB
- **5000实体**: FPS ≥55, 内存 <200MB
- **网络延迟**: <100ms RTT

## 故障排除

### 性能下降排查
1. 检查实体数量是否异常增加
2. 查看Hitch计数是否上升
3. 分析内存使用趋势
4. 使用puffin定位热点函数

### 常见问题
- **FPS突然下降**: 检查GC压力或内存泄漏
- **内存持续增长**: 检查对象池是否正确重用
- **网络延迟增加**: 检查服务器负载或连接质量

## 贡献指南

提交性能改进时，请：
1. 包含before/after基准数据
2. 解释优化原理
3. 确保不影响功能正确性
4. 更新相关文档