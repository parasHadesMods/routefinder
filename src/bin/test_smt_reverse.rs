//! Test binary for SMT reverse engineering implementation
//! Tests Phase 2.1: Basic LCG State Advancement

#[cfg(feature = "smt")]
fn main() {
    use routefinder::smt_reverse_engineer;
    use routefinder::reverse_rng::data_point::DataPoint;
    
    println!("=== Testing SMT Reverse Engineering (Phase 2.1) ===");
    
    // Test 1: Simple single constraint
    println!("\nTest 1: Single constraint");
    let single_constraint = vec![
        DataPoint {
            offset: 0,
            range_min: 0.0,
            range_max: 1.0,
            observed: 0.5,
            name: "simple_test".to_string(),
        }
    ];
    
    match smt_reverse_engineer::smt_reverse_engineer(&single_constraint) {
        Ok(candidates) => {
            println!("  Found {} candidate(s)", candidates.len());
            for (i, candidate) in candidates.iter().enumerate() {
                println!("    Candidate {}: seed={}, state={:016x}", 
                         i + 1, candidate.seed, candidate.state);
            }
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }
    
    // Test 2: Multiple constraints with different offsets (small steps for now)
    println!("\nTest 2: Multiple constraints");
    let multi_constraints = vec![
        DataPoint {
            offset: 0,
            range_min: 0.0,
            range_max: 1.0,
            observed: 0.25,
            name: "first".to_string(),
        },
        DataPoint {
            offset: 1,
            range_min: 0.0,
            range_max: 1.0,  
            observed: 0.75,
            name: "second".to_string(),
        },
        DataPoint {
            offset: 2,
            range_min: 0.0,
            range_max: 1.0,
            observed: 0.33,
            name: "third".to_string(),
        }
    ];
    
    match smt_reverse_engineer::smt_reverse_engineer(&multi_constraints) {
        Ok(candidates) => {
            println!("  Found {} candidate(s)", candidates.len());
            for (i, candidate) in candidates.iter().enumerate() {
                println!("    Candidate {}: seed={}, state={:016x}", 
                         i + 1, candidate.seed, candidate.state);
            }
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }
    
    // Test 3: Conflicting constraints (should return no solutions)
    println!("\nTest 3: Conflicting constraints (should be unsatisfiable)");
    let conflicting_constraints = vec![
        DataPoint {
            offset: 0,
            range_min: 0.0,
            range_max: 1.0,
            observed: 0.1,  // Very specific value
            name: "first".to_string(),
        },
        DataPoint {
            offset: 0,  // Same offset but different observed value
            range_min: 0.0,
            range_max: 1.0,
            observed: 0.9,  // Conflicting value
            name: "conflict".to_string(),
        }
    ];
    
    match smt_reverse_engineer::smt_reverse_engineer(&conflicting_constraints) {
        Ok(candidates) => {
            if candidates.is_empty() {
                println!("  Correctly detected unsatisfiable constraints");
            } else {
                println!("  Unexpected: found {} candidates for conflicting constraints", candidates.len());
            }
        }
        Err(e) => {
            println!("  Error: {}", e);
        }
    }
    
    println!("\n=== SMT Reverse Engineering Tests Complete ===");
}

#[cfg(not(feature = "smt"))]
fn main() {
    println!("SMT feature not enabled. Run with: cargo run --bin test_smt_reverse --features smt");
}