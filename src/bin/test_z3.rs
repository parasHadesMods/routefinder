//! Simple Z3 integration test binary
//! Run with: cargo run --bin test_z3 --features smt

#[cfg(feature = "smt")]
fn main() {
    use routefinder::smt_test;
    smt_test::run_all_tests();
}

#[cfg(not(feature = "smt"))]
fn main() {
    println!("SMT feature not enabled. Run with --features smt");
}