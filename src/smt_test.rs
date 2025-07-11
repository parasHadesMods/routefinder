//! Basic Z3 integration tests for Phase 1 validation
//! 
//! Tests basic 64-bit multiplication and 32-bit rotation operations
//! to validate Z3 solver integration and bit-vector arithmetic capabilities.

use z3::*;
use z3::ast::Ast;

/// Test basic Z3 context creation and simple constraint solving
pub fn test_z3_basic() -> Result<(), String> {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    
    // Create a simple bit-vector variable
    let x = ast::BV::new_const(&ctx, "x", 32);
    let five = ast::BV::from_u64(&ctx, 5, 32);
    
    // Add constraint: x == 5
    solver.assert(&x._eq(&five));
    
    // Check satisfiability
    match solver.check() {
        SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let x_value = model.eval(&x, true).unwrap();
            println!("Basic Z3 test passed: x = {}", x_value);
            Ok(())
        }
        SatResult::Unsat => Err("Basic Z3 test failed: unsat".to_string()),
        SatResult::Unknown => Err("Basic Z3 test failed: unknown".to_string()),
    }
}

/// Test 64-bit multiplication operations
pub fn test_64bit_multiplication() -> Result<(), String> {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    
    // Test LCG multiplication: state * LCG_MULTIPLIER
    const LCG_MULTIPLIER: u64 = 0x5851f42d4c957f2d;
    
    let state = ast::BV::new_const(&ctx, "state", 64);
    let multiplier = ast::BV::from_u64(&ctx, LCG_MULTIPLIER, 64);
    let expected_result = ast::BV::from_u64(&ctx, 0x1234567890abcdef_u64.wrapping_mul(LCG_MULTIPLIER), 64);
    
    // Add constraint: state * multiplier == expected_result
    let product = state.bvmul(&multiplier);
    solver.assert(&product._eq(&expected_result));
    
    // Add constraint to find the original state value
    solver.assert(&state._eq(&ast::BV::from_u64(&ctx, 0x1234567890abcdef, 64)));
    
    match solver.check() {
        SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let state_value = model.eval(&state, true).unwrap();
            let product_value = model.eval(&product, true).unwrap();
            println!("64-bit multiplication test passed:");
            println!("  state = {}", state_value);
            println!("  product = {}", product_value);
            Ok(())
        }
        SatResult::Unsat => Err("64-bit multiplication test failed: unsat".to_string()),
        SatResult::Unknown => Err("64-bit multiplication test failed: unknown".to_string()),
    }
}

/// Test 32-bit rotation operations (for PCG output function)
pub fn test_32bit_rotation() -> Result<(), String> {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    
    // Test variable-amount right rotation as needed for PCG
    let value = ast::BV::new_const(&ctx, "value", 32);
    let rotation_amount = ast::BV::new_const(&ctx, "rot", 32);
    
    // Test a known rotation: 0x12345678 rotated right by 8 positions
    // Should result in 0x78123456
    let test_value = ast::BV::from_u64(&ctx, 0x12345678, 32);
    let test_rotation = ast::BV::from_u64(&ctx, 8, 32);
    let expected_result = ast::BV::from_u64(&ctx, 0x78123456, 32);
    
    // Implement right rotation using shifts and OR
    // rotr(x, n) = (x >> n) | (x << (32 - n))
    let thirty_two = ast::BV::from_u64(&ctx, 32, 32);
    let right_shift = test_value.bvlshr(&test_rotation);
    let left_shift_amount = thirty_two.bvsub(&test_rotation);
    let left_shift = test_value.bvshl(&left_shift_amount);
    let rotated = right_shift.bvor(&left_shift);
    
    solver.assert(&value._eq(&test_value));
    solver.assert(&rotation_amount._eq(&test_rotation));
    solver.assert(&rotated._eq(&expected_result));
    
    match solver.check() {
        SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let value_result = model.eval(&value, true).unwrap();
            let rotation_result = model.eval(&rotation_amount, true).unwrap();
            let rotated_result = model.eval(&rotated, true).unwrap();
            println!("32-bit rotation test passed:");
            println!("  value = {}", value_result);
            println!("  rotation = {}", rotation_result);
            println!("  rotated = {}", rotated_result);
            Ok(())
        }
        SatResult::Unsat => Err("32-bit rotation test failed: unsat".to_string()),
        SatResult::Unknown => Err("32-bit rotation test failed: unknown".to_string()),
    }
}

/// Test bit-vector constraints (simplified - no mixed theory for now)
pub fn test_bitvector_constraints() -> Result<(), String> {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    
    // Create a 32-bit value and add range constraints using bit-vector operations
    let bv_value = ast::BV::new_const(&ctx, "bv", 32);
    
    // Add constraint: 0x40000000 <= bv_value <= 0x80000000 (roughly 25%-50% range)
    let lower_bound = ast::BV::from_u64(&ctx, 0x40000000, 32);
    let upper_bound = ast::BV::from_u64(&ctx, 0x80000000, 32);
    
    solver.assert(&bv_value.bvuge(&lower_bound));
    solver.assert(&bv_value.bvule(&upper_bound));
    
    match solver.check() {
        SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let bv_result = model.eval(&bv_value, true).unwrap();
            println!("Bit-vector constraint test passed:");
            println!("  bit-vector = {}", bv_result);
            Ok(())
        }
        SatResult::Unsat => Err("Bit-vector constraint test failed: unsat".to_string()),
        SatResult::Unknown => Err("Bit-vector constraint test failed: unknown".to_string()),
    }
}

/// Run all Z3 integration tests
pub fn run_all_tests() {
    println!("=== Z3 Integration Tests ===");
    
    let tests = [
        ("Basic Z3 functionality", test_z3_basic as fn() -> Result<(), String>),
        ("64-bit multiplication", test_64bit_multiplication as fn() -> Result<(), String>),
        ("32-bit rotation", test_32bit_rotation as fn() -> Result<(), String>),
        ("Bit-vector constraints", test_bitvector_constraints as fn() -> Result<(), String>),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (name, test_fn) in tests.iter() {
        print!("Testing {}: ", name);
        match test_fn() {
            Ok(()) => {
                println!("PASS");
                passed += 1;
            }
            Err(e) => {
                println!("FAIL - {}", e);
                failed += 1;
            }
        }
    }
    
    println!("\n=== Test Summary ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Total:  {}", passed + failed);
    
    if failed == 0 {
        println!("All tests passed! Z3 integration is working correctly.");
    } else {
        println!("Some tests failed. Check Z3 installation and bindings.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_z3_integration() {
        run_all_tests();
    }
}