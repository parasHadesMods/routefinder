//! SMT encoding overhead benchmarks for Phase 1.4
//! 
//! Measures the time taken to encode various arithmetic circuits
//! to validate that encoding overhead stays under 1 second target.

#[cfg(feature = "smt")]
use z3::*;
#[cfg(feature = "smt")]
use z3::ast::Ast;
use std::time::{Duration, Instant};

#[cfg(feature = "smt")]
pub struct EncodingBenchmark {
    pub constraint_count: usize,
    pub encoding_time: Duration,
    pub solving_time: Duration,
    pub total_time: Duration,
}

#[cfg(feature = "smt")]
impl EncodingBenchmark {
    pub fn new() -> Self {
        Self {
            constraint_count: 0,
            encoding_time: Duration::ZERO,
            solving_time: Duration::ZERO,
            total_time: Duration::ZERO,
        }
    }
}

/// Benchmark LCG state advancement encoding
#[cfg(feature = "smt")]
pub fn benchmark_lcg_advancement(steps: u32) -> EncodingBenchmark {
    let start_total = Instant::now();
    let start_encoding = Instant::now();
    
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    
    // LCG constants
    const LCG_MULTIPLIER: u64 = 0x5851f42d4c957f2d;
    const LCG_INCREMENT: u64 = 0xb47c73972972b7b7;
    
    let multiplier = ast::BV::from_u64(&ctx, LCG_MULTIPLIER, 64);
    let increment = ast::BV::from_u64(&ctx, LCG_INCREMENT, 64);
    
    // Initial state variable
    let mut current_state = ast::BV::new_const(&ctx, "initial_state", 64);
    
    // Encode multiple LCG advancement steps
    for i in 0..steps {
        let next_state = ast::BV::new_const(&ctx, format!("state_{}", i + 1), 64);
        
        // next_state = (current_state * multiplier + increment) mod 2^64
        let temp = current_state.bvmul(&multiplier);
        let advanced = temp.bvadd(&increment);
        
        solver.assert(&next_state._eq(&advanced));
        current_state = next_state;
    }
    
    // Add a constraint to make the problem interesting
    let final_constraint = current_state.bvugt(&ast::BV::from_u64(&ctx, 1000000, 64));
    solver.assert(&final_constraint);
    
    let encoding_time = start_encoding.elapsed();
    
    // Measure solving time
    let start_solving = Instant::now();
    let result = solver.check();
    let solving_time = start_solving.elapsed();
    
    let total_time = start_total.elapsed();
    
    println!("LCG advancement benchmark ({}steps): encoding={:?}, solving={:?}, result={:?}",
             steps, encoding_time, solving_time, result);
    
    EncodingBenchmark {
        constraint_count: steps as usize + 1, // +1 for final constraint
        encoding_time,
        solving_time,
        total_time,
    }
}

/// Benchmark PCG output function encoding
#[cfg(feature = "smt")]
pub fn benchmark_pcg_output(count: u32) -> EncodingBenchmark {
    let start_total = Instant::now();
    let start_encoding = Instant::now();
    
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    
    let mut constraint_count = 0;
    
    // Encode multiple PCG output computations
    for i in 0..count {
        let state = ast::BV::new_const(&ctx, format!("state_{}", i), 64);
        let output = ast::BV::new_const(&ctx, format!("output_{}", i), 32);
        
        // PCG XSH-RR implementation
        // 1. Extract rotation amount (top 5 bits)
        let rot = state.extract(63, 59);
        
        // 2. Compute XSH (xor-shift) value
        let shifted = state.bvlshr(&ast::BV::from_u64(&ctx, 18, 64));
        let xored = state.bvxor(&shifted);
        let xsh = xored.extract(31, 0);
        
        // 3. Apply right rotation: rotr(xsh, rot)
        // rotr(x, n) = (x >> n) | (x << (32 - n))
        let thirty_two = ast::BV::from_u64(&ctx, 32, 32);
        let rot_32 = rot.zero_ext(32 - 5); // extend 5-bit to 32-bit
        let right_shift = xsh.bvlshr(&rot_32);
        let left_shift_amount = thirty_two.bvsub(&rot_32);
        let left_shift = xsh.bvshl(&left_shift_amount);
        let rotated = right_shift.bvor(&left_shift);
        
        solver.assert(&output._eq(&rotated));
        constraint_count += 1;
        
        // Add some range constraints to make it interesting
        if i < 3 {
            let state_min = ast::BV::from_u64(&ctx, 1000000, 64);
            let state_max = ast::BV::from_u64(&ctx, 18446744073709551615u64 - 1000000, 64);
            solver.assert(&state.bvuge(&state_min));
            solver.assert(&state.bvule(&state_max));
            constraint_count += 2;
        }
    }
    
    let encoding_time = start_encoding.elapsed();
    
    // Measure solving time
    let start_solving = Instant::now();
    let result = solver.check();
    let solving_time = start_solving.elapsed();
    
    let total_time = start_total.elapsed();
    
    println!("PCG output benchmark ({}ops): encoding={:?}, solving={:?}, result={:?}",
             count, encoding_time, solving_time, result);
    
    EncodingBenchmark {
        constraint_count,
        encoding_time,
        solving_time,
        total_time,
    }
}

/// Benchmark bit-vector range constraints
#[cfg(feature = "smt")]
pub fn benchmark_bitvector_ranges(constraint_count: u32) -> EncodingBenchmark {
    let start_total = Instant::now();
    let start_encoding = Instant::now();
    
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    
    // Encode multiple bit-vector range constraints
    for i in 0..constraint_count {
        let bv_value = ast::BV::new_const(&ctx, format!("bv_{}", i), 32);
        
        // Add range constraint based on index
        // Split [0, 2^32] into constraint_count segments
        let segment_size = (u32::MAX as u64) / (constraint_count as u64);
        let min_val = (i as u64) * segment_size;
        let max_val = ((i + 1) as u64) * segment_size;
        
        let min_bv = ast::BV::from_u64(&ctx, min_val, 32);
        let max_bv = ast::BV::from_u64(&ctx, max_val.min(u32::MAX as u64), 32);
        
        solver.assert(&bv_value.bvuge(&min_bv));
        solver.assert(&bv_value.bvule(&max_bv));
    }
    
    let encoding_time = start_encoding.elapsed();
    
    // Measure solving time
    let start_solving = Instant::now();
    let result = solver.check();
    let solving_time = start_solving.elapsed();
    
    let total_time = start_total.elapsed();
    
    println!("Bit-vector ranges benchmark ({}constraints): encoding={:?}, solving={:?}, result={:?}",
             constraint_count, encoding_time, solving_time, result);
    
    EncodingBenchmark {
        constraint_count: (constraint_count * 2) as usize, // 2 constraints per iteration
        encoding_time,
        solving_time,
        total_time,
    }
}

/// Run comprehensive encoding benchmarks
#[cfg(feature = "smt")]
pub fn run_encoding_benchmarks() {
    println!("=== SMT Encoding Overhead Benchmarks ===");
    
    let mut total_encoding_time = Duration::ZERO;
    let mut total_constraints = 0;
    
    // LCG advancement benchmarks
    for steps in [5, 10, 20] {
        println!("\nRunning: LCG advancement ({} steps)", steps);
        let result = benchmark_lcg_advancement(steps);
        
        total_encoding_time += result.encoding_time;
        total_constraints += result.constraint_count;
        
        println!("  Constraints: {}", result.constraint_count);
        println!("  Encoding: {:?}", result.encoding_time);
        println!("  Solving: {:?}", result.solving_time);
        println!("  Total: {:?}", result.total_time);
        
        // Check if encoding time is reasonable (target: <1s)
        if result.encoding_time > Duration::from_millis(1000) {
            println!("  ⚠️  SLOW ENCODING: {:?} > 1s target", result.encoding_time);
        } else if result.encoding_time > Duration::from_millis(100) {
            println!("  ⚠️  Moderate encoding time: {:?}", result.encoding_time);
        } else {
            println!("  ✅ Fast encoding: {:?}", result.encoding_time);
        }
    }
    
    // PCG output benchmarks
    for ops in [3, 5, 10] {
        println!("\nRunning: PCG output ({} ops)", ops);
        let result = benchmark_pcg_output(ops);
        
        total_encoding_time += result.encoding_time;
        total_constraints += result.constraint_count;
        
        println!("  Constraints: {}", result.constraint_count);
        println!("  Encoding: {:?}", result.encoding_time);
        println!("  Solving: {:?}", result.solving_time);
        println!("  Total: {:?}", result.total_time);
        
        if result.encoding_time > Duration::from_millis(1000) {
            println!("  ⚠️  SLOW ENCODING: {:?} > 1s target", result.encoding_time);
        } else if result.encoding_time > Duration::from_millis(100) {
            println!("  ⚠️  Moderate encoding time: {:?}", result.encoding_time);
        } else {
            println!("  ✅ Fast encoding: {:?}", result.encoding_time);
        }
    }
    
    // Bit-vector range benchmarks
    for constraints in [5, 10, 20] {
        println!("\nRunning: Bit-vector ranges ({} constraints)", constraints);
        let result = benchmark_bitvector_ranges(constraints);
        
        total_encoding_time += result.encoding_time;
        total_constraints += result.constraint_count;
        
        println!("  Constraints: {}", result.constraint_count);
        println!("  Encoding: {:?}", result.encoding_time);
        println!("  Solving: {:?}", result.solving_time);
        println!("  Total: {:?}", result.total_time);
        
        if result.encoding_time > Duration::from_millis(1000) {
            println!("  ⚠️  SLOW ENCODING: {:?} > 1s target", result.encoding_time);
        } else if result.encoding_time > Duration::from_millis(100) {
            println!("  ⚠️  Moderate encoding time: {:?}", result.encoding_time);
        } else {
            println!("  ✅ Fast encoding: {:?}", result.encoding_time);
        }
    }
    
    println!("\n=== Summary ===");
    println!("Total constraints encoded: {}", total_constraints);
    println!("Total encoding time: {:?}", total_encoding_time);
    if total_constraints > 0 {
        println!("Average time per constraint: {:?}", total_encoding_time / total_constraints as u32);
    }
    
    if total_encoding_time > Duration::from_secs(1) {
        println!("⚠️  Total encoding time exceeds 1s target");
    } else {
        println!("✅ Total encoding time within 1s target");
    }
}

#[cfg(not(feature = "smt"))]
pub fn run_encoding_benchmarks() {
    println!("SMT feature not enabled. Run with --features smt");
}