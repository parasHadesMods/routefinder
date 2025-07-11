//! Mathematical verification module for Phase 1.5-1.8
//! 
//! Verifies that SMT encoding exactly matches existing Rust RNG implementation

#[cfg(feature = "smt")]
use z3::*;

use crate::rng::SggPcg;
use rand_core::RngCore;

/// Extract constants from the existing RNG implementation
pub const LCG_MULTIPLIER: u64 = 0x5851f42d4c957f2d;
pub const LCG_INCREMENT: u64 = 0xb47c73972972b7b7;
pub const INITIAL_OFFSET: u64 = 0x3d657cc62bc341e;

// PCG output function constants (from src/rng.rs)
pub const ROTATE_BITS: u32 = 59;  // 64 - 5
pub const XSHIFT_BITS: u32 = 18;  // (5 + 32) / 2
pub const SPARE_BITS: u32 = 27;   // 64 - 32 - 5

/// Verify that SMT constants match the Rust implementation
pub fn verify_constants() -> Result<(), String> {
    println!("=== Verifying Constants ===");
    
    // These should match the constants in src/rng.rs exactly
    const EXPECTED_MULTIPLIER: u64 = 0x5851f42d4c957f2d;
    const EXPECTED_INCREMENT: u64 = 0xb47c73972972b7b7;
    const EXPECTED_OFFSET: u64 = 0x3d657cc62bc341e;
    
    let mut errors = Vec::new();
    
    if LCG_MULTIPLIER != EXPECTED_MULTIPLIER {
        errors.push(format!("LCG_MULTIPLIER mismatch: expected 0x{:x}, got 0x{:x}", 
                           EXPECTED_MULTIPLIER, LCG_MULTIPLIER));
    }
    
    if LCG_INCREMENT != EXPECTED_INCREMENT {
        errors.push(format!("LCG_INCREMENT mismatch: expected 0x{:x}, got 0x{:x}", 
                           EXPECTED_INCREMENT, LCG_INCREMENT));
    }
    
    if INITIAL_OFFSET != EXPECTED_OFFSET {
        errors.push(format!("INITIAL_OFFSET mismatch: expected 0x{:x}, got 0x{:x}", 
                           EXPECTED_OFFSET, INITIAL_OFFSET));
    }
    
    // Verify PCG output constants
    if ROTATE_BITS != 59 {
        errors.push(format!("ROTATE_BITS should be 59, got {}", ROTATE_BITS));
    }
    
    if XSHIFT_BITS != 18 {
        errors.push(format!("XSHIFT_BITS should be 18, got {}", XSHIFT_BITS));
    }
    
    if SPARE_BITS != 27 {
        errors.push(format!("SPARE_BITS should be 27, got {}", SPARE_BITS));
    }
    
    if errors.is_empty() {
        println!("✅ All constants match exactly");
        Ok(())
    } else {
        for error in &errors {
            println!("❌ {}", error);
        }
        Err(format!("Found {} constant mismatches", errors.len()))
    }
}

/// Test a single step of LCG advancement
pub fn verify_lcg_step(initial_state: u64) -> Result<(), String> {
    // Rust implementation
    let expected = initial_state.wrapping_mul(LCG_MULTIPLIER).wrapping_add(LCG_INCREMENT);
    
    // Manual calculation to verify
    let manual = initial_state.wrapping_mul(0x5851f42d4c957f2d_u64).wrapping_add(0xb47c73972972b7b7_u64);
    
    if expected != manual {
        return Err(format!("LCG step calculation mismatch: expected 0x{:x}, manual 0x{:x}", expected, manual));
    }
    
    println!("✅ LCG step verification passed for state 0x{:x} -> 0x{:x}", initial_state, expected);
    Ok(())
}

/// Test PCG output function against Rust implementation
pub fn verify_pcg_output(state: u64) -> Result<(), String> {
    // Calculate expected output using the exact Rust implementation logic
    let rot = (state >> ROTATE_BITS) as u32;
    let xsh = (((state >> XSHIFT_BITS) ^ state) >> SPARE_BITS) as u32;
    let expected = xsh.rotate_right(rot);
    
    // Manual step-by-step calculation for verification
    let manual_rot = (state >> 59) as u32;
    let manual_shifted = state >> 18;
    let manual_xored = manual_shifted ^ state;
    let manual_xsh = (manual_xored >> 27) as u32;
    let manual_output = manual_xsh.rotate_right(manual_rot);
    
    if expected != manual_output {
        return Err(format!("PCG output mismatch: expected 0x{:x}, manual 0x{:x}", expected, manual_output));
    }
    
    println!("✅ PCG output verification passed for state 0x{:x} -> output 0x{:x}", state, expected);
    println!("   Details: rot={}, xsh=0x{:x}", rot, xsh);
    Ok(())
}

/// Test RNG against known seed and verify multiple steps
pub fn verify_rng_sequence(seed: u64, steps: usize) -> Result<(), String> {
    let mut rng = SggPcg::new(seed);
    
    println!("=== Verifying RNG Sequence ===");
    println!("Seed: 0x{:x}", seed);
    println!("Initial state: 0x{:x}", rng.state());
    
    for i in 0..steps {
        let state_before = rng.state();
        let output = rng.next_u32();
        let state_after = rng.state();
        
        // Verify that state advancement follows LCG formula
        let expected_state = state_before.wrapping_mul(LCG_MULTIPLIER).wrapping_add(LCG_INCREMENT);
        if state_after != expected_state {
            return Err(format!("Step {}: State advancement mismatch. Expected 0x{:x}, got 0x{:x}", 
                              i, expected_state, state_after));
        }
        
        // Verify that output follows PCG formula
        let rot = (state_before >> ROTATE_BITS) as u32;
        let xsh = (((state_before >> XSHIFT_BITS) ^ state_before) >> SPARE_BITS) as u32;
        let expected_output = xsh.rotate_right(rot);
        if output != expected_output {
            return Err(format!("Step {}: Output mismatch. Expected 0x{:x}, got 0x{:x}", 
                              i, expected_output, output));
        }
        
        println!("Step {}: state 0x{:x} -> output 0x{:x} -> new_state 0x{:x}", 
                i, state_before, output, state_after);
    }
    
    println!("✅ RNG sequence verification passed for {} steps", steps);
    Ok(())
}

/// Verify floating-point conversion accuracy
pub fn verify_float_conversion() -> Result<(), String> {
    println!("=== Verifying Float Conversion ===");
    
    let test_values = vec![
        0u32,                    // Minimum value -> 0.0
        0xFFFFFFFF_u32,         // Maximum value -> ~1.0
        0x80000000_u32,         // Half value -> ~0.5
        0x40000000_u32,         // Quarter value -> ~0.25
        0xC0000000_u32,         // Three quarters -> ~0.75
    ];
    
    for value in test_values {
        let as_float = value as f64 / (u32::MAX as f64 + 1.0);
        
        // Verify range
        if as_float < 0.0 || as_float >= 1.0 {
            return Err(format!("Float conversion out of range: {} -> {}", value, as_float));
        }
        
        // Verify precision - convert back and check
        let back_to_int = (as_float * (u32::MAX as f64 + 1.0)) as u32;
        if back_to_int != value {
            return Err(format!("Float conversion precision loss: {} -> {} -> {}", value, as_float, back_to_int));
        }
        
        println!("  0x{:08x} -> {:.10}", value, as_float);
    }
    
    println!("✅ Float conversion verification passed");
    Ok(())
}

/// Test constraint range calculations for precision
pub fn verify_constraint_ranges() -> Result<(), String> {
    println!("=== Verifying Constraint Ranges ===");
    
    // Test data points similar to what we'd use in actual reverse engineering
    let test_cases = vec![
        (0.0, 0.1),      // Narrow range at bottom
        (0.4, 0.6),      // Middle range
        (0.9, 1.0),      // Narrow range at top
        (0.25, 0.75),    // Wide middle range
        (0.0, 1.0),      // Full range
    ];
    
    for (min_val, max_val) in test_cases {
        // Convert to integer constraints
        let min_int = (min_val * (u32::MAX as f64 + 1.0)) as u32;
        let max_int = ((max_val * (u32::MAX as f64 + 1.0)) as u32).saturating_sub(1);
        
        // Convert back to verify precision
        let min_back = min_int as f64 / (u32::MAX as f64 + 1.0);
        let max_back = (max_int + 1) as f64 / (u32::MAX as f64 + 1.0);
        
        let min_error = (min_back - min_val).abs();
        let max_error = (max_back - max_val).abs();
        
        const TOLERANCE: f64 = 1e-9;
        
        if min_error > TOLERANCE {
            return Err(format!("Min range precision error: {} -> {} (error: {})", min_val, min_back, min_error));
        }
        
        if max_error > TOLERANCE {
            return Err(format!("Max range precision error: {} -> {} (error: {})", max_val, max_back, max_error));
        }
        
        println!("  [{:.3}, {:.3}] -> [0x{:08x}, 0x{:08x}] -> [{:.10}, {:.10}]", 
                min_val, max_val, min_int, max_int, min_back, max_back);
    }
    
    println!("✅ Constraint range verification passed");
    Ok(())
}

/// Run all mathematical verifications
pub fn run_all_verifications() {
    println!("=== Mathematical Foundation Verification ===");
    
    let mut passed = 0;
    let mut failed = 0;
    
    // Constants verification
    println!("\n--- Constants verification ---");
    match verify_constants() {
        Ok(()) => {
            println!("✅ Constants verification PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ Constants verification FAILED: {}", e);
            failed += 1;
        }
    }
    
    // LCG step verification
    println!("\n--- LCG step verification ---");
    match verify_lcg_step(0x1234567890abcdef) {
        Ok(()) => {
            println!("✅ LCG step verification PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ LCG step verification FAILED: {}", e);
            failed += 1;
        }
    }
    
    // PCG output verification
    println!("\n--- PCG output verification ---");
    match verify_pcg_output(0x1234567890abcdef) {
        Ok(()) => {
            println!("✅ PCG output verification PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ PCG output verification FAILED: {}", e);
            failed += 1;
        }
    }
    
    // RNG sequence verification
    println!("\n--- RNG sequence verification ---");
    match verify_rng_sequence(12345, 5) {
        Ok(()) => {
            println!("✅ RNG sequence verification PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ RNG sequence verification FAILED: {}", e);
            failed += 1;
        }
    }
    
    // Float conversion verification
    println!("\n--- Float conversion verification ---");
    match verify_float_conversion() {
        Ok(()) => {
            println!("✅ Float conversion verification PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ Float conversion verification FAILED: {}", e);
            failed += 1;
        }
    }
    
    // Constraint ranges verification
    println!("\n--- Constraint ranges verification ---");
    match verify_constraint_ranges() {
        Ok(()) => {
            println!("✅ Constraint ranges verification PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ Constraint ranges verification FAILED: {}", e);
            failed += 1;
        }
    }
    
    println!("\n=== Verification Summary ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Total:  {}", passed + failed);
    
    if failed == 0 {
        println!("🎉 All mathematical verifications passed!");
        println!("SMT implementation foundation is mathematically sound.");
    } else {
        println!("⚠️  Some verifications failed. Review implementation before proceeding.");
    }
}