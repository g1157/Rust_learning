use ndarray::Array1;

fn schrodinger_rhs(x: f64, psi: f64, phi: f64, e: f64) -> f64 {
    // 势函数 V(x) = 0.5 * x^2 （简谐振子）
    let v = 0.5 * x * x;
    let hbar = 1.0;
    let m = 1.0;
    (2.0 * m / (hbar * hbar)) * (v - e) * psi
}

// 单步 Runge-Kutta4
fn rk4_step(x: f64, dx: f64, psi: f64, phi: f64, e: f64) -> (f64, f64) {
    let k1_psi = dx * phi;
    let k1_phi = dx * schrodinger_rhs(x, psi, phi, e);

    let k2_psi = dx * (phi + 0.5 * k1_phi);
    let k2_phi = dx * schrodinger_rhs(x + 0.5 * dx, psi + 0.5 * k1_psi, phi + 0.5 * k1_phi, e);

    let k3_psi = dx * (phi + 0.5 * k2_phi);
    let k3_phi = dx * schrodinger_rhs(x + 0.5 * dx, psi + 0.5 * k2_psi, phi + 0.5 * k2_phi, e);

    let k4_psi = dx * (phi + k3_phi);
    let k4_phi = dx * schrodinger_rhs(x + dx, psi + k3_psi, phi + k3_phi, e);

    let psi_next = psi + (k1_psi + 2.0 * k2_psi + 2.0 * k3_psi + k4_psi) / 6.0;
    let phi_next = phi + (k1_phi + 2.0 * k2_phi + 2.0 * k3_phi + k4_phi) / 6.0;
    (psi_next, phi_next)
}

// 对给定能量 E，积分出 ψ(L)
fn shoot(e: f64, x_min: f64, x_max: f64, dx: f64) -> f64 {
    let mut psi = 0.0;
    let mut phi = 1e-5; // 初始斜率 ≠ 0，保证非平凡解
    let mut x = x_min;

    while x < x_max {
        let (psi_next, phi_next) = rk4_step(x, dx, psi, phi, e);
        psi = psi_next;
        phi = phi_next;
        x += dx;
    }
    psi
}

// 二分搜索能量
fn find_energy(x_min: f64, x_max: f64, dx: f64, e1: f64, e2: f64, tol: f64) -> f64 {
    let mut a = e1;
    let mut b = e2;
    let mut fa = shoot(a, x_min, x_max, dx);
    let mut fb = shoot(b, x_min, x_max, dx);

    while (b - a).abs() > tol {
        let c = 0.5 * (a + b);
        let fc = shoot(c, x_min, x_max, dx);
        if fa * fc < 0.0 {
            b = c;
            fb = fc;
        } else {
            a = c;
            fa = fc;
        }
    }
    0.5 * (a + b)
}

fn main() {
    let x_min = -5.0;
    let x_max = 5.0;
    let dx = 0.01;

    let e = find_energy(x_min, x_max, dx, 0.4, 0.6, 1e-5);
    println!("Ground state energy ≈ {:.5}", e);
}
