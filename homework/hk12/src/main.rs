//! 伊辛模型蒙特卡洛模拟项目
//!
//! 本项目包含两个独立的可执行程序：
//!
//! ## 程序一：临界指数计算 (ising_critical)
//! 计算二维伊辛模型的磁化强度M，并估算临界指数β≈1/8。
//! ```bash
//! cargo run --release --bin ising_critical
//! cargo run --release --bin ising_critical -- --lattice triangular
//! ```
//!
//! ## 程序二：高温外场响应 (ising_field)  
//! 验证高温下M(H) ≈ tanh(H/T)的理论预期。
//! ```bash
//! cargo run --release --bin ising_field
//! cargo run --release --bin ising_field -- --temperatures 100,50,20
//! ```

fn main() {
    println!("伊辛模型蒙特卡洛模拟项目");
    println!("========================");
    println!();
    println!("本项目包含两个独立程序：");
    println!();
    println!("1. 临界指数计算:");
    println!("   cargo run --release --bin ising_critical");
    println!("   cargo run --release --bin ising_critical -- --help");
    println!();
    println!("2. 高温外场响应:");
    println!("   cargo run --release --bin ising_field");
    println!("   cargo run --release --bin ising_field -- --help");
}
