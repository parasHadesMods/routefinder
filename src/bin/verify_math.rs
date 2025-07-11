//! Mathematical verification binary for Phase 1.5-1.8
//! Run with: cargo run --bin verify_math

fn main() {
    use routefinder::smt_verification;
    smt_verification::run_all_verifications();
}