//! SMT encoding overhead benchmark binary
//! Run with: cargo run --bin benchmark_smt --features smt

#[cfg(feature = "smt")]
fn main() {
    use routefinder::smt_benchmark;
    smt_benchmark::run_encoding_benchmarks();
}

#[cfg(not(feature = "smt"))]
fn main() {
    println!("SMT feature not enabled. Run with --features smt");
}