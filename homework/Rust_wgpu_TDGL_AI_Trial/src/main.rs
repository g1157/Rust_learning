//! TDGL 阶段 2+3+4：缺陷/钉扎势 + 可视化 + 涡旋检测

use std::borrow::Cow;
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use rand::Rng;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

// 网格参数
const NX: u32 = 256;
const NY: u32 = 256;
const GRID_SIZE: usize = (NX as usize) * (NY as usize);

// 数值参数
const DT: f32 = 0.01;
const DX: f32 = 1.0;
const B_FIELD: f32 = 0.02; // 外磁场强度（Landau gauge: A = (0, Bx, 0)）

// 缺陷参数
const DEFAULT_ALPHA: f32 = 1.0;
const DEFECT_ALPHA: f32 = -0.5;
const DEFECT_RADIUS: i32 = 3;
const DEFECT_COUNT: usize = 50;

// GPU 参数
const WORKGROUP_X: u32 = 8;
const WORKGROUP_Y: u32 = 8;
const STEPS_PER_FRAME: u32 = 10;

// 涡旋检测参数
const VORTEX_SAMPLE_PERIOD: u64 = 100; // 每 100 步采样一次

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct Complex { re: f32, im: f32 }

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct Params {
    nx: u32,
    ny: u32,
    show_alpha: u32,
    _pad0: u32,
    dt: f32,
    dx: f32,
    b_field: f32,  // 外磁场强度
    _pad1: f32,
}

// TDGL compute shader（Gauge-covariant，含磁场）
const COMPUTE_SHADER: &str = r#"
struct Params { nx: u32, ny: u32, show_alpha: u32, _pad0: u32, dt: f32, dx: f32, B: f32, _pad1: f32, }

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> psi_in: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> psi_out: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read> alpha: array<f32>;

fn idx(x: u32, y: u32) -> u32 { return y * params.nx + x; }
fn wrap(v: i32, e: u32) -> u32 { let n = i32(e); return u32((v % n + n) % n); }
fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> { return vec2(a.x*b.x - a.y*b.y, a.x*b.y + a.y*b.x); }
fn conj(a: vec2<f32>) -> vec2<f32> { return vec2(a.x, -a.y); }

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.nx || gid.y >= params.ny { return; }
    let i = idx(gid.x, gid.y);
    let psi = psi_in[i];
    let xp = wrap(i32(gid.x)+1, params.nx); let xm = wrap(i32(gid.x)-1, params.nx);
    let yp = wrap(i32(gid.y)+1, params.ny); let ym = wrap(i32(gid.y)-1, params.ny);

    // Landau gauge: A = (0, Bx, 0), link Uy = exp(-i B x dx)
    let theta = -params.B * f32(gid.x) * params.dx;
    let Uy = vec2(cos(theta), sin(theta));

    // Gauge-covariant Laplacian
    let psi_xp = psi_in[idx(xp, gid.y)];  // Ux = 1
    let psi_xm = psi_in[idx(xm, gid.y)];
    let psi_yp = cmul(Uy, psi_in[idx(gid.x, yp)]);
    let psi_ym = cmul(conj(Uy), psi_in[idx(gid.x, ym)]);
    let lap = (psi_xp + psi_xm + psi_yp + psi_ym - 4.0*psi) / (params.dx*params.dx);

    let mag2 = dot(psi, psi);
    let rhs = lap + alpha[i] * psi - psi * mag2;
    psi_out[i] = psi + params.dt * rhs;
}
"#;

// 渲染 shader（支持切换显示 psi/alpha）
const RENDER_SHADER: &str = r#"
struct Params { nx: u32, ny: u32, show_alpha: u32, _pad0: u32, dt: f32, dx: f32, B: f32, _pad1: f32, }

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> psi: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read> alpha: array<f32>;

struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32>, }

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var p = array<vec2<f32>,3>(vec2(-1.0,-3.0), vec2(3.0,1.0), vec2(-1.0,1.0));
    var u = array<vec2<f32>,3>(vec2(0.0,2.0), vec2(2.0,0.0), vec2(0.0,0.0));
    var out: VsOut; out.pos = vec4(p[vi], 0.0, 1.0); out.uv = u[vi]; return out;
}

fn colormap(t: f32) -> vec3<f32> {
    let c1 = vec3(0.1,0.2,0.6); let c2 = vec3(1.0,1.0,1.0); let c3 = vec3(0.8,0.1,0.1);
    if t < 0.5 { return mix(c1, c2, t*2.0); } else { return mix(c2, c3, (t-0.5)*2.0); }
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let x = u32(clamp(in.uv.x, 0.0, 0.999) * f32(params.nx));
    let y = u32(clamp(in.uv.y, 0.0, 0.999) * f32(params.ny));
    let i = y * params.nx + x;
    if params.show_alpha != 0u {
        let a = clamp((alpha[i] + 1.0) * 0.5, 0.0, 1.0);
        return vec4(colormap(a), 1.0);
    }
    let mag = sqrt(dot(psi[i], psi[i]));
    return vec4(colormap(clamp(mag, 0.0, 1.0)), 1.0);
}
"#;

fn main() {
    env_logger::init();

    // 命令行参数：--bench 运行性能基准测试
    if std::env::args().any(|a| a == "--bench") {
        pollster::block_on(run_benchmark());
        return;
    }

    let event_loop = EventLoop::new().expect("创建事件循环失败");
    let mut app = App { state: None };
    event_loop.run_app(&mut app).expect("运行失败");
}

/// 性能基准测试（无窗口）
async fn run_benchmark() {
    println!("=== TDGL GPU 性能基准测试 ===\n");

    let instance = wgpu::Instance::default();
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        ..Default::default()
    }).await.expect("未找到 GPU");

    println!("GPU: {}", adapter.get_info().name);
    println!();

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();

    let grid_sizes = [128u32, 256, 512, 1024];
    let steps = 500;

    println!("{:>10} {:>12} {:>15}", "网格", "steps/s", "cells/s");
    println!("{}", "-".repeat(40));

    for &n in &grid_sizes {
        let rate = bench_grid(&device, &queue, n, steps);
        let cells_per_sec = rate * (n * n) as f64;
        println!("{:>8}² {:>12.1} {:>12.2e}", n, rate, cells_per_sec);
    }

    println!("\n基准测试完成");
}

fn bench_grid(device: &wgpu::Device, queue: &wgpu::Queue, n: u32, steps: u32) -> f64 {
    let grid_size = (n * n) as usize;

    // 创建 buffers
    let psi_a = device.create_buffer(&wgpu::BufferDescriptor {
        label: None, size: (grid_size * 8) as u64, usage: wgpu::BufferUsages::STORAGE, mapped_at_creation: false,
    });
    let psi_b = device.create_buffer(&wgpu::BufferDescriptor {
        label: None, size: (grid_size * 8) as u64, usage: wgpu::BufferUsages::STORAGE, mapped_at_creation: false,
    });
    let alpha_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None, size: (grid_size * 4) as u64, usage: wgpu::BufferUsages::STORAGE, mapped_at_creation: false,
    });
    let params = Params { nx: n, ny: n, show_alpha: 0, _pad0: 0, dt: DT, dx: DX, b_field: B_FIELD, _pad1: 0.0 };
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None, contents: bytemuck::bytes_of(&params), usage: wgpu::BufferUsages::UNIFORM,
    });

    // Pipeline
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
        ],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COMPUTE_SHADER)) });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None, layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[&bgl], push_constant_ranges: &[] })),
        module: &shader, entry_point: Some("main"), compilation_options: Default::default(), cache: None,
    });
    let bg_ab = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &bgl, entries: &[
        wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
        wgpu::BindGroupEntry { binding: 1, resource: psi_a.as_entire_binding() },
        wgpu::BindGroupEntry { binding: 2, resource: psi_b.as_entire_binding() },
        wgpu::BindGroupEntry { binding: 3, resource: alpha_buf.as_entire_binding() },
    ]});
    let bg_ba = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &bgl, entries: &[
        wgpu::BindGroupEntry { binding: 0, resource: params_buf.as_entire_binding() },
        wgpu::BindGroupEntry { binding: 1, resource: psi_b.as_entire_binding() },
        wgpu::BindGroupEntry { binding: 2, resource: psi_a.as_entire_binding() },
        wgpu::BindGroupEntry { binding: 3, resource: alpha_buf.as_entire_binding() },
    ]});

    let (xg, yg) = ((n + WORKGROUP_X - 1) / WORKGROUP_X, (n + WORKGROUP_Y - 1) / WORKGROUP_Y);

    // 预热
    for i in 0..10 {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        { let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
          cp.set_pipeline(&pipeline);
          cp.set_bind_group(0, if i % 2 == 0 { &bg_ab } else { &bg_ba }, &[]);
          cp.dispatch_workgroups(xg, yg, 1); }
        queue.submit(Some(enc.finish()));
    }
    device.poll(wgpu::Maintain::Wait);

    // 计时
    let start = std::time::Instant::now();
    for i in 0..steps {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        { let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
          cp.set_pipeline(&pipeline);
          cp.set_bind_group(0, if i % 2 == 0 { &bg_ab } else { &bg_ba }, &[]);
          cp.dispatch_workgroups(xg, yg, 1); }
        queue.submit(Some(enc.finish()));
    }
    device.poll(wgpu::Maintain::Wait);
    let elapsed = start.elapsed().as_secs_f64();

    steps as f64 / elapsed
}

struct App { state: Option<State> }

struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    compute_bgs: [wgpu::BindGroup; 2],
    render_bgs: [wgpu::BindGroup; 2],
    params_buffer: wgpu::Buffer,
    params: Params,
    ping_is_a: bool,
    step_count: u64,
    // 涡旋检测
    psi_a: wgpu::Buffer,
    psi_b: wgpu::Buffer,
    readback_buffer: wgpu::Buffer,
    vortex_csv: File,
    last_vortex_sample: u64,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() { return; }
        let window = Arc::new(event_loop.create_window(
            Window::default_attributes()
                .with_title("TDGL + 缺陷 (按 A 切换显示)")
                .with_inner_size(PhysicalSize::new(512, 512)),
        ).expect("创建窗口失败"));
        self.state = Some(pollster::block_on(State::new(window)));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = match &mut self.state { Some(s) => s, None => return };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => state.resize(size),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                if let PhysicalKey::Code(KeyCode::KeyA) = event.physical_key {
                    state.toggle_alpha();
                }
            }
            WindowEvent::RedrawRequested => {
                state.update();
                state.render();
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.unwrap();
        log::info!("GPU: {:?}", adapter.get_info().name);

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, format,
            width: size.width.max(1), height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0], view_formats: vec![], desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Buffers
        let psi0 = gen_noise();
        let alpha_field = gen_alpha();
        let psi_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PsiA"), contents: bytemuck::cast_slice(&psi0),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        let psi_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PsiB"), size: (GRID_SIZE * 8) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false,
        });
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback"), size: (GRID_SIZE * 8) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false,
        });
        // CSV 文件
        let mut vortex_csv = File::create("vortices.csv").expect("创建 CSV 失败");
        writeln!(vortex_csv, "step,time,vortices,antivortices,net").unwrap();
        let alpha_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Alpha"), contents: bytemuck::cast_slice(&alpha_field), usage: wgpu::BufferUsages::STORAGE,
        });
        let params = Params { nx: NX, ny: NY, show_alpha: 0, _pad0: 0, dt: DT, dx: DX, b_field: B_FIELD, _pad1: 0.0 };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params"), contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Compute pipeline
        let comp_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });
        let comp_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COMPUTE_SHADER)) });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None, layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[&comp_bgl], push_constant_ranges: &[] })),
            module: &comp_shader, entry_point: Some("main"), compilation_options: Default::default(), cache: None,
        });
        let comp_bg_ab = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &comp_bgl, entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: psi_a.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: psi_b.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: alpha_buf.as_entire_binding() },
        ]});
        let comp_bg_ba = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &comp_bgl, entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: psi_b.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: psi_a.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: alpha_buf.as_entire_binding() },
        ]});

        // Render pipeline
        let rend_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });
        let rend_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(RENDER_SHADER)) });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None, layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[&rend_bgl], push_constant_ranges: &[] })),
            vertex: wgpu::VertexState { module: &rend_shader, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &rend_shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState::default(), depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None, cache: None,
        });
        let rend_bg_a = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &rend_bgl, entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: psi_a.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: alpha_buf.as_entire_binding() },
        ]});
        let rend_bg_b = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &rend_bgl, entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: params_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: psi_b.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: alpha_buf.as_entire_binding() },
        ]});

        Self { window, device, queue, surface, config, compute_pipeline, render_pipeline,
            compute_bgs: [comp_bg_ab, comp_bg_ba], render_bgs: [rend_bg_a, rend_bg_b],
            params_buffer, params, ping_is_a: true, step_count: 0,
            psi_a, psi_b, readback_buffer, vortex_csv, last_vortex_sample: 0 }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    fn toggle_alpha(&mut self) {
        self.params.show_alpha = if self.params.show_alpha == 0 { 1 } else { 0 };
        self.queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&self.params));
        log::info!("显示模式: {}", if self.params.show_alpha == 1 { "alpha场" } else { "|ψ|" });
    }

    fn update(&mut self) {
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let (xg, yg) = ((NX + WORKGROUP_X - 1) / WORKGROUP_X, (NY + WORKGROUP_Y - 1) / WORKGROUP_Y);
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            cp.set_pipeline(&self.compute_pipeline);
            for _ in 0..STEPS_PER_FRAME {
                cp.set_bind_group(0, &self.compute_bgs[if self.ping_is_a { 0 } else { 1 }], &[]);
                cp.dispatch_workgroups(xg, yg, 1);
                self.ping_is_a = !self.ping_is_a;
                self.step_count += 1;
            }
        }
        // 涡旋检测采样
        let should_sample = self.step_count >= self.last_vortex_sample + VORTEX_SAMPLE_PERIOD;
        if should_sample {
            let src = if self.ping_is_a { &self.psi_a } else { &self.psi_b };
            enc.copy_buffer_to_buffer(src, 0, &self.readback_buffer, 0, (GRID_SIZE * 8) as u64);
        }
        self.queue.submit(Some(enc.finish()));
        // 执行涡旋检测
        if should_sample {
            self.sample_vortices();
        }
    }

    fn sample_vortices(&mut self) {
        let slice = self.readback_buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);
        {
            let data = slice.get_mapped_range();
            let psi: &[Complex] = bytemuck::cast_slice(&data);
            let (vort, anti) = detect_vortices(psi);
            let t = self.step_count as f32 * DT;
            writeln!(self.vortex_csv, "{},{:.2},{},{},{}", self.step_count, t, vort, anti, vort - anti).unwrap();
            log::info!("涡旋检测 @ step {}: +{}/-{} net={}", self.step_count, vort, anti, vort - anti);
        }
        self.readback_buffer.unmap();
        self.last_vortex_sample = self.step_count;
    }

    fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f, Err(_) => { self.surface.configure(&self.device, &self.config); return; }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
            rp.set_pipeline(&self.render_pipeline);
            rp.set_bind_group(0, &self.render_bgs[if self.ping_is_a { 0 } else { 1 }], &[]);
            rp.draw(0..3, 0..1);
        }
        self.queue.submit(Some(enc.finish()));
        frame.present();
        if self.step_count % 100 == 0 { log::info!("步数: {}", self.step_count); }
    }
}

fn gen_noise() -> Vec<Complex> {
    let mut rng = rand::thread_rng();
    (0..GRID_SIZE).map(|_| Complex { re: rng.gen_range(-0.1..0.1), im: rng.gen_range(-0.1..0.1) }).collect()
}

fn gen_alpha() -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let mut field = vec![DEFAULT_ALPHA; GRID_SIZE];
    for _ in 0..DEFECT_COUNT {
        let (cx, cy) = (rng.gen_range(0..NX as i32), rng.gen_range(0..NY as i32));
        for dy in -DEFECT_RADIUS..=DEFECT_RADIUS {
            for dx in -DEFECT_RADIUS..=DEFECT_RADIUS {
                if dx*dx + dy*dy > DEFECT_RADIUS*DEFECT_RADIUS { continue; }
                let x = ((cx + dx + NX as i32) % NX as i32) as usize;
                let y = ((cy + dy + NY as i32) % NY as i32) as usize;
                field[y * NX as usize + x] = DEFECT_ALPHA;
            }
        }
    }
    log::info!("生成 {} 个缺陷点", DEFECT_COUNT);
    field
}

/// 相位 unwrap 到 (-π, π]
fn wrap_phase(d: f32) -> f32 {
    let tau = 2.0 * PI;
    let mut v = d;
    while v > PI { v -= tau; }
    while v <= -PI { v += tau; }
    v
}

/// 涡旋检测：相位绕数算法
/// 返回 (涡旋数, 反涡旋数)
fn detect_vortices(psi: &[Complex]) -> (i32, i32) {
    const TAU: f32 = 2.0 * PI;
    const THRESHOLD: f32 = TAU * 0.75; // 更严格的阈值，减少误判
    let (mut vort, mut anti) = (0i32, 0i32);
    for y in 0..NY {
        for x in 0..NX {
            // 四个角点索引（周期边界）
            let idx = |xx: u32, yy: u32| -> usize {
                ((yy % NY) * NX + (xx % NX)) as usize
            };
            let phase = |xx: u32, yy: u32| -> f32 {
                let c = psi[idx(xx, yy)];
                c.im.atan2(c.re)
            };
            // 四角相位
            let p00 = phase(x, y);
            let p10 = phase(x + 1, y);
            let p11 = phase(x + 1, y + 1);
            let p01 = phase(x, y + 1);
            // 绕一圈的相位差之和
            let sum = wrap_phase(p10 - p00)
                    + wrap_phase(p11 - p10)
                    + wrap_phase(p01 - p11)
                    + wrap_phase(p00 - p01);
            if sum > THRESHOLD { vort += 1; }
            else if sum < -THRESHOLD { anti += 1; }
        }
    }
    (vort, anti)
}
