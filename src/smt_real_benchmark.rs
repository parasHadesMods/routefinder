//! Real-world SMT benchmark using data from real_ursa_data_fixed.txt
//! 
//! This benchmark replicates the exact constraints from real data to measure
//! SMT performance against the known brute-force baseline.

#[cfg(feature = "smt")]
use z3::*;
#[cfg(feature = "smt")]
use z3::ast::Ast;
use std::time::{Duration, Instant};

/// Data point from real_ursa_data_fixed.txt
#[derive(Debug, Clone)]
pub struct RealDataPoint {
    pub name: String,
    pub offset: u32,
    pub min: f64,
    pub max: f64,
    pub observed: f64,
}

/// Parse real_ursa_data_fixed.txt format
pub fn parse_real_data() -> Vec<RealDataPoint> {
    vec![
        RealDataPoint { name: "nassault".to_string(), offset: 14, min: 1.3, max: 1.5, observed: 1.35 },
        RealDataPoint { name: "nambush".to_string(), offset: 18, min: 1.6, max: 1.8, observed: 1.78 },
        RealDataPoint { name: "nfavor".to_string(), offset: 20, min: 0.11, max: 0.2, observed: 0.12 },
        RealDataPoint { name: "nlunge".to_string(), offset: 22, min: 1.4, max: 1.6, observed: 1.42 },
        RealDataPoint { name: "nsoul".to_string(), offset: 24, min: 0.3, max: 0.4, observed: 0.39 },
        RealDataPoint { name: "nstrike".to_string(), offset: 26, min: 1.3, max: 1.4, observed: 1.4 },
        RealDataPoint { name: "neclipse".to_string(), offset: 28, min: 1.5, max: 1.8, observed: 1.72 },
    ]
}

#[cfg(feature = "smt")]
pub struct RealWorldBenchmark {
    pub encoding_time: Duration,
    pub solving_time: Duration,
    pub total_time: Duration,
    pub constraint_count: usize,
    pub found_seed: Option<u64>,
    pub expected_seed: u64,
}

/// Benchmark SMT solver on real Ursa data
#[cfg(feature = "smt")]
pub fn benchmark_real_ursa_data() -> RealWorldBenchmark {
    let start_total = Instant::now();
    let start_encoding = Instant::now();
    
    println!("=== Real Ursa Data SMT Benchmark ===");
    println!("Expected seed: 1152303697");
    
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    
    // LCG constants from our verified implementation
    const LCG_MULTIPLIER: u64 = 0x5851f42d4c957f2d;
    const LCG_INCREMENT: u64 = 0xb47c73972972b7b7;
    
    let multiplier = ast::BV::from_u64(&ctx, LCG_MULTIPLIER, 64);
    let increment = ast::BV::from_u64(&ctx, LCG_INCREMENT, 64);
    
    // Initial RNG state variable (what we're solving for)
    let mut current_state = ast::BV::new_const(&ctx, "initial_state", 64);
    
    let data_points = parse_real_data();
    let mut constraint_count = 0;
    let mut last_offset = 0;
    
    println!("Encoding {} constraints:", data_points.len());
    
    for (i, point) in data_points.iter().enumerate() {
        println!("  {}: {} at offset {} in range [{}, {}]", i+1, point.name, point.offset, point.min, point.max);
        
        // Advance RNG state to the required offset
        let steps = point.offset - last_offset;
        for step in 0..steps {
            let next_state = ast::BV::new_const(&ctx, format!("state_{}_{}", point.offset, step), 64);
            let temp = current_state.bvmul(&multiplier);
            let advanced = temp.bvadd(&increment);
            solver.assert(&next_state._eq(&advanced));
            current_state = next_state;
            constraint_count += 1;
        }
        last_offset = point.offset;
        
        // Apply PCG output function to current state
        let pcg_state = current_state.clone();
        
        // PCG XSH-RR implementation
        // 1. Extract rotation amount (top 5 bits)
        let rot = pcg_state.extract(63, 59);
        
        // 2. Compute XSH (xor-shift) value  
        let shifted = pcg_state.bvlshr(&ast::BV::from_u64(&ctx, 18, 64));
        let xored = pcg_state.bvxor(&shifted);
        let xsh = xored.extract(31, 0);
        
        // 3. Apply right rotation: rotr(xsh, rot)
        let thirty_two = ast::BV::from_u64(&ctx, 32, 32);
        let rot_32 = rot.zero_ext(32 - 5); // extend 5-bit to 32-bit
        let right_shift = xsh.bvlshr(&rot_32);
        let left_shift_amount = thirty_two.bvsub(&rot_32);
        let left_shift = xsh.bvshl(&left_shift_amount);
        let pcg_output = right_shift.bvor(&left_shift);
        
        // Replicate the exact logic from the brute-force approach in data_point.rs
        // The brute-force uses observed values with rounding tolerance
        
        let range_size = point.max - point.min;
        
        // Values that would round to our observed value (±0.005 tolerance)
        let actual_min = point.observed - 0.005;
        let actual_max = point.observed + 0.005;
        
        // Convert these actual values back to fractions of the [0,1] range
        let fraction_min = ((actual_min - point.min) / range_size).max(0.0).min(1.0);
        let fraction_max = ((actual_max - point.min) / range_size).max(0.0).min(1.0);
        
        // Find the u32 values that would map to these fractions
        // u32 value V maps to fraction V / u32::MAX
        // So we want: fraction_min <= V / u32::MAX <= fraction_max
        // Which means: fraction_min * u32::MAX <= V <= fraction_max * u32::MAX
        
        let min_u32_exact = fraction_min * u32::MAX as f64;
        let max_u32_exact = fraction_max * u32::MAX as f64;
        
        // Convert to actual u32 bounds (matching the brute-force logic exactly)
        let min_u32 = min_u32_exact.ceil() as u32;
        let max_u32 = max_u32_exact.floor() as u32;
        
        // Handle edge case where the range is too narrow
        let (final_min, final_max) = if min_u32 > max_u32 {
            let mid_u32 = ((min_u32_exact + max_u32_exact) * 0.5) as u32;
            (mid_u32, mid_u32)
        } else {
            (min_u32, max_u32)
        };
        
        let min_bv = ast::BV::from_u64(&ctx, final_min as u64, 32);
        let max_bv = ast::BV::from_u64(&ctx, final_max as u64, 32);
        
        println!("    -> PCG output range: [0x{:08x}, 0x{:08x}] ({} values)", 
                final_min, final_max, final_max.saturating_sub(final_min) + 1);
        
        solver.assert(&pcg_output.bvuge(&min_bv));
        solver.assert(&pcg_output.bvule(&max_bv));
        constraint_count += 2;
        
        // Advance state one more time for next iteration
        let next_state = ast::BV::new_const(&ctx, format!("state_after_{}", point.offset), 64);
        let temp = current_state.bvmul(&multiplier);
        let advanced = temp.bvadd(&increment);
        solver.assert(&next_state._eq(&advanced));
        current_state = next_state;
        constraint_count += 1;
    }
    
    let encoding_time = start_encoding.elapsed();
    println!("Encoding completed: {:?} ({} constraints)", encoding_time, constraint_count);
    
    // Solve the constraints
    println!("Solving constraints...");
    let start_solving = Instant::now();
    let result = solver.check();
    let solving_time = start_solving.elapsed();
    
    let total_time = start_total.elapsed();
    
    let found_seed = match result {
        SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let initial_state_var = ast::BV::new_const(&ctx, "initial_state", 64);
            if let Some(seed_ast) = model.eval(&initial_state_var, true) {
                // Extract the u64 value from the AST
                if let Some(seed_str) = seed_ast.as_u64() {
                    Some(seed_str)
                } else {
                    println!("Could not extract seed value from model");
                    None
                }
            } else {
                println!("Could not evaluate initial_state in model");
                None
            }
        }
        SatResult::Unsat => {
            println!("UNSAT: No solution found!");
            None
        }
        SatResult::Unknown => {
            println!("UNKNOWN: Solver could not determine satisfiability");
            None
        }
    };
    
    RealWorldBenchmark {
        encoding_time,
        solving_time,
        total_time,
        constraint_count,
        found_seed,
        expected_seed: 1152303697,
    }
}

/// Run the real-world benchmark and display results
#[cfg(feature = "smt")]
pub fn run_real_world_benchmark() {
    let benchmark = benchmark_real_ursa_data();
    
    println!("\n=== Real Ursa Data Benchmark Results ===");
    println!("Constraints encoded: {}", benchmark.constraint_count);
    println!("Encoding time: {:?}", benchmark.encoding_time);
    println!("Solving time: {:?}", benchmark.solving_time);
    println!("Total time: {:?}", benchmark.total_time);
    
    match benchmark.found_seed {
        Some(seed) => {
            println!("Found seed: {}", seed);
            if seed == benchmark.expected_seed {
                println!("✅ SUCCESS: Found the expected seed!");
            } else {
                println!("❌ INCORRECT: Expected {}, found {}", benchmark.expected_seed, seed);
            }
        }
        None => {
            println!("❌ FAILED: No seed found");
        }
    }
    
    // Performance analysis
    println!("\n=== Performance Analysis ===");
    if benchmark.encoding_time > Duration::from_secs(1) {
        println!("⚠️  Encoding time > 1s target: {:?}", benchmark.encoding_time);
    } else {
        println!("✅ Encoding time within 1s target: {:?}", benchmark.encoding_time);
    }
    
    if benchmark.total_time > Duration::from_secs(10) {
        println!("⚠️  Total time > 10s target: {:?}", benchmark.total_time);
    } else {
        println!("✅ Total time within 10s target: {:?}", benchmark.total_time);
    }
    
    // Compare to brute force baseline (71s)
    let brute_force_baseline = Duration::from_secs(71);
    if benchmark.total_time < brute_force_baseline {
        let speedup = brute_force_baseline.as_secs_f64() / benchmark.total_time.as_secs_f64();
        println!("🚀 SPEEDUP: {:.1}x faster than brute force ({:?} vs 71s)", speedup, benchmark.total_time);
    } else {
        let slowdown = benchmark.total_time.as_secs_f64() / brute_force_baseline.as_secs_f64();
        println!("🐌 SLOWER: {:.1}x slower than brute force ({:?} vs 71s)", slowdown, benchmark.total_time);
    }
}

#[cfg(not(feature = "smt"))]
pub fn run_real_world_benchmark() {
    println!("SMT feature not enabled. Run with --features smt");
}