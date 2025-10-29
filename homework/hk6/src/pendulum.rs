pub const G: f64 = 9.8;
pub const L: f64 = 9.8;
pub const Q: f64 = 0.5;
pub const DEFAULT_F_D: f64 = 1.2;
pub const OMEGA_D: f64 = 2.0 / 3.0;

#[derive(Clone, Copy, Debug)]
pub struct PendulumState {
    pub theta: f64,
    pub omega: f64,
    pub t: f64,
    pub dt: f64,
    pub f_drive: f64,
}

impl PendulumState {
    pub fn new(theta: f64, omega: f64, t: f64, dt: f64, f_drive: f64) -> Self {
        Self {
            theta,
            omega,
            t,
            dt,
            f_drive,
        }
    }

    pub fn acceleration(theta: f64, omega: f64, t: f64, f_drive: f64) -> f64 {
        - (G / L) * theta.sin() - Q * omega + f_drive * (OMEGA_D * t).sin()
    }

    pub fn next(&self) -> Self {
        let dt = self.dt;

        let k1_theta = self.omega;
        let k1_omega = Self::acceleration(self.theta, self.omega, self.t, self.f_drive);

        let theta_mid = self.theta + 0.5 * dt * k1_theta;
        let omega_mid = self.omega + 0.5 * dt * k1_omega;
        let t_mid = self.t + 0.5 * dt;

        let k2_theta = omega_mid;
        let k2_omega = Self::acceleration(theta_mid, omega_mid, t_mid, self.f_drive);

        let theta_mid = self.theta + 0.5 * dt * k2_theta;
        let omega_mid = self.omega + 0.5 * dt * k2_omega;

        let k3_theta = omega_mid;
        let k3_omega = Self::acceleration(theta_mid, omega_mid, t_mid, self.f_drive);

        let theta_end = self.theta + dt * k3_theta;
        let omega_end = self.omega + dt * k3_omega;
        let t_end = self.t + dt;

        let k4_theta = omega_end;
        let k4_omega = Self::acceleration(theta_end, omega_end, t_end, self.f_drive);

        let theta = self.theta + (dt / 6.0) * (k1_theta + 2.0 * k2_theta + 2.0 * k3_theta + k4_theta);
        let omega = self.omega + (dt / 6.0) * (k1_omega + 2.0 * k2_omega + 2.0 * k3_omega + k4_omega);
        let t = self.t + dt;

        Self::new(theta, omega, t, dt, self.f_drive)
    }
}
