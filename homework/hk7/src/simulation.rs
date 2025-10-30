use std::error::Error;

#[derive(Clone)]
pub struct SimulationConfig {
    pub boundary_segments: Vec<BoundarySegment>,
    pub dt: f64,
    pub total_time: f64,
    pub initial_position: (f64, f64),
    pub initial_velocity: (f64, f64),
    pub damping: f64,
    pub record_interval: usize,
}

#[derive(Clone)]
pub struct BoundarySegment {
    pub control_points: Vec<(f64, f64)>,
    pub resolution: usize,
}

impl BoundarySegment {
    pub fn sample_points(&self) -> Vec<(f64, f64)> {
        match self.control_points.len() {
            2 => sample_linear(&self.control_points[0], &self.control_points[1], self.resolution),
            3 => sample_quadratic(
                &self.control_points[0],
                &self.control_points[1],
                &self.control_points[2],
                self.resolution,
            ),
            4 => sample_cubic(
                &self.control_points[0],
                &self.control_points[1],
                &self.control_points[2],
                &self.control_points[3],
                self.resolution,
            ),
            _ => panic!("BoundarySegment requires 2-4 control points"),
        }
    }
}

pub struct SimulationResult {
    pub positions: Vec<(f64, f64)>,
    pub velocities: Vec<(f64, f64)>,
    pub collisions: usize,
    pub boundary_points: Vec<(f64, f64)>,
}

pub fn run_simulation(config: &SimulationConfig) -> Result<SimulationResult, Box<dyn Error>> {
    let mut boundary: Vec<(f64, f64)> = Vec::new();
    for seg in &config.boundary_segments {
        let mut pts = seg.sample_points();
        if let Some(last) = boundary.last().copied() {
            if let Some(first) = pts.first() {
                if (first.0 - last.0).abs() < 1e-9 && (first.1 - last.1).abs() < 1e-9 {
                    pts.remove(0);
                }
            }
        }
        boundary.extend(pts);
    }

    if boundary.len() < 3 {
        return Err("边界点数不足".into());
    }

    let mut pos = config.initial_position;
    let mut vel = config.initial_velocity;
    let mut positions = Vec::new();
    let mut velocities = Vec::new();
    let mut collisions = 0usize;

    let steps = (config.total_time / config.dt).ceil() as usize;
    for step in 0..steps {
        let next_pos = (
            pos.0 + vel.0 * config.dt,
            pos.1 + vel.1 * config.dt,
        );

        if let Some((impact_pos, new_vel)) = reflect_if_needed(pos, next_pos, vel, &boundary) {
            let eps = 1e-4;
            pos = (
                impact_pos.0 + new_vel.0 * eps,
                impact_pos.1 + new_vel.1 * eps,
            );
            vel = new_vel;
            collisions += 1;
        } else {
            pos = next_pos;
        }

        vel = (
            vel.0 * (1.0 - config.damping * config.dt),
            vel.1 * (1.0 - config.damping * config.dt),
        );

        if step % config.record_interval == 0 {
            positions.push(pos);
            velocities.push(vel);
        }
    }

    Ok(SimulationResult {
        positions,
        velocities,
        collisions,
        boundary_points: boundary,
    })
}

fn reflect_if_needed(
    prev_pos: (f64, f64),
    next_pos: (f64, f64),
    vel: (f64, f64),
    boundary: &[(f64, f64)],
) -> Option<((f64, f64), (f64, f64))> {
    let mut earliest: Option<(f64, (f64, f64), (f64, f64))> = None;

    for window in boundary.windows(2) {
        let (a, b) = (window[0], window[1]);
        if let Some((impact, normal, t)) = segment_intersection(prev_pos, next_pos, a, b) {
            if earliest.as_ref().map_or(true, |(best_t, _, _)| t < *best_t) {
                earliest = Some((t, impact, normal));
            }
        }
    }

    if let Some((_, impact, mut normal)) = earliest {
        if dot(vel, normal) > 0.0 {
            normal = (-normal.0, -normal.1);
        }
        let dot_vn = dot(vel, normal);
        let reflected = (
            vel.0 - 2.0 * dot_vn * normal.0,
            vel.1 - 2.0 * dot_vn * normal.1,
        );
        let speed = (reflected.0 * reflected.0 + reflected.1 * reflected.1).sqrt();
        let limit = 5.0;
        let limited = if speed > limit {
            let factor = limit / speed;
            (reflected.0 * factor, reflected.1 * factor)
        } else {
            reflected
        };
        Some((impact, limited))
    } else {
        None
    }
}

fn segment_intersection(
    p: (f64, f64),
    q: (f64, f64),
    a: (f64, f64),
    b: (f64, f64),
) -> Option<((f64, f64), (f64, f64), f64)> {
    let r = (q.0 - p.0, q.1 - p.1);
    let s = (b.0 - a.0, b.1 - a.1);
    let denom = cross(r, s);
    if denom.abs() < 1e-9 {
        return None;
    }

    let diff = (a.0 - p.0, a.1 - p.1);
    let t = cross(diff, s) / denom;
    let u = cross(diff, r) / denom;

    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        let impact = (p.0 + t * r.0, p.1 + t * r.1);
        let tangent = normalize(s);
        let mut normal = normalize((-tangent.1, tangent.0));
        if dot((r.0, r.1), normal) > 0.0 {
            normal = (-normal.0, -normal.1);
        }
        Some((impact, normal, t))
    } else {
        None
    }
}

fn dot(a: (f64, f64), b: (f64, f64)) -> f64 {
    a.0 * b.0 + a.1 * b.1
}

fn cross(a: (f64, f64), b: (f64, f64)) -> f64 {
    a.0 * b.1 - a.1 * b.0
}

fn normalize(v: (f64, f64)) -> (f64, f64) {
    let len = (v.0 * v.0 + v.1 * v.1).sqrt();
    if len < 1e-9 { (0.0, 0.0) } else { (v.0 / len, v.1 / len) }
}

fn sample_linear(p0: &(f64, f64), p1: &(f64, f64), n: usize) -> Vec<(f64, f64)> {
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            (
                p0.0 + t * (p1.0 - p0.0),
                p0.1 + t * (p1.1 - p0.1),
            )
        })
        .collect()
}

fn sample_quadratic(p0: &(f64, f64), p1: &(f64, f64), p2: &(f64, f64), n: usize) -> Vec<(f64, f64)> {
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let u = 1.0 - t;
            (
                u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0,
                u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1,
            )
        })
        .collect()
}

fn sample_cubic(
    p0: &(f64, f64),
    p1: &(f64, f64),
    p2: &(f64, f64),
    p3: &(f64, f64),
    n: usize,
) -> Vec<(f64, f64)> {
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let u = 1.0 - t;
            (
                u * u * u * p0.0
                    + 3.0 * u * u * t * p1.0
                    + 3.0 * u * t * t * p2.0
                    + t * t * t * p3.0,
                u * u * u * p0.1
                    + 3.0 * u * u * t * p1.1
                    + 3.0 * u * t * t * p2.1
                    + t * t * t * p3.1,
            )
        })
        .collect()
}

pub fn example_boundary() -> Vec<BoundarySegment> {
    vec![
        BoundarySegment {
            control_points: vec![(-2.0, -1.0), (-1.5, 2.5), (0.0, 3.0)],
            resolution: 80,
        },
        BoundarySegment {
            control_points: vec![(0.0, 3.0), (1.5, 2.5), (2.0, -1.0)],
            resolution: 80,
        },
        BoundarySegment {
            control_points: vec![(2.0, -1.0), (0.5, -2.5), (0.0, -3.0)],
            resolution: 80,
        },
        BoundarySegment {
            control_points: vec![(0.0, -3.0), (-0.5, -2.5), (-2.0, -1.0)],
            resolution: 80,
        },
    ]
}
