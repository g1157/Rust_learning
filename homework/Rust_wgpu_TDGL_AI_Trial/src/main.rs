//! TDGL stage 2+3+4: defects/pinning + MPBC + vortex detection
//!
//! NOTE: The ASCII-only header above avoids a Windows apply_patch UTF-8 slicing bug.
//! (Chinese content continues below.)
//!
//! TDGL 阶段 2+3+4：缺陷/钉扎势 + 可视化 + 涡旋检测

use std::borrow::Cow;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bytemuck::{Pod, Zeroable};
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
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

fn grid_size(nx: u32, ny: u32) -> usize {
    (nx as usize) * (ny as usize)
}

// 数值参数
const DEFAULT_DT: f32 = 0.01;
const DEFAULT_DX: f32 = 1.0;
const DEFAULT_B_FIELD: f32 = 0.02; // 目标外磁场强度（后续会量子化到合法的磁通）
const DEFAULT_KAPPA: f32 = 0.0; // 相位扭曲/等效驱动（见 doc/RESEARCH_ROADMAP.md 方向 B）

// 缺陷参数
const DEFAULT_ALPHA: f32 = 1.0;
const DEFECT_ALPHA: f32 = -0.5;
const DEFECT_RADIUS: i32 = 3;
const DEFECT_COUNT: usize = 50;
const DEFAULT_DEFECT_SPACING: i32 = 32; // 周期缺陷阵列间距（cell），仅 defect-mode=lattice 时使用

// GPU 参数
const WORKGROUP_X: u32 = 8;
const WORKGROUP_Y: u32 = 8;
const STEPS_PER_FRAME: u32 = 10;

// 涡旋检测参数
const VORTEX_SAMPLE_PERIOD: u64 = 100; // 每 100 步采样一次
const DEFAULT_KAPPA_SWEEP_RELAX_STEPS: u64 = 2000;
const DEFAULT_KAPPA_SWEEP_MEASURE_STEPS: u64 = 5000;
const TRACK_MAX_DIST_CELLS: i32 = 16; // 涡旋追踪最近邻匹配最大距离（以 cell 为单位）

#[derive(Clone, Copy, Debug)]
enum DefectMode {
    Random,
    SquareLattice,
}

impl DefectMode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "random" => Some(Self::Random),
            "lattice" | "square" | "square_lattice" => Some(Self::SquareLattice),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Random => "random",
            Self::SquareLattice => "lattice",
        }
    }
}

#[derive(Clone, Debug)]
struct RunConfig {
    nx: u32,
    ny: u32,
    dt: f32,
    dx: f32,
    flux_n: i32,
    phi: f32, // plaquette flux: phi = B * dx^2
    kappa: f32,
    seed: u64,
    alpha_default: f32,
    alpha_defect: f32,
    defect_radius: i32,
    defect_count: usize,
    defect_mode: DefectMode,
    defect_spacing: i32,
    dump_positions: bool,
    out_dir: PathBuf,
}

#[derive(Clone, Copy, Debug)]
enum RunMode {
    Interactive,
    Bench,
    Headless {
        steps: u64,
        sample_period: u64,
    },
    HeadlessKappaSweep {
        kappa_start: f32,
        kappa_end: f32,
        kappa_step: f32,
        initial_relax_steps: u64,
        relax_steps: u64,
        measure_steps: u64,
        sample_period: u64,
    },
}

#[derive(Clone, Copy, Debug)]
struct KappaSweepConfig {
    kappa_start: f32,
    kappa_end: f32,
    kappa_step: f32,
    initial_relax_steps: u64,
    relax_steps: u64,
    measure_steps: u64,
    sample_period: u64,
}

/// Plaquette flux `phi` (dimensionless) for an `nx * ny` lattice, with total flux quanta `flux_n`.
///
/// In MPBC on a torus, the uniform field is globally consistent only when:
///   phi * nx * ny = 2*PI * flux_n
fn phi_from_flux_n(flux_n: i32, nx: u32, ny: u32) -> f32 {
    let area_cells = (nx as f32) * (ny as f32);
    (2.0 * PI) * (flux_n as f32) / area_cells
}

/// Convert a target continuous-field strength `B` to the nearest quantized `flux_n`.
///
/// `B` is the physical field in our nondimensional units; internally the solver uses:
///   phi = B * dx^2
/// then quantizes `phi` via `flux_n`.
fn flux_n_from_b_field(b_field: f32, nx: u32, ny: u32, dx: f32) -> i32 {
    let area = (nx as f32) * (ny as f32) * dx * dx;
    ((b_field * area) / (2.0 * PI)).round() as i32
}

/// Recover `B` from plaquette flux `phi` using `phi = B * dx^2`.
fn b_field_from_phi(phi: f32, dx: f32) -> f32 {
    phi / (dx * dx)
}

fn escape_toml_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 8);
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn write_config_toml(
    out_dir: &Path,
    config: &RunConfig,
    mode: &str,
    headless_steps: Option<u64>,
    headless_sample_period: Option<u64>,
    kappa_sweep: Option<KappaSweepConfig>,
) -> io::Result<()> {
    let path = out_dir.join("config.toml");
    let mut file = File::create(&path)?;

    let b = b_field_from_phi(config.phi, config.dx);
    let out_dir_str = out_dir.to_string_lossy().replace('\\', "/");

    writeln!(
        file,
        "# Auto-generated by {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )?;
    writeln!(file, "mode = \"{}\"", escape_toml_string(mode))?;
    writeln!(file, "nx = {}", config.nx)?;
    writeln!(file, "ny = {}", config.ny)?;
    writeln!(file, "dt = {:.8}", config.dt)?;
    writeln!(file, "dx = {:.8}", config.dx)?;
    writeln!(file, "flux_n = {}", config.flux_n)?;
    writeln!(file, "phi = {:.8e}", config.phi)?;
    writeln!(file, "b = {:.8e}", b)?;
    writeln!(file, "kappa = {:.8e}", config.kappa)?;
    writeln!(file, "seed = {}", config.seed)?;
    writeln!(file)?;

    writeln!(file, "[defects]")?;
    writeln!(file, "alpha_default = {:.8}", config.alpha_default)?;
    writeln!(file, "alpha_defect = {:.8}", config.alpha_defect)?;
    writeln!(file, "defect_radius = {}", config.defect_radius)?;
    writeln!(file, "defect_count = {}", config.defect_count)?;
    writeln!(
        file,
        "defect_count_effective = {}",
        defect_count_effective(
            config.defect_mode,
            config.defect_count,
            config.defect_spacing,
            config.nx,
            config.ny,
        )
    )?;
    writeln!(
        file,
        "defect_mode = \"{}\"",
        escape_toml_string(config.defect_mode.as_str())
    )?;
    writeln!(file, "defect_spacing = {}", config.defect_spacing)?;
    writeln!(file)?;

    writeln!(file, "[output]")?;
    writeln!(file, "out_dir = \"{}\"", escape_toml_string(&out_dir_str))?;
    writeln!(file, "dump_positions = {}", config.dump_positions)?;

    if let Some(steps) = headless_steps {
        writeln!(file)?;
        writeln!(file, "[headless]")?;
        writeln!(file, "steps = {}", steps)?;
        if let Some(p) = headless_sample_period {
            writeln!(file, "sample_period = {}", p)?;
        }
    }

    if let Some(sweep) = kappa_sweep {
        writeln!(file)?;
        writeln!(file, "[kappa_sweep]")?;
        writeln!(file, "kappa_start = {:.8e}", sweep.kappa_start)?;
        writeln!(file, "kappa_end = {:.8e}", sweep.kappa_end)?;
        writeln!(file, "kappa_step = {:.8e}", sweep.kappa_step)?;
        writeln!(file, "initial_relax_steps = {}", sweep.initial_relax_steps)?;
        writeln!(file, "relax_steps = {}", sweep.relax_steps)?;
        writeln!(file, "measure_steps = {}", sweep.measure_steps)?;
        writeln!(file, "sample_period = {}", sweep.sample_period)?;
    }

    Ok(())
}

fn write_meta_json(out_dir: &Path, adapter: &wgpu::Adapter, mode: &str) -> io::Result<()> {
    let path = out_dir.join("meta.json");
    let mut file = File::create(&path)?;

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let args: Vec<String> = std::env::args().collect();
    let args_json = args
        .iter()
        .map(|a| format!("\"{}\"", json_escape(a)))
        .collect::<Vec<_>>()
        .join(",");

    let info = adapter.get_info();
    let adapter_name = json_escape(&info.name);
    let backend = json_escape(&format!("{:?}", info.backend));
    let device_type = json_escape(&format!("{:?}", info.device_type));

    writeln!(file, "{{")?;
    writeln!(file, "  \"created_unix_ms\": {now_ms},")?;
    writeln!(
        file,
        "  \"package\": {{\"name\": \"{}\", \"version\": \"{}\"}},",
        json_escape(env!("CARGO_PKG_NAME")),
        json_escape(env!("CARGO_PKG_VERSION"))
    )?;
    writeln!(
        file,
        "  \"platform\": {{\"os\": \"{}\", \"arch\": \"{}\"}},",
        std::env::consts::OS,
        std::env::consts::ARCH
    )?;
    writeln!(file, "  \"mode\": \"{}\",", json_escape(mode))?;
    writeln!(file, "  \"argv\": [{args_json}],")?;
    writeln!(
        file,
        "  \"adapter\": {{\"name\": \"{adapter_name}\", \"backend\": \"{backend}\", \"device_type\": \"{device_type}\", \"vendor\": {}, \"device\": {}}}",
        info.vendor,
        info.device
    )?;
    writeln!(file, "}}")?;
    Ok(())
}

fn print_help_and_exit(exit_code: i32) -> ! {
    eprintln!(
        concat!(
        "Usage:\n  cargo run -- [--bench|--headless] [--steps <u64>] [--sample-period <u64>] [--nx <u32>] [--ny <u32>] [--dt <f32>] [--dx <f32>] [--b <f32>] [--flux-n <i32>] [--kappa <f32>] [--kappa-start <f32> --kappa-end <f32> --kappa-step <f32>] [--kappa-initial-relax-steps <u64>] [--kappa-relax-steps <u64>] [--kappa-measure-steps <u64>] [--alpha-default <f32>] [--alpha-defect <f32>] [--defect-radius <i32>] [--defect-count <usize>] [--defect-mode <random|lattice>] [--defect-spacing <i32>] [--seed <u64>] [--out-dir <path>] [--dump-positions]\n",
        "\n",
        "Options:\n",
        "  --bench                 Run GPU benchmark (no window)\n",
        "  --headless              Run simulation without a window (writes outputs under --out-dir)\n",
        "  --steps <u64>            Total steps in headless mode (default: 5000)\n",
        "  --sample-period <u64>    Vortex/energy sampling period in headless mode (default: {VORTEX_SAMPLE_PERIOD})\n",
        "  --nx <u32>              Grid size in x (default: {NX})\n",
        "  --ny <u32>              Grid size in y (default: {NY})\n",
        "  --dt <f32>              Time step (default: {DEFAULT_DT})\n",
        "  --dx <f32>              Grid spacing (default: {DEFAULT_DX})\n",
        "  --b <f32>               Target B field; will be quantized to nearest flux-n\n",
        "  --flux-n <i32>          Total flux quanta n (preferred, exact on torus)\n",
        "  --kappa <f32>           Drive (phase twist / constant Ay0) (default: {DEFAULT_KAPPA})\n",
        "  --kappa-start <f32>     Start kappa for sweep (headless only)\n",
        "  --kappa-end <f32>       End kappa for sweep (headless only)\n",
        "  --kappa-step <f32>      Step size for kappa sweep (headless only)\n",
        "  --kappa-initial-relax-steps <u64>  Relax steps for the first kappa point in sweep (default: same as --kappa-relax-steps)\n",
        "  --kappa-relax-steps <u64>   Relax steps per kappa (default: {DEFAULT_KAPPA_SWEEP_RELAX_STEPS})\n",
        "  --kappa-measure-steps <u64>  Measure steps per kappa (default: {DEFAULT_KAPPA_SWEEP_MEASURE_STEPS})\n",
        "  --alpha-default <f32>   Alpha in SC region (default: {DEFAULT_ALPHA})\n",
        "  --alpha-defect <f32>    Alpha in defect region (default: {DEFECT_ALPHA})\n",
        "  --defect-radius <i32>   Defect radius in cells (default: {DEFECT_RADIUS})\n",
        "  --defect-count <usize>  Number of circular defects (random mode) (default: {DEFECT_COUNT})\n",
        "  --defect-mode <random|lattice>  Defect geometry (default: random)\n",
        "  --defect-spacing <i32>  Lattice spacing in cells (lattice mode) (default: {DEFAULT_DEFECT_SPACING})\n",
        "  --seed <u64>            RNG seed for reproducible init/defects (default: random, printed to log/CSV)\n",
        "  --out-dir <path>        Output directory (default: runs/<mode>_<unix_ms>)\n",
        "  --dump-positions        Also write vortex positions to vortex_positions.csv\n",
        "  -h, --help              Show this help\n",
        "\n",
        "Notes:\n",
        "  - Default out-dir is runs/<mode>_<unix_ms>; pass --out-dir . to write into cwd.\n",
        "  - For dx convergence, keep L = nx*dx constant (e.g. nx=256,dx=1 vs nx=512,dx=0.5).\n",
        ),
        VORTEX_SAMPLE_PERIOD = VORTEX_SAMPLE_PERIOD,
        NX = NX,
        NY = NY,
        DEFAULT_DT = DEFAULT_DT,
        DEFAULT_DX = DEFAULT_DX,
        DEFAULT_KAPPA = DEFAULT_KAPPA,
        DEFAULT_KAPPA_SWEEP_RELAX_STEPS = DEFAULT_KAPPA_SWEEP_RELAX_STEPS,
        DEFAULT_KAPPA_SWEEP_MEASURE_STEPS = DEFAULT_KAPPA_SWEEP_MEASURE_STEPS,
        DEFAULT_ALPHA = DEFAULT_ALPHA,
        DEFECT_ALPHA = DEFECT_ALPHA,
        DEFECT_RADIUS = DEFECT_RADIUS,
        DEFECT_COUNT = DEFECT_COUNT,
        DEFAULT_DEFECT_SPACING = DEFAULT_DEFECT_SPACING,
    );
    std::process::exit(exit_code);
}

fn default_out_dir(mode: &RunMode) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let tag = match mode {
        RunMode::Interactive => "interactive",
        RunMode::Bench => "bench",
        RunMode::Headless { .. } => "headless",
        RunMode::HeadlessKappaSweep { .. } => "kappa_sweep",
    };
    PathBuf::from("runs").join(format!("{tag}_{ts}"))
}

fn parse_run_config() -> (RunMode, RunConfig) {
    let mut bench = false;
    let mut headless = false;
    let mut steps: Option<u64> = None;
    let mut sample_period: Option<u64> = None;
    let mut nx: Option<u32> = None;
    let mut ny: Option<u32> = None;
    let mut dt: Option<f32> = None;
    let mut dx: Option<f32> = None;
    let mut b_field: Option<f32> = None;
    let mut flux_n: Option<i32> = None;
    let mut kappa: Option<f32> = None;
    let mut kappa_start: Option<f32> = None;
    let mut kappa_end: Option<f32> = None;
    let mut kappa_step: Option<f32> = None;
    let mut kappa_initial_relax_steps: Option<u64> = None;
    let mut kappa_relax_steps: Option<u64> = None;
    let mut kappa_measure_steps: Option<u64> = None;
    let mut seed: Option<u64> = None;
    let mut alpha_default: Option<f32> = None;
    let mut alpha_defect: Option<f32> = None;
    let mut defect_radius: Option<i32> = None;
    let mut defect_count: Option<usize> = None;
    let mut defect_mode: Option<DefectMode> = None;
    let mut defect_spacing: Option<i32> = None;
    let mut dump_positions = false;
    let mut out_dir: Option<PathBuf> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bench" => bench = true,
            "--headless" => headless = true,
            "--steps" => {
                let v = args.next().expect("--steps 需要一个整数");
                steps = Some(v.parse::<u64>().expect("--steps 解析失败"));
            }
            "--sample-period" => {
                let v = args.next().expect("--sample-period 需要一个整数");
                sample_period = Some(v.parse::<u64>().expect("--sample-period 解析失败"));
            }
            "--nx" => {
                let v = args.next().expect("--nx 需要一个整数");
                nx = Some(v.parse::<u32>().expect("--nx 解析失败"));
            }
            "--ny" => {
                let v = args.next().expect("--ny 需要一个整数");
                ny = Some(v.parse::<u32>().expect("--ny 解析失败"));
            }
            "--dt" => {
                let v = args.next().expect("--dt 需要一个数值");
                dt = Some(v.parse::<f32>().expect("--dt 解析失败"));
            }
            "--dx" => {
                let v = args.next().expect("--dx 需要一个数值");
                dx = Some(v.parse::<f32>().expect("--dx 解析失败"));
            }
            "--b" => {
                let v = args.next().expect("--b 需要一个数值");
                b_field = Some(v.parse::<f32>().expect("--b 解析失败"));
            }
            "--flux-n" | "--n" => {
                let v = args.next().expect("--flux-n 需要一个整数");
                flux_n = Some(v.parse::<i32>().expect("--flux-n 解析失败"));
            }
            "--kappa" => {
                let v = args.next().expect("--kappa 需要一个数值");
                kappa = Some(v.parse::<f32>().expect("--kappa 解析失败"));
            }
            "--kappa-start" => {
                let v = args.next().expect("--kappa-start 需要一个数值");
                kappa_start = Some(v.parse::<f32>().expect("--kappa-start 解析失败"));
            }
            "--kappa-end" => {
                let v = args.next().expect("--kappa-end 需要一个数值");
                kappa_end = Some(v.parse::<f32>().expect("--kappa-end 解析失败"));
            }
            "--kappa-step" => {
                let v = args.next().expect("--kappa-step 需要一个数值");
                kappa_step = Some(v.parse::<f32>().expect("--kappa-step 解析失败"));
            }
            "--kappa-initial-relax-steps" => {
                let v = args
                    .next()
                    .expect("--kappa-initial-relax-steps 需要一个整数");
                kappa_initial_relax_steps = Some(
                    v.parse::<u64>()
                        .expect("--kappa-initial-relax-steps 解析失败"),
                );
            }
            "--kappa-relax-steps" => {
                let v = args.next().expect("--kappa-relax-steps 需要一个整数");
                kappa_relax_steps = Some(v.parse::<u64>().expect("--kappa-relax-steps 解析失败"));
            }
            "--kappa-measure-steps" => {
                let v = args.next().expect("--kappa-measure-steps 需要一个整数");
                kappa_measure_steps =
                    Some(v.parse::<u64>().expect("--kappa-measure-steps 解析失败"));
            }
            "--alpha-default" => {
                let v = args.next().expect("--alpha-default 需要一个数值");
                alpha_default = Some(v.parse::<f32>().expect("--alpha-default 解析失败"));
            }
            "--alpha-defect" => {
                let v = args.next().expect("--alpha-defect 需要一个数值");
                alpha_defect = Some(v.parse::<f32>().expect("--alpha-defect 解析失败"));
            }
            "--defect-radius" => {
                let v = args.next().expect("--defect-radius 需要一个整数");
                defect_radius = Some(v.parse::<i32>().expect("--defect-radius 解析失败"));
            }
            "--defect-count" => {
                let v = args.next().expect("--defect-count 需要一个整数");
                defect_count = Some(v.parse::<usize>().expect("--defect-count 解析失败"));
            }
            "--defect-mode" => {
                let v = args.next().expect("--defect-mode 需要一个值");
                defect_mode = Some(DefectMode::parse(&v).unwrap_or_else(|| {
                    eprintln!("--defect-mode 无效: {v} (可选: random|lattice)");
                    print_help_and_exit(2);
                }));
            }
            "--defect-spacing" => {
                let v = args.next().expect("--defect-spacing 需要一个整数");
                defect_spacing = Some(v.parse::<i32>().expect("--defect-spacing 解析失败"));
            }
            "--dump-positions" => dump_positions = true,
            "--out-dir" => {
                let v = args.next().expect("--out-dir 需要一个路径");
                out_dir = Some(PathBuf::from(v));
            }
            "--seed" => {
                let v = args.next().expect("--seed 需要一个整数");
                seed = Some(v.parse::<u64>().expect("--seed 解析失败"));
            }
            "-h" | "--help" => print_help_and_exit(0),
            _ => {
                eprintln!("未知参数: {arg}");
                print_help_and_exit(2);
            }
        }
    }

    let dt = dt.unwrap_or(DEFAULT_DT);
    let dx = dx.unwrap_or(DEFAULT_DX);
    let nx = nx.unwrap_or(NX);
    let ny = ny.unwrap_or(NY);
    let kappa = kappa.unwrap_or(DEFAULT_KAPPA);
    let alpha_default = alpha_default.unwrap_or(DEFAULT_ALPHA);
    let alpha_defect = alpha_defect.unwrap_or(DEFECT_ALPHA);
    let defect_radius = defect_radius.unwrap_or(DEFECT_RADIUS);
    let defect_count = defect_count.unwrap_or(DEFECT_COUNT);
    let defect_mode = defect_mode.unwrap_or(DefectMode::Random);
    let defect_spacing = defect_spacing.unwrap_or(DEFAULT_DEFECT_SPACING);
    let out_dir_opt = out_dir;

    if nx == 0 || ny == 0 {
        eprintln!("--nx/--ny 必须为正数: nx={} ny={}", nx, ny);
        print_help_and_exit(2);
    }

    let (flux_n, target_b) = match (flux_n, b_field) {
        (Some(n), b) => (n, b),
        (None, Some(b)) => (flux_n_from_b_field(b, nx, ny, dx), Some(b)),
        (None, None) => (
            flux_n_from_b_field(DEFAULT_B_FIELD, nx, ny, dx),
            Some(DEFAULT_B_FIELD),
        ),
    };

    let phi = phi_from_flux_n(flux_n, nx, ny);
    let quantized_b = b_field_from_phi(phi, dx);
    if let Some(target_b) = target_b {
        if (quantized_b - target_b).abs() > 1e-6 {
            log::warn!(
                "B 被量子化: 目标 B={:.8} -> n={} -> B={:.8} (phi={:.8})",
                target_b,
                flux_n,
                quantized_b,
                phi
            );
        }
    }

    let seed = seed.unwrap_or_else(|| rand::thread_rng().gen::<u64>());
    log::info!("RNG seed: {}", seed);
    if defect_radius < 0 {
        eprintln!("--defect-radius 不能为负数: {}", defect_radius);
        print_help_and_exit(2);
    }
    if defect_spacing <= 0 {
        eprintln!("--defect-spacing 必须为正数: {}", defect_spacing);
        print_help_and_exit(2);
    }

    let kappa_sweep = match (kappa_start, kappa_end, kappa_step) {
        (None, None, None) => None,
        (Some(start), Some(end), Some(step)) => Some((start, end, step)),
        _ => {
            eprintln!("kappa sweep 需要同时提供 --kappa-start/--kappa-end/--kappa-step");
            print_help_and_exit(2);
        }
    };

    let mode = match (bench, headless) {
        (true, true) => {
            eprintln!("--bench 与 --headless 不能同时使用");
            print_help_and_exit(2);
        }
        (true, false) => RunMode::Bench,
        (false, true) => {
            if let Some((kappa_start, kappa_end, kappa_step)) = kappa_sweep {
                if kappa_step <= 0.0 {
                    eprintln!("--kappa-step 必须为正数: {}", kappa_step);
                    print_help_and_exit(2);
                }
                if kappa_end < kappa_start {
                    eprintln!(
                        "--kappa-end 必须 >= --kappa-start: {} < {}",
                        kappa_end, kappa_start
                    );
                    print_help_and_exit(2);
                }
                let relax_steps = kappa_relax_steps.unwrap_or(DEFAULT_KAPPA_SWEEP_RELAX_STEPS);
                let initial_relax_steps = kappa_initial_relax_steps.unwrap_or(relax_steps);
                RunMode::HeadlessKappaSweep {
                    kappa_start,
                    kappa_end,
                    kappa_step,
                    initial_relax_steps,
                    relax_steps,
                    measure_steps: kappa_measure_steps.unwrap_or(DEFAULT_KAPPA_SWEEP_MEASURE_STEPS),
                    sample_period: sample_period.unwrap_or(VORTEX_SAMPLE_PERIOD),
                }
            } else {
                RunMode::Headless {
                    steps: steps.unwrap_or(5000),
                    sample_period: sample_period.unwrap_or(VORTEX_SAMPLE_PERIOD),
                }
            }
        }
        (false, false) => RunMode::Interactive,
    };

    let out_dir = out_dir_opt.unwrap_or_else(|| default_out_dir(&mode));

    (
        mode,
        RunConfig {
            nx,
            ny,
            dt,
            dx,
            flux_n,
            phi,
            kappa,
            seed,
            alpha_default,
            alpha_defect,
            defect_radius,
            defect_count,
            defect_mode,
            defect_spacing,
            dump_positions,
            out_dir,
        },
    )
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct Complex {
    re: f32,
    im: f32,
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct Params {
    nx: u32,
    ny: u32,
    show_alpha: u32,
    _pad0: u32,
    dt: f32,
    dx: f32,
    phi: f32, // plaquette flux: phi = B * dx^2
    kappa: f32,
}

// TDGL compute shader（Gauge-covariant，含磁场）
const COMPUTE_SHADER: &str = r#"
struct Params { nx: u32, ny: u32, show_alpha: u32, _pad0: u32, dt: f32, dx: f32, phi: f32, kappa: f32, }

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> psi_in: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> psi_out: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read> alpha: array<f32>;

fn idx(x: u32, y: u32) -> u32 { return y * params.nx + x; }
fn wrap(v: i32, e: u32) -> u32 { let n = i32(e); return u32((v % n + n) % n); }
fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> { return vec2(a.x*b.x - a.y*b.y, a.x*b.y + a.y*b.x); }
fn conj(a: vec2<f32>) -> vec2<f32> { return vec2(a.x, -a.y); }

// Magnetic periodic boundary condition (torus):
// Ux(x,y) = 1 except on the x-boundary hop (nx-1 -> 0),
// where Ux = exp(+i phi * nx * y), with phi the plaquette flux.
fn ux(x: u32, y: u32) -> vec2<f32> {
    if x + 1u == params.nx {
        let ang = params.phi * f32(params.nx) * f32(y);
        return vec2(cos(ang), sin(ang));
    }
    return vec2(1.0, 0.0);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= params.nx || gid.y >= params.ny { return; }
    let i = idx(gid.x, gid.y);
    let psi = psi_in[i];
    let xp = wrap(i32(gid.x)+1, params.nx); let xm = wrap(i32(gid.x)-1, params.nx);
    let yp = wrap(i32(gid.y)+1, params.ny); let ym = wrap(i32(gid.y)-1, params.ny);

    // Landau gauge: A = (0, Bx, 0)
    // Uy = exp(-i ∫ A·dl) = exp(-i B (x*dx) dx) = exp(-i phi * x), with phi = B dx^2.
    let theta = -(params.phi * f32(gid.x) + params.kappa);
    let Uy = vec2(cos(theta), sin(theta));

    // Gauge-covariant Laplacian
    let psi_xp = cmul(ux(gid.x, gid.y), psi_in[idx(xp, gid.y)]);
    let psi_xm = cmul(conj(ux(xm, gid.y)), psi_in[idx(xm, gid.y)]);
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
struct Params { nx: u32, ny: u32, show_alpha: u32, _pad0: u32, dt: f32, dx: f32, phi: f32, kappa: f32, }

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

    let (mode, config) = parse_run_config();
    match mode {
        RunMode::Bench => {
            pollster::block_on(run_benchmark(config));
        }
        RunMode::Headless {
            steps,
            sample_period,
        } => {
            pollster::block_on(run_headless(config, steps, sample_period));
        }
        RunMode::HeadlessKappaSweep {
            kappa_start,
            kappa_end,
            kappa_step,
            initial_relax_steps,
            relax_steps,
            measure_steps,
            sample_period,
        } => {
            pollster::block_on(run_headless_kappa_sweep(
                config,
                kappa_start,
                kappa_end,
                kappa_step,
                initial_relax_steps,
                relax_steps,
                measure_steps,
                sample_period,
            ));
        }
        RunMode::Interactive => {
            let event_loop = EventLoop::new().expect("创建事件循环失败");
            let mut app = App {
                state: None,
                config,
            };
            event_loop.run_app(&mut app).expect("运行失败");
        }
    }
}

/// 性能基准测试（无窗口）
async fn run_benchmark(config: RunConfig) {
    println!("=== TDGL GPU 性能基准测试 ===\n");

    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            ..Default::default()
        })
        .await
        .expect("未找到 GPU");

    println!("GPU: {}", adapter.get_info().name);
    println!();
    println!(
        "配置: dt={}, dx={}, flux_n={} (bench quantizes phi per grid size)\n",
        config.dt, config.dx, config.flux_n
    );

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .unwrap();

    let grid_sizes = [128u32, 256, 512, 1024];
    let steps = 500;

    println!("{:>10} {:>12} {:>15}", "网格", "steps/s", "cells/s");
    println!("{}", "-".repeat(40));

    for &n in &grid_sizes {
        let rate = bench_grid(&device, &queue, n, steps, &config);
        let cells_per_sec = rate * (n * n) as f64;
        println!("{:>8}² {:>12.1} {:>12.2e}", n, rate, cells_per_sec);
    }

    println!("\n基准测试完成");
}

/// Headless simulation (no window): runs TDGL and writes `vortices.csv`.
async fn run_headless(config: RunConfig, total_steps: u64, sample_period: u64) {
    if sample_period == 0 {
        eprintln!("--sample-period must be > 0");
        std::process::exit(2);
    }

    println!("=== TDGL headless run ===");
    println!(
        "config: nx={} ny={} dt={} dx={} flux_n={} (phi={:.8}, kappa={:.6}, B≈{:.8}, seed={})",
        config.nx,
        config.ny,
        config.dt,
        config.dx,
        config.flux_n,
        config.phi,
        config.kappa,
        b_field_from_phi(config.phi, config.dx),
        config.seed
    );
    println!(
        "steps: {} | sample_period: {} | out_dir: {:?}",
        total_steps, sample_period, config.out_dir
    );

    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            ..Default::default()
        })
        .await
        .expect("no compatible GPU found");

    log::info!("GPU: {}", adapter.get_info().name);

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .expect("request_device failed");

    let grid_len = grid_size(config.nx, config.ny);
    let psi0 = gen_noise(config.seed, grid_len);
    let alpha_field = gen_alpha(
        config.seed,
        config.alpha_default,
        config.alpha_defect,
        config.defect_radius,
        config.defect_count,
        config.defect_mode,
        config.defect_spacing,
        config.nx,
        config.ny,
    );
    let psi_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("PsiA"),
        contents: bytemuck::cast_slice(&psi0),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });
    let psi_b = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("PsiB"),
        size: (grid_len * 8) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Readback"),
        size: (grid_len * 8) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let alpha_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Alpha"),
        contents: bytemuck::cast_slice(&alpha_field),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let params = Params {
        nx: config.nx,
        ny: config.ny,
        show_alpha: 0,
        _pad0: 0,
        dt: config.dt,
        dx: config.dx,
        phi: config.phi,
        kappa: config.kappa,
    };
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    std::fs::create_dir_all(&config.out_dir).unwrap_or_else(|e| {
        panic!("创建输出目录失败 {:?}: {e}", config.out_dir);
    });
    write_config_toml(
        &config.out_dir,
        &config,
        "headless",
        Some(total_steps),
        Some(sample_period),
        None,
    )
    .unwrap_or_else(|e| {
        panic!("写入 config.toml 失败 {:?}: {e}", config.out_dir);
    });
    write_meta_json(&config.out_dir, &adapter, "headless").unwrap_or_else(|e| {
        panic!("写入 meta.json 失败 {:?}: {e}", config.out_dir);
    });
    let vortices_path = config.out_dir.join("vortices.csv");
    let mut vortex_csv = File::create(&vortices_path).unwrap_or_else(|e| {
        panic!("创建 vortices.csv 失败 {:?}: {e}", vortices_path);
    });
    let mut positions_csv = config.dump_positions.then(|| {
        let positions_path = config.out_dir.join("vortex_positions.csv");
        File::create(&positions_path).unwrap_or_else(|e| {
            panic!("创建 vortex_positions.csv 失败 {:?}: {e}", positions_path);
        })
    });
    let b = b_field_from_phi(config.phi, config.dx);
    writeln!(
        vortex_csv,
        "# mode=headless steps={} sample_period={}",
        total_steps, sample_period
    )
    .unwrap();
    writeln!(
        vortex_csv,
        "# nx={} ny={} dt={} dx={} flux_n={} phi={} kappa={} B={} seed={}",
        config.nx,
        config.ny,
        config.dt,
        config.dx,
        config.flux_n,
        config.phi,
        config.kappa,
        b,
        config.seed
    )
    .unwrap();
    if let Some(file) = positions_csv.as_mut() {
        writeln!(
            file,
            "# mode=headless steps={} sample_period={}",
            total_steps, sample_period
        )
        .unwrap();
        writeln!(
            file,
            "# nx={} ny={} dt={} dx={} flux_n={} phi={} kappa={} B={} seed={}",
            config.nx,
            config.ny,
            config.dt,
            config.dx,
            config.flux_n,
            config.phi,
            config.kappa,
            b,
            config.seed
        )
        .unwrap();
    }
    let defect_count_eff = defect_count_effective(
        config.defect_mode,
        config.defect_count,
        config.defect_spacing,
        config.nx,
        config.ny,
    );
    writeln!(
        vortex_csv,
        "# defects: mode={} count={} radius={} spacing={} alpha_default={} alpha_defect={}",
        config.defect_mode.as_str(),
        defect_count_eff,
        config.defect_radius,
        config.defect_spacing,
        config.alpha_default,
        config.alpha_defect
    )
    .unwrap();
    if let Some(file) = positions_csv.as_mut() {
        writeln!(
            file,
            "# defects: mode={} count={} radius={} spacing={} alpha_default={} alpha_defect={}",
            config.defect_mode.as_str(),
            defect_count_eff,
            config.defect_radius,
            config.defect_spacing,
            config.alpha_default,
            config.alpha_defect
        )
        .unwrap();
    }
    writeln!(
        vortex_csv,
        "step,time,kappa,vortices,antivortices,net,energy,energy_density,pinned_v,pinned_av,pinned_net,mean_vx,mean_vy,mean_speed"
    )
    .unwrap();
    if let Some(file) = positions_csv.as_mut() {
        writeln!(file, "step,time,kappa,x_cell,y_cell,sign").unwrap();
    }

    // Compute pipeline
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COMPUTE_SHADER)),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bgl],
                push_constant_ranges: &[],
            }),
        ),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bg_ab = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: psi_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: psi_b.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: alpha_buf.as_entire_binding(),
            },
        ],
    });
    let bg_ba = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: psi_b.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: psi_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: alpha_buf.as_entire_binding(),
            },
        ],
    });

    let (xg, yg) = (
        (config.nx + WORKGROUP_X - 1) / WORKGROUP_X,
        (config.ny + WORKGROUP_Y - 1) / WORKGROUP_Y,
    );
    let area = (config.nx as f64) * (config.ny as f64) * (config.dx as f64) * (config.dx as f64);

    let start = std::time::Instant::now();
    let mut step_count: u64 = 0;
    let mut last_sample_step: u64 = 0;
    let mut ping_is_a = true;
    let mut samples_written: u64 = 0;
    let mut prev_pos_vortices: Option<Vec<(u32, u32)>> = None;
    let mut prev_pos_step: Option<u64> = None;

    while step_count < total_steps {
        let next_sample_step = last_sample_step.saturating_add(sample_period);
        let target_step = std::cmp::min(total_steps, next_sample_step);
        let steps_to_run = target_step - step_count;

        let need_sample_after = target_step == next_sample_step || target_step == total_steps;
        if steps_to_run > 0 {
            const MAX_STEPS_PER_SUBMIT: u64 = 4096;
            let mut remaining = steps_to_run;
            while remaining > 0 {
                let batch = std::cmp::min(remaining, MAX_STEPS_PER_SUBMIT);
                let is_last_batch = batch == remaining;

                let mut enc =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                {
                    let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                    cp.set_pipeline(&pipeline);
                    for _ in 0..batch {
                        cp.set_bind_group(0, if ping_is_a { &bg_ab } else { &bg_ba }, &[]);
                        cp.dispatch_workgroups(xg, yg, 1);
                        ping_is_a = !ping_is_a;
                    }
                }

                if need_sample_after && is_last_batch {
                    let src = if ping_is_a { &psi_a } else { &psi_b };
                    enc.copy_buffer_to_buffer(src, 0, &readback_buffer, 0, (grid_len * 8) as u64);
                }
                queue.submit(Some(enc.finish()));

                remaining -= batch;
                step_count += batch;
            }
        }

        if need_sample_after {
            let slice = readback_buffer.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            device.poll(wgpu::Maintain::Wait);
            {
                let data = slice.get_mapped_range();
                let psi: &[Complex] = bytemuck::cast_slice(&data);
                let detection =
                    detect_vortices(psi, config.nx, config.ny, config.phi, config.kappa);
                let vort = detection.vortices;
                let anti = detection.antivortices;
                let energy = energy_functional(
                    psi,
                    &alpha_field,
                    config.nx,
                    config.ny,
                    config.dx,
                    config.phi,
                    config.kappa,
                );
                let t = step_count as f32 * config.dt;
                let energy_density = energy / area;
                let mut pinned_v: i32 = 0;
                let mut pinned_av: i32 = 0;
                for cell in &detection.cells {
                    if is_pinned_cell(
                        &alpha_field,
                        config.nx,
                        config.ny,
                        cell.x,
                        cell.y,
                        config.alpha_default,
                    ) {
                        if cell.sign > 0 {
                            pinned_v += 1;
                        } else {
                            pinned_av += 1;
                        }
                    }
                }
                let pinned_net = pinned_v - pinned_av;

                let curr_pos_vortices: Vec<(u32, u32)> = detection
                    .cells
                    .iter()
                    .filter(|c| c.sign > 0)
                    .map(|c| (c.x, c.y))
                    .collect();
                let (mean_vx, mean_vy, mean_speed) = if let (Some(prev), Some(prev_step)) =
                    (prev_pos_vortices.as_deref(), prev_pos_step)
                {
                    let dt_sample = (step_count.saturating_sub(prev_step) as f32) * config.dt;
                    mean_velocity_from_cells(
                        prev,
                        &curr_pos_vortices,
                        config.nx,
                        config.ny,
                        config.dx,
                        dt_sample,
                    )
                } else {
                    (0.0, 0.0, 0.0)
                };
                prev_pos_vortices = Some(curr_pos_vortices);
                prev_pos_step = Some(step_count);

                if let Some(file) = positions_csv.as_mut() {
                    for cell in &detection.cells {
                        writeln!(
                            file,
                            "{},{:.4},{:.6e},{},{},{}",
                            step_count, t, config.kappa, cell.x, cell.y, cell.sign
                        )
                        .unwrap();
                    }
                }
                writeln!(
                    vortex_csv,
                    "{},{:.4},{:.6e},{},{},{},{:.6e},{:.6e},{},{},{},{:.6e},{:.6e},{:.6e}",
                    step_count,
                    t,
                    config.kappa,
                    vort,
                    anti,
                    vort - anti,
                    energy,
                    energy_density,
                    pinned_v,
                    pinned_av,
                    pinned_net,
                    mean_vx,
                    mean_vy,
                    mean_speed
                )
                .unwrap();
                log::info!(
                    "headless sample @ step {}: +{}/-{} net={} pinned={} | v=({:.2e},{:.2e}) speed={:.2e} | target n={} | F={:.3e} dens={:.3e}",
                    step_count,
                    vort,
                    anti,
                    vort - anti,
                    pinned_net,
                    mean_vx,
                    mean_vy,
                    mean_speed,
                    config.flux_n,
                    energy,
                    energy_density
                );
            }
            readback_buffer.unmap();
            last_sample_step = step_count;
            samples_written += 1;
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    let rate = if elapsed > 0.0 {
        (total_steps as f64) / elapsed
    } else {
        0.0
    };
    println!(
        "headless done: steps={} samples={} elapsed={:.3}s steps/s={:.1}",
        total_steps, samples_written, elapsed, rate
    );
}

async fn run_headless_kappa_sweep(
    config: RunConfig,
    kappa_start: f32,
    kappa_end: f32,
    kappa_step: f32,
    initial_relax_steps: u64,
    relax_steps: u64,
    measure_steps: u64,
    sample_period: u64,
) {
    if sample_period == 0 {
        eprintln!("--sample-period must be > 0");
        std::process::exit(2);
    }
    if kappa_step <= 0.0 {
        eprintln!("--kappa-step must be > 0");
        std::process::exit(2);
    }
    if kappa_end < kappa_start {
        eprintln!("--kappa-end must be >= --kappa-start");
        std::process::exit(2);
    }
    if measure_steps == 0 {
        eprintln!("--kappa-measure-steps must be > 0");
        std::process::exit(2);
    }

    println!("=== TDGL headless kappa sweep ===");
    println!(
        "config: nx={} ny={} dt={} dx={} flux_n={} (phi={:.8}, B≈{:.8}, seed={})",
        config.nx,
        config.ny,
        config.dt,
        config.dx,
        config.flux_n,
        config.phi,
        b_field_from_phi(config.phi, config.dx),
        config.seed
    );
    println!(
        "kappa sweep: start={} end={} step={} initial_relax_steps={} relax_steps={} measure_steps={} sample_period={} out_dir={:?}",
        kappa_start,
        kappa_end,
        kappa_step,
        initial_relax_steps,
        relax_steps,
        measure_steps,
        sample_period,
        config.out_dir
    );

    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            ..Default::default()
        })
        .await
        .expect("no compatible GPU found");

    log::info!("GPU: {}", adapter.get_info().name);

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .expect("request_device failed");

    std::fs::create_dir_all(&config.out_dir).unwrap_or_else(|e| {
        panic!("创建输出目录失败 {:?}: {e}", config.out_dir);
    });
    write_config_toml(
        &config.out_dir,
        &config,
        "headless_kappa_sweep",
        None,
        None,
        Some(KappaSweepConfig {
            kappa_start,
            kappa_end,
            kappa_step,
            initial_relax_steps,
            relax_steps,
            measure_steps,
            sample_period,
        }),
    )
    .unwrap_or_else(|e| {
        panic!("写入 config.toml 失败 {:?}: {e}", config.out_dir);
    });
    write_meta_json(&config.out_dir, &adapter, "headless_kappa_sweep").unwrap_or_else(|e| {
        panic!("写入 meta.json 失败 {:?}: {e}", config.out_dir);
    });

    let vortices_path = config.out_dir.join("vortices.csv");
    let mut vortex_csv = File::create(&vortices_path).unwrap_or_else(|e| {
        panic!("创建 vortices.csv 失败 {:?}: {e}", vortices_path);
    });
    let mut positions_csv = config.dump_positions.then(|| {
        let positions_path = config.out_dir.join("vortex_positions.csv");
        File::create(&positions_path).unwrap_or_else(|e| {
            panic!("创建 vortex_positions.csv 失败 {:?}: {e}", positions_path);
        })
    });
    let sweep_path = config.out_dir.join("kappa_sweep.csv");
    let mut sweep_csv = File::create(&sweep_path).unwrap_or_else(|e| {
        panic!("创建 kappa_sweep.csv 失败 {:?}: {e}", sweep_path);
    });

    let b = b_field_from_phi(config.phi, config.dx);
    let defect_count_eff = defect_count_effective(
        config.defect_mode,
        config.defect_count,
        config.defect_spacing,
        config.nx,
        config.ny,
    );
    writeln!(
        vortex_csv,
        "# mode=headless_kappa_sweep kappa_start={} kappa_end={} kappa_step={} initial_relax_steps={} relax_steps={} measure_steps={} sample_period={}",
        kappa_start,
        kappa_end,
        kappa_step,
        initial_relax_steps,
        relax_steps,
        measure_steps,
        sample_period
    )
    .unwrap();
    writeln!(
        vortex_csv,
        "# nx={} ny={} dt={} dx={} flux_n={} phi={} B={} seed={}",
        config.nx, config.ny, config.dt, config.dx, config.flux_n, config.phi, b, config.seed
    )
    .unwrap();
    writeln!(
        vortex_csv,
        "# defects: mode={} count={} radius={} spacing={} alpha_default={} alpha_defect={}",
        config.defect_mode.as_str(),
        defect_count_eff,
        config.defect_radius,
        config.defect_spacing,
        config.alpha_default,
        config.alpha_defect
    )
    .unwrap();
    writeln!(
        vortex_csv,
        "step,time,kappa,vortices,antivortices,net,energy,energy_density,pinned_v,pinned_av,pinned_net,mean_vx,mean_vy,mean_speed"
    )
    .unwrap();

    writeln!(
        sweep_csv,
        "# mode=headless_kappa_sweep kappa_start={} kappa_end={} kappa_step={} initial_relax_steps={} relax_steps={} measure_steps={} sample_period={}",
        kappa_start,
        kappa_end,
        kappa_step,
        initial_relax_steps,
        relax_steps,
        measure_steps,
        sample_period
    )
    .unwrap();
    writeln!(
        sweep_csv,
        "# nx={} ny={} dt={} dx={} flux_n={} phi={} B={} seed={}",
        config.nx, config.ny, config.dt, config.dx, config.flux_n, config.phi, b, config.seed
    )
    .unwrap();
    writeln!(
        sweep_csv,
        "# defects: mode={} count={} radius={} spacing={} alpha_default={} alpha_defect={}",
        config.defect_mode.as_str(),
        defect_count_eff,
        config.defect_radius,
        config.defect_spacing,
        config.alpha_default,
        config.alpha_defect
    )
    .unwrap();
    writeln!(
        sweep_csv,
        "kappa,samples,mean_speed,mean_vx,mean_vy,net_mean,pinned_net_mean,energy_density_mean"
    )
    .unwrap();

    if let Some(file) = positions_csv.as_mut() {
        writeln!(
            file,
            "# mode=headless_kappa_sweep kappa_start={} kappa_end={} kappa_step={} initial_relax_steps={} relax_steps={} measure_steps={} sample_period={}",
            kappa_start,
            kappa_end,
            kappa_step,
            initial_relax_steps,
            relax_steps,
            measure_steps,
            sample_period
        )
        .unwrap();
        writeln!(
            file,
            "# nx={} ny={} dt={} dx={} flux_n={} phi={} B={} seed={}",
            config.nx, config.ny, config.dt, config.dx, config.flux_n, config.phi, b, config.seed
        )
        .unwrap();
        writeln!(
            file,
            "# defects: mode={} count={} radius={} spacing={} alpha_default={} alpha_defect={}",
            config.defect_mode.as_str(),
            defect_count_eff,
            config.defect_radius,
            config.defect_spacing,
            config.alpha_default,
            config.alpha_defect
        )
        .unwrap();
        writeln!(file, "step,time,kappa,x_cell,y_cell,sign").unwrap();
    }

    let grid_len = grid_size(config.nx, config.ny);
    let psi0 = gen_noise(config.seed, grid_len);
    let alpha_field = gen_alpha(
        config.seed,
        config.alpha_default,
        config.alpha_defect,
        config.defect_radius,
        config.defect_count,
        config.defect_mode,
        config.defect_spacing,
        config.nx,
        config.ny,
    );
    let psi_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("PsiA"),
        contents: bytemuck::cast_slice(&psi0),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });
    let psi_b = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("PsiB"),
        size: (grid_len * 8) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Readback"),
        size: (grid_len * 8) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let alpha_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Alpha"),
        contents: bytemuck::cast_slice(&alpha_field),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let mut params = Params {
        nx: config.nx,
        ny: config.ny,
        show_alpha: 0,
        _pad0: 0,
        dt: config.dt,
        dx: config.dx,
        phi: config.phi,
        kappa: kappa_start,
    };
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COMPUTE_SHADER)),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bgl],
                push_constant_ranges: &[],
            }),
        ),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bg_ab = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: psi_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: psi_b.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: alpha_buf.as_entire_binding(),
            },
        ],
    });
    let bg_ba = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: psi_b.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: psi_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: alpha_buf.as_entire_binding(),
            },
        ],
    });

    let (xg, yg) = (
        (config.nx + WORKGROUP_X - 1) / WORKGROUP_X,
        (config.ny + WORKGROUP_Y - 1) / WORKGROUP_Y,
    );
    let area = (config.nx as f64) * (config.ny as f64) * (config.dx as f64) * (config.dx as f64);

    let mut step_count: u64 = 0;
    let mut ping_is_a = true;

    let mut kappa_values: Vec<f32> = Vec::new();
    let mut k = kappa_start;
    while k <= kappa_end + 1e-6 {
        kappa_values.push(k);
        k += kappa_step;
    }

    let start = std::time::Instant::now();
    let mut total_samples: u64 = 0;

    for (i, kappa_current) in kappa_values.iter().copied().enumerate() {
        params.kappa = kappa_current;
        queue.write_buffer(&params_buf, 0, bytemuck::bytes_of(&params));

        writeln!(
            vortex_csv,
            "# sweep_kappa={} begin_step={} (index={}/{})",
            kappa_current,
            step_count,
            i + 1,
            kappa_values.len()
        )
        .unwrap();
        if let Some(file) = positions_csv.as_mut() {
            writeln!(
                file,
                "# sweep_kappa={} begin_step={} (index={}/{})",
                kappa_current,
                step_count,
                i + 1,
                kappa_values.len()
            )
            .unwrap();
        }

        // Relax (no sampling)
        {
            let relax_this = if i == 0 {
                initial_relax_steps
            } else {
                relax_steps
            };
            let mut remaining = relax_this;
            const MAX_STEPS_PER_SUBMIT: u64 = 4096;
            while remaining > 0 {
                let batch = std::cmp::min(remaining, MAX_STEPS_PER_SUBMIT);
                let mut enc =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                {
                    let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                    cp.set_pipeline(&pipeline);
                    for _ in 0..batch {
                        cp.set_bind_group(0, if ping_is_a { &bg_ab } else { &bg_ba }, &[]);
                        cp.dispatch_workgroups(xg, yg, 1);
                        ping_is_a = !ping_is_a;
                    }
                }
                queue.submit(Some(enc.finish()));
                remaining -= batch;
                step_count += batch;
            }
        }

        let mut prev_pos_vortices: Option<Vec<(u32, u32)>> = None;
        let mut prev_pos_step: Option<u64> = None;

        // Measure (sample every sample_period)
        let end_step = step_count + measure_steps;
        let mut local_samples: u64 = 0;
        let mut sum_speed: f64 = 0.0;
        let mut sum_vx: f64 = 0.0;
        let mut sum_vy: f64 = 0.0;
        let mut count_speed: u64 = 0;
        let mut sum_net: f64 = 0.0;
        let mut sum_pinned_net: f64 = 0.0;
        let mut sum_energy_density: f64 = 0.0;

        while step_count < end_step {
            let target = std::cmp::min(end_step, step_count + sample_period);
            let steps_to_run = target - step_count;

            if steps_to_run > 0 {
                const MAX_STEPS_PER_SUBMIT: u64 = 4096;
                let mut remaining = steps_to_run;
                while remaining > 0 {
                    let batch = std::cmp::min(remaining, MAX_STEPS_PER_SUBMIT);
                    let is_last_batch = batch == remaining;
                    let mut enc =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                    {
                        let mut cp =
                            enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                        cp.set_pipeline(&pipeline);
                        for _ in 0..batch {
                            cp.set_bind_group(0, if ping_is_a { &bg_ab } else { &bg_ba }, &[]);
                            cp.dispatch_workgroups(xg, yg, 1);
                            ping_is_a = !ping_is_a;
                        }
                    }
                    if is_last_batch {
                        let src = if ping_is_a { &psi_a } else { &psi_b };
                        enc.copy_buffer_to_buffer(
                            src,
                            0,
                            &readback_buffer,
                            0,
                            (grid_len * 8) as u64,
                        );
                    }
                    queue.submit(Some(enc.finish()));
                    remaining -= batch;
                    step_count += batch;
                }
            }

            let slice = readback_buffer.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            device.poll(wgpu::Maintain::Wait);
            {
                let data = slice.get_mapped_range();
                let psi: &[Complex] = bytemuck::cast_slice(&data);
                let detection =
                    detect_vortices(psi, config.nx, config.ny, config.phi, kappa_current);
                let vort = detection.vortices;
                let anti = detection.antivortices;
                let energy = energy_functional(
                    psi,
                    &alpha_field,
                    config.nx,
                    config.ny,
                    config.dx,
                    config.phi,
                    kappa_current,
                );
                let t = step_count as f32 * config.dt;
                let energy_density = energy / area;

                let mut pinned_v: i32 = 0;
                let mut pinned_av: i32 = 0;
                for cell in &detection.cells {
                    if is_pinned_cell(
                        &alpha_field,
                        config.nx,
                        config.ny,
                        cell.x,
                        cell.y,
                        config.alpha_default,
                    ) {
                        if cell.sign > 0 {
                            pinned_v += 1;
                        } else {
                            pinned_av += 1;
                        }
                    }
                }
                let pinned_net = pinned_v - pinned_av;

                let curr_pos_vortices: Vec<(u32, u32)> = detection
                    .cells
                    .iter()
                    .filter(|c| c.sign > 0)
                    .map(|c| (c.x, c.y))
                    .collect();
                let has_prev = prev_pos_vortices.is_some() && prev_pos_step.is_some();
                let (mean_vx, mean_vy, mean_speed) = if let (Some(prev), Some(prev_step)) =
                    (prev_pos_vortices.as_deref(), prev_pos_step)
                {
                    let dt_sample = (step_count.saturating_sub(prev_step) as f32) * config.dt;
                    mean_velocity_from_cells(
                        prev,
                        &curr_pos_vortices,
                        config.nx,
                        config.ny,
                        config.dx,
                        dt_sample,
                    )
                } else {
                    (0.0, 0.0, 0.0)
                };
                prev_pos_vortices = Some(curr_pos_vortices);
                prev_pos_step = Some(step_count);

                if let Some(file) = positions_csv.as_mut() {
                    for cell in &detection.cells {
                        writeln!(
                            file,
                            "{},{:.4},{:.6e},{},{},{}",
                            step_count, t, kappa_current, cell.x, cell.y, cell.sign
                        )
                        .unwrap();
                    }
                }

                writeln!(
                    vortex_csv,
                    "{},{:.4},{:.6e},{},{},{},{:.6e},{:.6e},{},{},{},{:.6e},{:.6e},{:.6e}",
                    step_count,
                    t,
                    kappa_current,
                    vort,
                    anti,
                    vort - anti,
                    energy,
                    energy_density,
                    pinned_v,
                    pinned_av,
                    pinned_net,
                    mean_vx,
                    mean_vy,
                    mean_speed
                )
                .unwrap();

                sum_net += (vort - anti) as f64;
                sum_pinned_net += pinned_net as f64;
                sum_energy_density += energy_density;
                if has_prev {
                    sum_speed += mean_speed;
                    sum_vx += mean_vx;
                    sum_vy += mean_vy;
                    count_speed += 1;
                }
            }
            readback_buffer.unmap();

            local_samples += 1;
            total_samples += 1;
        }

        let net_mean = if local_samples > 0 {
            sum_net / (local_samples as f64)
        } else {
            0.0
        };
        let pinned_net_mean = if local_samples > 0 {
            sum_pinned_net / (local_samples as f64)
        } else {
            0.0
        };
        let energy_density_mean = if local_samples > 0 {
            sum_energy_density / (local_samples as f64)
        } else {
            0.0
        };
        let mean_speed_avg = if count_speed > 0 {
            sum_speed / (count_speed as f64)
        } else {
            0.0
        };
        let mean_vx_avg = if count_speed > 0 {
            sum_vx / (count_speed as f64)
        } else {
            0.0
        };
        let mean_vy_avg = if count_speed > 0 {
            sum_vy / (count_speed as f64)
        } else {
            0.0
        };

        writeln!(
            sweep_csv,
            "{:.6e},{},{:.6e},{:.6e},{:.6e},{:.6e},{:.6e},{:.6e}",
            kappa_current,
            local_samples,
            mean_speed_avg,
            mean_vx_avg,
            mean_vy_avg,
            net_mean,
            pinned_net_mean,
            energy_density_mean
        )
        .unwrap();
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!(
        "kappa sweep done: kappa_points={} total_samples={} elapsed={:.3}s",
        kappa_values.len(),
        total_samples,
        elapsed
    );
}

fn bench_grid(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    n: u32,
    steps: u32,
    config: &RunConfig,
) -> f64 {
    let grid_size = (n * n) as usize;
    let phi = phi_from_flux_n(config.flux_n, n, n);

    // 创建 buffers
    let psi0 = vec![Complex { re: 0.0, im: 0.0 }; grid_size];
    let psi_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&psi0),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let psi_b = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (grid_size * 8) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let alpha = vec![DEFAULT_ALPHA; grid_size];
    let alpha_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&alpha),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let params = Params {
        nx: n,
        ny: n,
        show_alpha: 0,
        _pad0: 0,
        dt: config.dt,
        dx: config.dx,
        phi,
        kappa: config.kappa,
    };
    let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    // Pipeline
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COMPUTE_SHADER)),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bgl],
                push_constant_ranges: &[],
            }),
        ),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bg_ab = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: psi_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: psi_b.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: alpha_buf.as_entire_binding(),
            },
        ],
    });
    let bg_ba = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: psi_b.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: psi_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: alpha_buf.as_entire_binding(),
            },
        ],
    });

    let (xg, yg) = (
        (n + WORKGROUP_X - 1) / WORKGROUP_X,
        (n + WORKGROUP_Y - 1) / WORKGROUP_Y,
    );

    // 预热
    for i in 0..10 {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            cp.set_pipeline(&pipeline);
            cp.set_bind_group(0, if i % 2 == 0 { &bg_ab } else { &bg_ba }, &[]);
            cp.dispatch_workgroups(xg, yg, 1);
        }
        queue.submit(Some(enc.finish()));
    }
    device.poll(wgpu::Maintain::Wait);

    // 计时
    let start = std::time::Instant::now();
    for i in 0..steps {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            cp.set_pipeline(&pipeline);
            cp.set_bind_group(0, if i % 2 == 0 { &bg_ab } else { &bg_ba }, &[]);
            cp.dispatch_workgroups(xg, yg, 1);
        }
        queue.submit(Some(enc.finish()));
    }
    device.poll(wgpu::Maintain::Wait);
    let elapsed = start.elapsed().as_secs_f64();

    steps as f64 / elapsed
}

struct App {
    state: Option<State>,
    config: RunConfig,
}

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
    run_config: RunConfig,
    alpha_field: Vec<f32>,
    ping_is_a: bool,
    step_count: u64,
    // 涡旋检测
    psi_a: wgpu::Buffer,
    psi_b: wgpu::Buffer,
    readback_buffer: wgpu::Buffer,
    vortex_csv: File,
    positions_csv: Option<File>,
    prev_pos_vortices: Option<Vec<(u32, u32)>>,
    prev_pos_step: Option<u64>,
    last_vortex_sample: u64,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let b = b_field_from_phi(self.config.phi, self.config.dx);
        let title = format!(
            "TDGL + 缺陷 (按 A 切换显示)  n={}  B≈{:.6}",
            self.config.flux_n, b
        );
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(title)
                        .with_inner_size(PhysicalSize::new(512, 512)),
                )
                .expect("创建窗口失败"),
        );
        self.state = Some(pollster::block_on(State::new(window, self.config.clone())));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return,
        };
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
    async fn new(window: Arc<Window>, run_config: RunConfig) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();
        log::info!("GPU: {:?}", adapter.get_info().name);
        log::info!(
            "配置: dt={}, dx={}, flux_n={} (phi={:.8}, B≈{:.8}, seed={})",
            run_config.dt,
            run_config.dx,
            run_config.flux_n,
            run_config.phi,
            b_field_from_phi(run_config.phi, run_config.dx),
            run_config.seed
        );

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Buffers
        let grid_len = grid_size(run_config.nx, run_config.ny);
        let psi0 = gen_noise(run_config.seed, grid_len);
        let alpha_field = gen_alpha(
            run_config.seed,
            run_config.alpha_default,
            run_config.alpha_defect,
            run_config.defect_radius,
            run_config.defect_count,
            run_config.defect_mode,
            run_config.defect_spacing,
            run_config.nx,
            run_config.ny,
        );
        let psi_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PsiA"),
            contents: bytemuck::cast_slice(&psi0),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        let psi_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PsiB"),
            size: (grid_len * 8) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback"),
            size: (grid_len * 8) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        std::fs::create_dir_all(&run_config.out_dir).unwrap_or_else(|e| {
            panic!("创建输出目录失败 {:?}: {e}", run_config.out_dir);
        });
        log::info!("out_dir: {}", run_config.out_dir.display());
        write_config_toml(
            &run_config.out_dir,
            &run_config,
            "interactive",
            None,
            None,
            None,
        )
        .unwrap_or_else(|e| {
            panic!("写入 config.toml 失败 {:?}: {e}", run_config.out_dir);
        });
        write_meta_json(&run_config.out_dir, &adapter, "interactive").unwrap_or_else(|e| {
            panic!("写入 meta.json 失败 {:?}: {e}", run_config.out_dir);
        });
        let vortices_path = run_config.out_dir.join("vortices.csv");
        let mut vortex_csv = File::create(&vortices_path).unwrap_or_else(|e| {
            panic!("创建 vortices.csv 失败 {:?}: {e}", vortices_path);
        });
        let mut positions_csv = run_config.dump_positions.then(|| {
            let positions_path = run_config.out_dir.join("vortex_positions.csv");
            File::create(&positions_path).unwrap_or_else(|e| {
                panic!("创建 vortex_positions.csv 失败 {:?}: {e}", positions_path);
            })
        });
        let b = b_field_from_phi(run_config.phi, run_config.dx);
        let defect_count_eff = defect_count_effective(
            run_config.defect_mode,
            run_config.defect_count,
            run_config.defect_spacing,
            run_config.nx,
            run_config.ny,
        );
        writeln!(vortex_csv, "# mode=interactive").unwrap();
        writeln!(
            vortex_csv,
            "# nx={} ny={} dt={} dx={} flux_n={} phi={} kappa={} B={} seed={}",
            run_config.nx,
            run_config.ny,
            run_config.dt,
            run_config.dx,
            run_config.flux_n,
            run_config.phi,
            run_config.kappa,
            b,
            run_config.seed
        )
        .unwrap();
        writeln!(
            vortex_csv,
            "# defects: mode={} count={} radius={} spacing={} alpha_default={} alpha_defect={}",
            run_config.defect_mode.as_str(),
            defect_count_eff,
            run_config.defect_radius,
            run_config.defect_spacing,
            run_config.alpha_default,
            run_config.alpha_defect
        )
        .unwrap();
        writeln!(
            vortex_csv,
            "step,time,kappa,vortices,antivortices,net,energy,energy_density,pinned_v,pinned_av,pinned_net,mean_vx,mean_vy,mean_speed"
        )
        .unwrap();
        if let Some(file) = positions_csv.as_mut() {
            writeln!(file, "# mode=interactive").unwrap();
            writeln!(
                file,
                "# nx={} ny={} dt={} dx={} flux_n={} phi={} kappa={} B={} seed={}",
                run_config.nx,
                run_config.ny,
                run_config.dt,
                run_config.dx,
                run_config.flux_n,
                run_config.phi,
                run_config.kappa,
                b,
                run_config.seed
            )
            .unwrap();
            writeln!(
                file,
                "# defects: mode={} count={} radius={} spacing={} alpha_default={} alpha_defect={}",
                run_config.defect_mode.as_str(),
                defect_count_eff,
                run_config.defect_radius,
                run_config.defect_spacing,
                run_config.alpha_default,
                run_config.alpha_defect
            )
            .unwrap();
            writeln!(file, "step,time,kappa,x_cell,y_cell,sign").unwrap();
        }
        let alpha_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Alpha"),
            contents: bytemuck::cast_slice(&alpha_field),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let params = Params {
            nx: run_config.nx,
            ny: run_config.ny,
            show_alpha: 0,
            _pad0: 0,
            dt: run_config.dt,
            dx: run_config.dx,
            phi: run_config.phi,
            kappa: run_config.kappa,
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Compute pipeline
        let comp_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let comp_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COMPUTE_SHADER)),
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&comp_bgl],
                    push_constant_ranges: &[],
                }),
            ),
            module: &comp_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let comp_bg_ab = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &comp_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: psi_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: psi_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: alpha_buf.as_entire_binding(),
                },
            ],
        });
        let comp_bg_ba = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &comp_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: psi_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: psi_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: alpha_buf.as_entire_binding(),
                },
            ],
        });

        // Render pipeline
        let rend_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let rend_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(RENDER_SHADER)),
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&rend_bgl],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &rend_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &rend_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let rend_bg_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &rend_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: psi_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: alpha_buf.as_entire_binding(),
                },
            ],
        });
        let rend_bg_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &rend_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: psi_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: alpha_buf.as_entire_binding(),
                },
            ],
        });

        Self {
            window,
            device,
            queue,
            surface,
            config,
            compute_pipeline,
            render_pipeline,
            compute_bgs: [comp_bg_ab, comp_bg_ba],
            render_bgs: [rend_bg_a, rend_bg_b],
            params_buffer,
            params,
            run_config,
            alpha_field,
            ping_is_a: true,
            step_count: 0,
            psi_a,
            psi_b,
            readback_buffer,
            vortex_csv,
            positions_csv,
            prev_pos_vortices: None,
            prev_pos_step: None,
            last_vortex_sample: 0,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    fn toggle_alpha(&mut self) {
        self.params.show_alpha = if self.params.show_alpha == 0 { 1 } else { 0 };
        self.queue
            .write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&self.params));
        log::info!(
            "显示模式: {}",
            if self.params.show_alpha == 1 {
                "alpha场"
            } else {
                "|ψ|"
            }
        );
    }

    fn update(&mut self) {
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let (xg, yg) = (
            (self.params.nx + WORKGROUP_X - 1) / WORKGROUP_X,
            (self.params.ny + WORKGROUP_Y - 1) / WORKGROUP_Y,
        );
        {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            cp.set_pipeline(&self.compute_pipeline);
            for _ in 0..STEPS_PER_FRAME {
                cp.set_bind_group(
                    0,
                    &self.compute_bgs[if self.ping_is_a { 0 } else { 1 }],
                    &[],
                );
                cp.dispatch_workgroups(xg, yg, 1);
                self.ping_is_a = !self.ping_is_a;
                self.step_count += 1;
            }
        }
        // 涡旋检测采样
        let should_sample = self.step_count >= self.last_vortex_sample + VORTEX_SAMPLE_PERIOD;
        if should_sample {
            let src = if self.ping_is_a {
                &self.psi_a
            } else {
                &self.psi_b
            };
            let grid_len = (self.params.nx as u64) * (self.params.ny as u64);
            enc.copy_buffer_to_buffer(src, 0, &self.readback_buffer, 0, grid_len * 8);
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
            let detection = detect_vortices(
                psi,
                self.params.nx,
                self.params.ny,
                self.params.phi,
                self.params.kappa,
            );
            let vort = detection.vortices;
            let anti = detection.antivortices;
            let energy = energy_functional(
                psi,
                &self.alpha_field,
                self.params.nx,
                self.params.ny,
                self.params.dx,
                self.params.phi,
                self.params.kappa,
            );
            let t = self.step_count as f32 * self.params.dt;
            let area = (self.params.nx as f64)
                * (self.params.ny as f64)
                * (self.params.dx as f64)
                * (self.params.dx as f64);
            let energy_density = energy / area;
            let mut pinned_v: i32 = 0;
            let mut pinned_av: i32 = 0;
            for cell in &detection.cells {
                if is_pinned_cell(
                    &self.alpha_field,
                    self.params.nx,
                    self.params.ny,
                    cell.x,
                    cell.y,
                    self.run_config.alpha_default,
                ) {
                    if cell.sign > 0 {
                        pinned_v += 1;
                    } else {
                        pinned_av += 1;
                    }
                }
            }
            let pinned_net = pinned_v - pinned_av;

            let curr_pos_vortices: Vec<(u32, u32)> = detection
                .cells
                .iter()
                .filter(|c| c.sign > 0)
                .map(|c| (c.x, c.y))
                .collect();
            let (mean_vx, mean_vy, mean_speed) = if let (Some(prev), Some(prev_step)) =
                (self.prev_pos_vortices.as_deref(), self.prev_pos_step)
            {
                let dt_sample = (self.step_count.saturating_sub(prev_step) as f32) * self.params.dt;
                mean_velocity_from_cells(
                    prev,
                    &curr_pos_vortices,
                    self.params.nx,
                    self.params.ny,
                    self.params.dx,
                    dt_sample,
                )
            } else {
                (0.0, 0.0, 0.0)
            };
            self.prev_pos_vortices = Some(curr_pos_vortices);
            self.prev_pos_step = Some(self.step_count);

            if let Some(file) = self.positions_csv.as_mut() {
                for cell in &detection.cells {
                    writeln!(
                        file,
                        "{},{:.4},{:.6e},{},{},{}",
                        self.step_count, t, self.params.kappa, cell.x, cell.y, cell.sign
                    )
                    .unwrap();
                }
            }
            writeln!(
                self.vortex_csv,
                "{},{:.4},{:.6e},{},{},{},{:.6e},{:.6e},{},{},{},{:.6e},{:.6e},{:.6e}",
                self.step_count,
                t,
                self.params.kappa,
                vort,
                anti,
                vort - anti,
                energy,
                energy_density,
                pinned_v,
                pinned_av,
                pinned_net,
                mean_vx,
                mean_vy,
                mean_speed
            )
            .unwrap();
            log::info!(
                "涡旋检测 @ step {}: +{}/-{} net={} pinned={} | v=({:.2e},{:.2e}) speed={:.2e} | target n={} | F={:.3e} dens={:.3e}",
                self.step_count,
                vort,
                anti,
                vort - anti,
                pinned_net,
                mean_vx,
                mean_vy,
                mean_speed,
                self.run_config.flux_n,
                energy,
                energy_density
            );
        }
        self.readback_buffer.unmap();
        self.last_vortex_sample = self.step_count;
    }

    fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            rp.set_pipeline(&self.render_pipeline);
            rp.set_bind_group(0, &self.render_bgs[if self.ping_is_a { 0 } else { 1 }], &[]);
            rp.draw(0..3, 0..1);
        }
        self.queue.submit(Some(enc.finish()));
        frame.present();
        if self.step_count % 100 == 0 {
            log::info!("步数: {}", self.step_count);
        }
    }
}

fn gen_noise(seed: u64, grid_len: usize) -> Vec<Complex> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..grid_len)
        .map(|_| Complex {
            re: rng.gen_range(-0.1..0.1),
            im: rng.gen_range(-0.1..0.1),
        })
        .collect()
}

fn defect_count_effective(
    defect_mode: DefectMode,
    defect_count: usize,
    defect_spacing: i32,
    nx: u32,
    ny: u32,
) -> usize {
    match defect_mode {
        DefectMode::Random => defect_count,
        DefectMode::SquareLattice => {
            let spacing = defect_spacing.max(1) as u32;
            let nx_sites = (nx + spacing - 1) / spacing;
            let ny_sites = (ny + spacing - 1) / spacing;
            (nx_sites * ny_sites) as usize
        }
    }
}

fn gen_alpha(
    seed: u64,
    alpha_default: f32,
    alpha_defect: f32,
    defect_radius: i32,
    defect_count: usize,
    defect_mode: DefectMode,
    defect_spacing: i32,
    nx: u32,
    ny: u32,
) -> Vec<f32> {
    let mut field = vec![alpha_default; grid_size(nx, ny)];

    let apply_disk = |field: &mut [f32], cx: i32, cy: i32| {
        for dy in -defect_radius..=defect_radius {
            for dx in -defect_radius..=defect_radius {
                if dx * dx + dy * dy > defect_radius * defect_radius {
                    continue;
                }
                let x = ((cx + dx + nx as i32) % nx as i32) as usize;
                let y = ((cy + dy + ny as i32) % ny as i32) as usize;
                field[y * nx as usize + x] = alpha_defect;
            }
        }
    };

    match defect_mode {
        DefectMode::Random => {
            let mut rng = StdRng::seed_from_u64(seed ^ 0x9E37_79B9_7F4A_7C15);
            for _ in 0..defect_count {
                let (cx, cy) = (rng.gen_range(0..nx as i32), rng.gen_range(0..ny as i32));
                apply_disk(&mut field, cx, cy);
            }
        }
        DefectMode::SquareLattice => {
            let mut cx: i32 = 0;
            while cx < nx as i32 {
                let mut cy: i32 = 0;
                while cy < ny as i32 {
                    apply_disk(&mut field, cx, cy);
                    cy += defect_spacing;
                }
                cx += defect_spacing;
            }
        }
    }

    let count_effective = defect_count_effective(defect_mode, defect_count, defect_spacing, nx, ny);
    log::info!(
        "defects: mode={} count={} radius={} spacing={} alpha_default={} alpha_defect={}",
        defect_mode.as_str(),
        count_effective,
        defect_radius,
        defect_spacing,
        alpha_default,
        alpha_defect
    );
    field
}

fn cconj(a: Complex) -> Complex {
    Complex {
        re: a.re,
        im: -a.im,
    }
}

fn cmul(a: Complex, b: Complex) -> Complex {
    Complex {
        re: a.re * b.re - a.im * b.im,
        im: a.re * b.im + a.im * b.re,
    }
}

fn csub(a: Complex, b: Complex) -> Complex {
    Complex {
        re: a.re - b.re,
        im: a.im - b.im,
    }
}

fn cnorm2(a: Complex) -> f32 {
    a.re * a.re + a.im * a.im
}

fn cis(theta: f32) -> Complex {
    Complex {
        re: theta.cos(),
        im: theta.sin(),
    }
}

/// Link variable Uy in Landau gauge with an optional uniform twist `kappa`.
///
///   Uy(x) = exp(-i (phi*x + kappa))
///
/// `phi*x` encodes the uniform magnetic field; `kappa` acts like a drive (phase twist)
/// and typically produces a net drift along x (used as depinning order parameter).
fn link_uy(phi: f32, kappa: f32, x: u32) -> Complex {
    cis(-(phi * x as f32 + kappa))
}

/// Link variable Ux for magnetic-periodic boundary conditions (MPBC).
///
/// Interior: Ux = 1
/// Seam (x = nx-1): Ux = exp(+i phi * nx * y)
///
/// This "seam phase" ensures the plaquette flux is uniform and the field is globally
/// consistent on a torus when `phi*nx*ny = 2*PI*flux_n`.
fn link_ux(phi: f32, nx: u32, x: u32, y: u32) -> Complex {
    if x + 1 == nx {
        cis(phi * (nx as f32) * (y as f32))
    } else {
        Complex { re: 1.0, im: 0.0 }
    }
}

/// 相位 unwrap 到 (-π, π]
fn wrap_phase(d: f32) -> f32 {
    let tau = 2.0 * PI;
    let mut v = d;
    while v > PI {
        v -= tau;
    }
    while v <= -PI {
        v += tau;
    }
    v
}

/// 涡旋检测（gauge-invariant winding）。
///
/// 对每个网格元做离散的 ∮(∇θ - A)·dl，使用 link 变量保证规范不变性。
/// Returns VortexDetection (counts + cell positions).
#[derive(Clone, Copy, Debug)]
struct VortexCell {
    x: u32,
    y: u32,
    sign: i8,
}

#[derive(Clone, Debug)]
struct VortexDetection {
    vortices: i32,
    antivortices: i32,
    cells: Vec<VortexCell>,
}

fn mean_velocity_from_cells(
    prev: &[(u32, u32)],
    curr: &[(u32, u32)],
    nx: u32,
    ny: u32,
    dx: f32,
    dt: f32,
) -> (f64, f64, f64) {
    if prev.is_empty() || curr.is_empty() || dt <= 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let nx_i = nx as i32;
    let ny_i = ny as i32;
    let max_r = TRACK_MAX_DIST_CELLS;
    let max_r2 = max_r * max_r;

    let mut prev_map = vec![-1i32; (nx * ny) as usize];
    for (i, &(px, py)) in prev.iter().enumerate() {
        prev_map[(py * nx + px) as usize] = i as i32;
    }
    let mut used_prev = vec![false; prev.len()];

    let mut sum_vx: f64 = 0.0;
    let mut sum_vy: f64 = 0.0;
    let mut sum_speed: f64 = 0.0;
    let mut matched: u64 = 0;

    for &(cx, cy) in curr {
        let cx_i = cx as i32;
        let cy_i = cy as i32;
        let mut best: Option<(usize, i32, i32, i32)> = None; // (prev_idx, dx_off, dy_off, d2)

        for dy_off in -max_r..=max_r {
            for dx_off in -max_r..=max_r {
                let d2 = dx_off * dx_off + dy_off * dy_off;
                if d2 > max_r2 {
                    continue;
                }
                let x = (cx_i + dx_off).rem_euclid(nx_i) as u32;
                let y = (cy_i + dy_off).rem_euclid(ny_i) as u32;
                let idx = prev_map[(y * nx + x) as usize];
                if idx < 0 {
                    continue;
                }
                let i = idx as usize;
                if used_prev[i] {
                    continue;
                }
                match best {
                    None => best = Some((i, dx_off, dy_off, d2)),
                    Some((_, _, _, best_d2)) if d2 < best_d2 => {
                        best = Some((i, dx_off, dy_off, d2))
                    }
                    _ => {}
                }
            }
        }

        let Some((i, dx_off, dy_off, d2)) = best else {
            continue;
        };
        used_prev[i] = true;

        let dx_cells = (-dx_off) as f64;
        let dy_cells = (-dy_off) as f64;
        let dt_f64 = dt as f64;
        let scale = dx as f64 / dt_f64;

        sum_vx += dx_cells * scale;
        sum_vy += dy_cells * scale;
        sum_speed += (d2 as f64).sqrt() * scale;
        matched += 1;
    }

    if matched == 0 {
        return (0.0, 0.0, 0.0);
    }
    let n = matched as f64;
    (sum_vx / n, sum_vy / n, sum_speed / n)
}

fn is_pinned_cell(alpha: &[f32], nx: u32, ny: u32, x: u32, y: u32, alpha_default: f32) -> bool {
    let idx = |x: u32, y: u32| -> usize { (y * nx + x) as usize };
    let xp = (x + 1) % nx;
    let yp = (y + 1) % ny;
    let a0 = alpha[idx(x, y)];
    let a1 = alpha[idx(xp, y)];
    let a2 = alpha[idx(xp, yp)];
    let a3 = alpha[idx(x, yp)];
    let min_a = a0.min(a1).min(a2).min(a3);
    min_a < alpha_default - 1e-6
}

fn detect_vortices(psi: &[Complex], nx: u32, ny: u32, phi: f32, kappa: f32) -> VortexDetection {
    const TAU: f32 = 2.0 * PI;
    const THRESHOLD: f32 = TAU * 0.75;

    let (mut vort, mut anti) = (0i32, 0i32);
    let mut cells: Vec<VortexCell> = Vec::new();

    let idx = |x: u32, y: u32| -> usize { ((y % ny) * nx + (x % nx)) as usize };

    for y in 0..ny {
        for x in 0..nx {
            let xp = (x + 1) % nx;
            let yp = (y + 1) % ny;

            let psi00 = psi[idx(x, y)];
            let psi10 = psi[idx(xp, y)];
            let psi11 = psi[idx(xp, yp)];
            let psi01 = psi[idx(x, yp)];

            let ux0 = link_ux(phi, nx, x, y);
            let uy1 = link_uy(phi, kappa, xp);
            let ux2 = link_ux(phi, nx, x, yp);
            let uy0 = link_uy(phi, kappa, x);

            let e0 = cmul(cmul(cconj(psi00), ux0), psi10);
            let e1 = cmul(cmul(cconj(psi10), uy1), psi11);
            let e2 = cmul(cmul(cconj(psi11), cconj(ux2)), psi01);
            let e3 = cmul(cmul(cconj(psi01), cconj(uy0)), psi00);

            let sum = wrap_phase(e0.im.atan2(e0.re))
                + wrap_phase(e1.im.atan2(e1.re))
                + wrap_phase(e2.im.atan2(e2.re))
                + wrap_phase(e3.im.atan2(e3.re));

            if sum > THRESHOLD {
                vort += 1;
                cells.push(VortexCell { x, y, sign: 1 });
            } else if sum < -THRESHOLD {
                anti += 1;
                cells.push(VortexCell { x, y, sign: -1 });
            }
        }
    }

    VortexDetection {
        vortices: vort,
        antivortices: anti,
        cells,
    }
}

/// 离散能量泛函（用于耗散性/稳定性诊断）。
fn energy_functional(
    psi: &[Complex],
    alpha: &[f32],
    nx: u32,
    ny: u32,
    dx: f32,
    phi: f32,
    kappa: f32,
) -> f64 {
    let area = (dx as f64) * (dx as f64);
    let idx = |x: u32, y: u32| -> usize { (y * nx + x) as usize };

    let mut f_total: f64 = 0.0;
    for y in 0..ny {
        for x in 0..nx {
            let xp = (x + 1) % nx;
            let yp = (y + 1) % ny;

            let i = idx(x, y);
            let psi0 = psi[i];
            let psi_xp = psi[idx(xp, y)];
            let psi_yp = psi[idx(x, yp)];

            let ux = link_ux(phi, nx, x, y);
            let uy = link_uy(phi, kappa, x);

            let diff_x = csub(cmul(ux, psi_xp), psi0);
            let diff_y = csub(cmul(uy, psi_yp), psi0);
            let grad = (cnorm2(diff_x) + cnorm2(diff_y)) as f64;

            let psi2 = cnorm2(psi0) as f64;
            let pot = (-(alpha[i] as f64) * psi2) + 0.5 * psi2 * psi2;

            f_total += grad + area * pot;
        }
    }
    f_total
}
