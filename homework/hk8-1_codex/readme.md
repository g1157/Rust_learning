# 项目

## 作业题目
4.19. Study the behavior of our model for Hyperion for different initial conditions.
Estimate the Lyapunov exponent from calculations of 6.01 such as those shown
in Figure 4.19. Examine how this exponent varies as a function of the eccentricity
of the orbit.

FIGURE 4.19: Divergence of two nearby trajectories of the tumbling motion of Hyperion. We plot the difference between two calculated results for O(t) with different initial conditions. We used 0(0) = 0 for one trajectory and 0(0) = 0.01 for the other. In all cases the initial w was zero. Left: calculated for a circular orbit (as considered in Figure 4.17); right: calculated for an elliptical orbit (the same ellipse as used in Figure 4.18).

FIGURE 4.18: Tumbling of Hyperion calculated assuming an elliptical orbit. The initial distance from Hyperion to Saturn was 1 HU (Hyperion unit) and its initial velocity was 5 HU /Hyper ion -year, (we again took GMsat = 4PI^2 ) . The time step was 0.0001 Hyperion - year. The tumbling is now chaotic.

## 作业要求
- 编写rust代码 要求用plot和scatter绘图，绘制html
- 提交代码和绘图html文件

## 使用说明
### 环境准备
- Rust 1.80+（项目 `Cargo.toml` 使用 Edition 2024）
- 依赖通过 `cargo` 自动获取：`anyhow`, `clap`, `plotly`

### 构建、测试
```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

### 运行模拟并输出图像
```bash
cargo run -- \
  --eccentricities 0.0,0.2,0.4 \
  --duration 10.0 \
  --theta-offset 0.01 \
  --delta-min 1e-8 \
  --delta-max 0.3 \
  --output-dir plots
```
- `--eccentricity e`：单一偏心率；或使用 `--eccentricities e1,e2` 一次跑多组（会忽略 `--eccentricity`）。
- `--duration`：总模拟时间，单位为 Hyperion 年。
- `--dt`：时间步长，默认 `1e-4`。
- `--theta-offset`：第二条轨迹的初始角度偏移，用于计算 Δθ。
- `--delta-min/--delta-max`：筛选用于线性拟合的 Δθ 范围，避免初期噪声与饱和值。
- `--output-dir`：Plotly 生成的 HTML 输出目录；若加 `--no-plots` 则只打印数值结果。

程序会：
1. 为每个偏心率运行两条仅差 `θ_offset` 的轨迹，使用 Euler-Cromer 方法积分 Hyperion 模型。
2. 在 Δθ 满足区间时对 `ln|Δθ|` 与时间做最小二乘拟合，得到 Lyapunov 指数。
3. 输出四个 HTML：每个偏心率对应 `plots/delta_e*.html`（Δθ 对时间的对数曲线），以及 `plots/lyapunov_vs_eccentricity.html`（Lyapunov-偏心率散点图）。

## Notebook对内容的分析 
1. Hyperion 模型及其混沌行为
一、 模拟模型（土卫七的简化模型）
该模拟采用一个简化的“哑铃”模型来代表不规则形状的土卫七
：
1. 组成部分： 模型由两个粒子 m1​ 和 m2​ 构成，它们由一根无质量刚性杆连接
。
2. 轨道： 整个物体绕着一个位于原点的**大质量物体（土星 Msat​）**运行
。
3. 变量： 描述该模型需要跟踪以下变量：
    ◦ 质心位置： xc​ 和 yc​
。
    ◦ 角度： 杆与 x 轴的夹角 θ
。
    ◦ 角速度： ω=dθ/dt
。
二、 物理公式与运动方程
模拟土卫七的运动需要同时处理其质心的轨道运动和绕质心的旋转运动。
1. 质心运动 (Translational Motion)
物体的质心 (xc​,yc​) 的运动遵循标准的行星运动定律，与之前章节中的行星运动程序类似
。如果忽略其他卫星（如土卫六）的影响，质心主要受土星的引力作用。
2. 旋转运动 (Rotational Motion)
旋转运动由引力产生的扭矩驱动，扭矩与角加速度相关
。
• 角速度定义： $$\frac{d\theta}{dt} = \omega \quad \text{$$
$$}$$
• 引力公式 (作用于单个粒子)： 作用在粒子 m1​ 上的引力 F1​ 为
： $$\mathbf{F}_1 = - \frac{G M_{sat} m_1}{r_1^3} (x_1 \mathbf{i} + y_1 \mathbf{j}) \quad \text{(公式 4.21)} \text{$$$$}$$ 其中 r1​ 是 m1​ 到土星的距离，i 和 j 是 x 和 y 方向的单位向量
。
• 扭矩与角加速度的关系： 作用在整个模型上的总扭矩 Γ 等于转动惯量 I 乘以角加速度 dω/dt
。转动惯量 I 是 m1​ 和 m2​ 关于质心的转动惯量之和： $$\mathbf{\Gamma} = I \frac{d\omega}{dt} \quad \text{$$
$$}$$
• 角加速度的最终推导公式： 将引力产生的扭矩代入，并假设连接两质量的杆长度比土星到质心的距离 rc​ 小得多
，最终推导出的角加速度（或角速度的时间导数）为： $$\frac{d\omega}{dt} = - \frac{3 G M_{sat}}{r_c^5 I} (x_c \sin \theta - y_c \cos \theta) (x_c \cos \theta + y_c \sin \theta) \quad \text{(公式 4.24)} \text{$$$$}$$ 其中 rc​ 是质心到土星的距离，(xc​,yc​) 是质心坐标
。
三、 数值求解方法
为了求解上述微分方程，该节采用了数值方法：
• Euler-Cromer方法： 旋转运动（涉及 θ 和 ω）的方程以及质心运动的方程都被转化为差分方程，并使用 Euler-Cromer 方法进行迭代求解
。
• 迭代步骤： 在每个时间步 Δt 中，程序首先更新速度（包括质心速度和角速度），然后利用新的速度值更新位置（包括质心位置和角度 θ）
。这确保了能量在每个轨道周期中能够守恒
。
• 角度处理： 在计算中，角度 θ 会被“重置”以保持在 [−π,π] 的范围内，因为相差 2π 的角度对应于相同的角方向
。
。
2. 计算 Δθ 和估计李雅普诺夫指数 (λ)
混沌系统的一个基本特征是对初始条件的极其敏感性
。为了定量研究这种敏感性并确定系统是否为混沌系统，需要计算两个略微不同的初始条件下运行的系统的轨迹差异
。
• 差异计算 (Δθ)： 通过计算两个相近轨迹 θ1​(t) 和 θ2​(t) 之间的差值 Δθ=(θ1​−θ2​)2​ 来衡量轨迹的发散程度
。
• 李雅普诺夫指数的估计： 轨迹的发散通常可以用指数形式 Δθ∼eλt 来描述，其中 λ 被称为李雅普诺夫指数
。
    ◦ 如果 λ 为正值 (λ>0)， 则表明系统轨迹发散，运动是混沌的且具有不可预测性
。
    ◦ 如果 λ 为负值 (λ<0) 或近似为零， 则表明轨迹不会指数发散，运动是非混沌的
。
通过对 log(Δθ) 随时间 t 变化的图进行定性估计，可以从直线的斜率中估算出相应的李雅普诺夫指数
。
3. 李雅普诺夫指数与轨道偏心率的关系
您的查询要求考察李雅普诺夫指数如何随轨道偏心率变化
。来源中的模拟结果对比清楚地揭示了这种关系：
• 圆形轨道（低偏心率）： 对于圆形轨道（零偏心率），两个相邻的轨迹 θ1​(t) 和 θ2​(t) 之间的差异 Δθ 仅非常缓慢地增长（参见图 4.19 左侧），表明运动不是混沌的
。在这种情况下，李雅普诺夫指数 λ 将是负值或接近于零
。
• 椭圆形轨道（高偏心率）： 对于椭圆形轨道（非零偏心率），Δθ 随时间快速增长，大约呈指数级增长，直到达到 π 左右的数值并饱和（参见图 4.19 右侧），这是混沌行为的标志
。在这种情况下，李雅普诺夫指数 λ 将是正值
。
因此，通过计算 Δθ，可以观察到李雅普诺夫指数随轨道偏心率的增加而从接近零或负值变为正值，确认 Hyperion 模型在椭圆形轨道下进入混沌状态
。
