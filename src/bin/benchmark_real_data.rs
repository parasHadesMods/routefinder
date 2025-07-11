//! Real-world SMT benchmark binary using real_ursa_data_fixed.txt
//! Run with: cargo run --bin benchmark_real_data --features smt

#[cfg(feature = "smt")]
fn main() {
    use routefinder::smt_real_benchmark;
    smt_real_benchmark::run_real_world_benchmark();
}

#[cfg(not(feature = "smt"))]
fn main() {
    println!("SMT feature not enabled. Run with --features smt");
}