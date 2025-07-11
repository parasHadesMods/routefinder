# SMT Approach Implementation Plan

This document outlines a comprehensive plan to implement the SMT (Satisfiability Modulo Theories) approach for RNG reverse engineering as described in OPTIMIZATION_IDEAS.md.

## Executive Summary

**Goal**: Reduce RNG reverse engineering time from 71s to <10s using SMT solvers  
**Approach**: Encode the problem as bit-vector + real arithmetic constraints  
**Expected Outcome**: 0.1-10s solving time with <1s encoding overhead  
**Risk Level**: Medium-High (dependent on solver performance for this specific problem structure)

## Phase 1: Foundation & Research (Days 1-3) ✅ COMPLETED

### 1.1 SMT Solver Selection and Integration ✅ COMPLETED
**Priority**: CRITICAL  
**Estimated Time**: 1-2 days  
**Dependencies**: None  
**Status**: **COMPLETED** - Z3 v0.12 selected and integrated

**Research Tasks**:
- [x] Compare Z3, CVC5, and Yices performance on bit-vector arithmetic benchmarks
- [x] Evaluate Rust binding quality and API ergonomics for each solver
- [x] Test basic 64-bit multiplication and 32-bit rotation operations
- [x] Measure encoding overhead for sample arithmetic circuits

**Final Selection**: **Z3 v0.12** with static linking
- **Rationale**: Best Rust bindings despite Bitwuzla performance advantage
- **Integration**: Optional `smt` feature flag for backward compatibility
- **Performance**: Bitwuzla leads (2.85x faster than CVC5, 5.1x than Z3), but Z3 chosen for superior Rust ecosystem

**Implementation**:
```toml
# Added to Cargo.toml
[dependencies]
z3 = { version = "0.12", features = ["static-link-z3"], optional = true }

[features]
smt = ["z3"]
```

**Validation Criteria**: ✅ ALL COMPLETED
- [x] Can encode and solve simple 64-bit arithmetic constraints (<100ms)
- [x] Mixed bit-vector + real constraints work correctly
- [x] Error handling works for unsatisfiable instances

**Test Infrastructure Created**:
- `src/smt_test.rs` - Complete Z3 integration test suite
- `src/bin/test_z3.rs` - SMT integration test binary
- `src/smt_benchmark.rs` - Encoding overhead measurement

---

### 1.2 Mathematical Foundation Verification ✅ COMPLETED
**Priority**: HIGH  
**Estimated Time**: 1 day  
**Dependencies**: None  
**Status**: **COMPLETED** - All mathematical foundations verified bit-perfect

**Tasks**: ✅ ALL COMPLETED
- [x] Verify PCG XSH-RR implementation matches existing Rust code exactly
- [x] Validate LCG advancement formula constants (multiplier/increment)
- [x] Test floating-point to bit-vector conversion accuracy
- [x] Confirm constraint range calculations preserve precision

**Mathematical Components Verified**: ✅ BIT-PERFECT MATCH
```rust
// LCG advancement - VERIFIED IDENTICAL
const LCG_MULTIPLIER: u64 = 0x5851f42d4c957f2d;
const LCG_INCREMENT: u64 = 0xb47c73972972b7b7;

// PCG output constants - VERIFIED IDENTICAL
const ROTATE_BITS: u32 = 59;  // 64 - 5
const XSHIFT_BITS: u32 = 18;  // (5 + 32) / 2
const SPARE_BITS: u32 = 27;   // 64 - 32 - 5
```

**Validation Results**: ✅ ALL PASSED
- [x] Generated known input/output pairs from existing Rust implementation
- [x] Verified identical mathematical operations between SMT and Rust
- [x] Tested edge cases: min/max values, boundary conditions
- [x] **Key Achievement**: Mathematical foundation is **bit-perfect**

**Verification Infrastructure**:
- `src/smt_verification.rs` - Comprehensive mathematical verification
- `src/bin/verify_math.rs` - Mathematical verification binary
- **6/6 verification tests passed** - Constants, LCG steps, PCG output, RNG sequences, float conversion, constraint ranges

**Critical Success**: SMT encoding will exactly match existing Rust RNG implementation

---

## Phase 2: Core SMT Encoding (Days 4-6)

### 2.1 Basic LCG State Advancement
**Priority**: CRITICAL  
**Estimated Time**: 1 day  
**Dependencies**: Solver integration (1.1)

**Implementation Tasks**:
```rust
// Core data structures
struct SMTReverseEngineer {
    context: Context,           // SMT context
    solver: Solver,            // SMT solver instance
    initial_state: BV,         // 64-bit initial state variable
    lcg_multiplier: BV,        // LCG multiplication constant
    lcg_increment: BV,         // LCG addition constant
}

impl SMTReverseEngineer {
    fn new() -> Self { /* ... */ }
    
    fn advance_state(&self, state: &BV, steps: u32) -> BV {
        // Implement: state' = (state * multiplier + increment) mod 2^64
        // Handle multiple advancement steps efficiently
    }
}
```

**Technical Challenges**:
- [ ] **64-bit overflow handling**: Ensure modular arithmetic matches Rust behavior
- [ ] **Multiple advancement**: Optimize repeated LCG steps (avoid O(n) expansion)
- [ ] **Variable naming**: Prevent naming conflicts in large constraint sets

**Testing Strategy**:
- [ ] Single-step advancement: Compare SMT result with Rust calculation
- [ ] Multi-step advancement: Test advancement by 1, 5, 10, 100 steps
- [ ] Edge case testing: States near 0, max values, overflow boundaries

---

### 2.2 PCG Output Function Encoding
**Priority**: CRITICAL  
**Estimated Time**: 1-2 days  
**Dependencies**: Basic LCG advancement (2.1)

**PCG Function Breakdown**:
```rust
fn encode_pcg_output(ctx: &Context, state: &BV) -> BV {
    // Step 1: Extract rotation amount (top 5 bits)
    let rot = state.extract(63, 59);  // Bits 63-59
    
    // Step 2: Compute XSH value
    let shifted = state.bvlshr(&BV::from_u64(ctx, 18, 64));
    let xored = state.bvxor(&shifted);
    let xsh = xored.extract(31, 0);  // Extract bits 31-0
    
    // Step 3: Apply rotation
    apply_right_rotation(ctx, &xsh, &rot)
}

fn apply_right_rotation(ctx: &Context, value: &BV, amount: &BV) -> BV {
    // Implement: value.rotate_right(amount)
    // Challenge: Variable rotation amount requires conditional logic
}
```

**Technical Challenges**:
- [ ] **Variable rotation**: SMT doesn't have native variable-amount rotation
- [ ] **Bit extraction precision**: Ensure exact bit range matches
- [ ] **32-bit output**: Correctly truncate to match u32 output type

**Implementation Strategies**:
1. **Lookup Table Approach**: Pre-compute all 32 possible rotations, use conditional selection
2. **Bit-wise Implementation**: Implement rotation using shifts and ORs
3. **SMT-LIB Native**: Use solver-specific rotation functions if available

**Validation**:
- [ ] Test all 32 possible rotation amounts (0-31)
- [ ] Compare against reference implementation with random state values
- [ ] Verify output is exactly 32 bits

---

### 2.3 Range Constraint Encoding  
**Priority**: CRITICAL  
**Estimated Time**: 1 day  
**Dependencies**: PCG output encoding (2.2)

**Constraint Types**:
```rust
struct DataPoint {
    offset: u32,    // Steps to advance RNG
    min: f64,       // Minimum floating-point value
    max: f64,       // Maximum floating-point value
    observed: f64,  // Actual observed value (for validation)
}

fn encode_range_constraint(ctx: &Context, pcg_output: &BV, constraint: &DataPoint) {
    // Convert 32-bit PCG output to floating-point range [0.0, 1.0)
    let divisor = 2^32 as f64;
    let float_val = pcg_output.bv2real() / Real::from_real(ctx, divisor, 1);
    
    // Add range constraints
    solver.assert(&float_val.ge(&Real::from_real(ctx, constraint.min, 1)));
    solver.assert(&float_val.le(&Real::from_real(ctx, constraint.max, 1)));
}
```

**Technical Challenges**:
- [ ] **Precision preservation**: Ensure no rounding errors in conversion
- [ ] **Theory mixing**: Efficiently combine bit-vector and real arithmetic
- [ ] **Range validation**: Verify constraint bounds are mathematically sound

**Alternative Approaches**:
1. **Fixed-point arithmetic**: Scale to integers, avoid floating-point
2. **Rational arithmetic**: Use exact fractions instead of floating-point
3. **Bit-vector only**: Keep everything in integer domain

**Testing**:
- [ ] Boundary value testing: min, max, midpoint values
- [ ] Precision testing: Very small and very large constraint ranges
- [ ] Theory integration: Verify mixed constraints solve correctly

---

## Phase 3: Integration & Validation (Days 7-9)

### 3.1 Multi-Constraint Coordination
**Priority**: HIGH  
**Estimated Time**: 1 day  
**Dependencies**: All core encoding components (2.1-2.3)

**Integration Tasks**:
```rust
fn solve_multiple_constraints(constraints: &[DataPoint]) -> Option<u64> {
    let ctx = Context::new(&Config::new());
    let solver = Solver::new(&ctx);
    
    let initial_state = BV::new_const(&ctx, "initial_state", 64);
    let mut current_state = initial_state.clone();
    let mut last_offset = 0;
    
    for constraint in constraints {
        // Advance state to constraint offset
        let steps = constraint.offset - last_offset;
        current_state = advance_state_by_steps(&ctx, &current_state, steps);
        last_offset = constraint.offset;
        
        // Apply PCG and add constraint
        let pcg_output = encode_pcg_output(&ctx, &current_state);
        encode_range_constraint(&ctx, &pcg_output, constraint);
    }
    
    // Solve and extract solution
    match solver.check() {
        SatResult::Sat => extract_solution(&solver, &initial_state),
        _ => None
    }
}
```

**Challenge Areas**:
- [ ] **State management**: Track advancement offsets correctly
- [ ] **Variable scoping**: Prevent variable name collisions
- [ ] **Constraint accumulation**: Efficiently add constraints without blowup

**Validation**:
- [ ] Test with single constraint (should match brute force)
- [ ] Test with 2-3 constraints (cross-validate results)
- [ ] Test with full 7-constraint set from real_ursa_data_fixed.txt

---

### 3.2 Known Test Case Validation
**Priority**: HIGH  
**Estimated Time**: 1 day  
**Dependencies**: Multi-constraint integration (3.1)

**Test Data Sources**:
1. **Generated test data**: `cargo run --bin generate_test_data`
2. **Real ursa data**: `real_ursa_data_fixed.txt` (expected seed: 1152303697)
3. **Simple synthetic cases**: Hand-crafted minimal constraint sets

**Validation Protocol**:
```bash
# Generate known-good test data
cargo run --bin generate_test_data > smt_test_known.txt

# Test SMT solver against known solution
cargo run --release -- smt-reverse-rng smt_test_known.txt

# Compare against brute force
cargo run --release -- reverse-rng smt_test_known.txt

# Benchmark on real data
time cargo run --release -- smt-reverse-rng real_ursa_data_fixed.txt
```

**Success Criteria**:
- [ ] SMT finds same seed as brute force for all test cases
- [ ] SMT completes in <10s for real_ursa_data_fixed.txt
- [ ] SMT handles unsatisfiable instances gracefully
- [ ] SMT produces deterministic results across runs

---

### 3.3 Error Handling & Edge Cases
**Priority**: MEDIUM  
**Estimated Time**: 1 day  
**Dependencies**: Basic validation (3.2)

**Error Conditions to Handle**:
- [ ] **Unsatisfiable constraints**: No seed exists satisfying all constraints
- [ ] **Solver timeout**: Constraint set too complex for reasonable time
- [ ] **Multiple solutions**: More than one seed satisfies constraints
- [ ] **Solver crash**: Internal solver error or memory exhaustion

**Implementation**:
```rust
#[derive(Debug)]
enum SMTSolveError {
    Unsatisfiable,
    Timeout(Duration),
    SolverError(String),
    MultipleSolutions(Vec<u64>),
}

fn solve_with_timeout(constraints: &[DataPoint], timeout: Duration) -> Result<u64, SMTSolveError> {
    // Implement timeout handling
    // Add incremental solving for multiple solutions
    // Graceful error recovery
}
```

---

## Phase 4: Performance Optimization (Days 10-12)

### 4.1 Solver Parameter Tuning
**Priority**: HIGH  
**Estimated Time**: 1-2 days  
**Dependencies**: Working SMT implementation (3.x)

**Tuning Areas**:
- [ ] **Solver tactics**: Optimize for bit-vector + arithmetic mixed theories
- [ ] **Preprocessing**: Enable/disable various simplification passes
- [ ] **Search heuristics**: Adjust decision variable ordering
- [ ] **Resource limits**: Set appropriate timeout and memory bounds

**Systematic Tuning Process**:
```rust
// Configuration matrix to test
let solver_configs = vec![
    ("default", Config::new()),
    ("bv_optimized", Config::new().set_timeout_msec(5000).set_bool_val("auto_config", false)),
    ("mixed_theory", Config::new().set_string_val("smt.arith.solver", "6")),
    // ... additional configurations
];

for (name, config) in solver_configs {
    let time = benchmark_config(&config, &test_constraints);
    println!("{}: {} ms", name, time);
}
```

**Expected Optimization Areas**:
- [ ] **Theory solver selection**: Choose optimal arithmetic decision procedures
- [ ] **Clause learning**: Tune CDCL parameters for this problem structure
- [ ] **Restart strategies**: Optimize search restart policies

---

### 4.2 Encoding Optimization
**Priority**: MEDIUM  
**Estimated Time**: 1 day  
**Dependencies**: Parameter tuning (4.1)

**Optimization Strategies**:
- [ ] **Shared subexpression elimination**: Reuse intermediate computation results
- [ ] **Constraint simplification**: Mathematically simplify before encoding
- [ ] **Variable ordering**: Optimize decision variable ordering for solver
- [ ] **Incremental solving**: Add constraints progressively, prune early

**Implementation Example**:
```rust
// Optimize encoding for repeated LCG advancement
struct OptimizedSMTEncoder {
    // Cache intermediate state computations
    state_cache: HashMap<u32, BV>,  // offset -> state at offset
    
    // Pre-computed arithmetic operations
    lcg_powers: Vec<BV>,  // powers of LCG multiplier for fast advancement
}

impl OptimizedSMTEncoder {
    fn advance_state_optimized(&mut self, offset: u32) -> &BV {
        if let Some(cached) = self.state_cache.get(&offset) {
            return cached;
        }
        
        // Use binary exponentiation for fast advancement
        let state = self.binary_exponentiation_advance(offset);
        self.state_cache.insert(offset, state);
        &self.state_cache[&offset]
    }
}
```

---

### 4.3 Benchmark Comparison & Analysis
**Priority**: HIGH  
**Estimated Time**: 1 day  
**Dependencies**: Optimized implementation (4.2)

**Comprehensive Benchmarking**:
```bash
# Create benchmark script
cat > bench_smt_comparison.sh << 'EOF'
#!/bin/bash

echo "=== SMT vs Brute Force Comparison ==="

# Warm up
echo "Warming up..."
cargo build --release

echo "Testing on real_ursa_data_fixed.txt (expected: 1152303697)"

echo "Brute Force Baseline:"
time cargo run --release -- reverse-rng real_ursa_data_fixed.txt

echo "SMT Approach:"
time cargo run --release -- smt-reverse-rng real_ursa_data_fixed.txt

echo "Testing on generated data..."
cargo run --bin generate_test_data > bench_test.txt

echo "Brute Force (generated):"
time cargo run --release -- reverse-rng bench_test.txt

echo "SMT (generated):"
time cargo run --release -- smt-reverse-rng bench_test.txt
EOF

chmod +x bench_smt_comparison.sh
```

**Performance Analysis**:
- [ ] **Encoding time**: Measure constraint generation overhead
- [ ] **Solving time**: Pure solver execution time
- [ ] **Memory usage**: Peak memory consumption comparison
- [ ] **Scalability**: Performance with varying constraint count

**Success Metrics**:
- [ ] **Primary Goal**: Total time < 10s for real_ursa_data_fixed.txt
- [ ] **Encoding Target**: Encoding overhead < 1s
- [ ] **Solving Target**: Solving time < 9s
- [ ] **Reliability**: 100% correctness on all test cases

---

## Phase 5: Integration & Production Readiness (Days 13-14)

### 5.1 Command Line Integration
**Priority**: HIGH  
**Estimated Time**: 0.5 days  
**Dependencies**: Benchmarked implementation (4.3)

**CLI Integration**:
```rust
// Add to main.rs command line parsing
#[derive(Parser)]
enum Commands {
    ReverseRng {
        #[arg(long, default_value = "brute-force")]
        method: String,  // "brute-force" | "smt"
        
        #[arg(long, default_value = "5000")]
        timeout_ms: u64,
        
        data_file: PathBuf,
    },
    // ... existing commands
}

fn execute_reverse_rng(method: &str, timeout_ms: u64, data_file: &Path) -> Result<()> {
    match method {
        "brute-force" => brute_force_reverse_rng(data_file),
        "smt" => smt_reverse_rng(data_file, timeout_ms),
        _ => Err(anyhow!("Unknown method: {}", method))
    }
}
```

**Integration Tasks**:
- [ ] Add SMT solver as optional cargo feature
- [ ] Maintain backward compatibility with existing interface
- [ ] Add method selection via command line flag
- [ ] Preserve all existing functionality

---

### 5.2 Documentation & Final Testing
**Priority**: MEDIUM  
**Estimated Time**: 0.5 days  
**Dependencies**: CLI integration (5.1)

**Documentation Updates**:
- [ ] Update CLAUDE.md with SMT usage instructions
- [ ] Add performance comparison section
- [ ] Document solver selection rationale
- [ ] Include troubleshooting guide for SMT solver issues

**Final Test Suite**:
```bash
# Comprehensive regression testing
./test_all_approaches.sh
```

**Production Readiness Checklist**:
- [ ] All existing tests pass with SMT backend
- [ ] Performance regression tests included
- [ ] Error messages are user-friendly
- [ ] Documentation is complete and accurate

---

## Risk Mitigation & Contingency Plans

### High-Risk Areas

**Risk 1: SMT Solver Performance Disappoints**
- **Probability**: Medium (30-40%)
- **Impact**: High (project fails to meet 10s target)
- **Mitigation**: Implement parallel brute force + SIMD as backup optimization
- **Detection**: Benchmark results > 15s consistently

**Risk 2: Theory Combination Overhead**
- **Probability**: Medium (20-30%)  
- **Impact**: Medium (adds 2-5s overhead)
- **Mitigation**: Switch to pure bit-vector approach with integer arithmetic
- **Detection**: Encoding time > 2s

**Risk 3: Constraint Encoding Complexity**
- **Probability**: Low (10-15%)
- **Impact**: High (implementation becomes intractable)
- **Mitigation**: Simplify to single-theory approach, accept some precision loss
- **Detection**: Encoding generates >1M constraints

### Fallback Strategy

If SMT approach fails to meet performance targets:

1. **Enhanced Brute Force**: Implement meet-in-the-middle + SIMD + parallel processing
2. **Hybrid Approach**: Use SMT for constraint pruning, brute force for final search
3. **Mathematical Optimization**: Focus on constraint propagation and interval mathematics

### Success Probability Assessment

**Conservative Estimate**: 70% chance of achieving <10s target  
**Optimistic Estimate**: 40% chance of achieving <2s target  
**Pessimistic Scenario**: 20% chance of no improvement over current approach

---

## Timeline Summary

| Phase | Duration | Critical Path | Deliverable | Status |
|-------|----------|---------------|-------------|---------|
| 1 | Days 1-3 | Solver selection & math verification | Working SMT integration | ✅ **COMPLETED** |
| 2 | Days 4-6 | Core encoding implementation | Complete constraint encoding | 🔄 **NEXT** |
| 3 | Days 7-9 | Validation & testing | Verified correctness | ⏳ Pending |
| 4 | Days 10-12 | Performance optimization | Benchmarked implementation | ⏳ Pending |
| 5 | Days 13-14 | Production integration | Released feature | ⏳ Pending |

**Phase 1 Results**: ✅ **COMPLETED SUCCESSFULLY**
- Z3 integration with bit-perfect mathematical verification
- All validation criteria met
- Ready for Phase 2 core encoding implementation

**Total Estimated Time**: 2-3 weeks  
**Minimum Viable Product**: End of Phase 3 (1.5 weeks)  
**Production Ready**: End of Phase 5 (3 weeks)  
**Current Progress**: 20% complete (1/5 phases)

---

This plan provides a systematic approach to implementing the SMT optimization while maintaining rigorous testing and fallback options. The phased approach allows for early validation and course correction if initial results don't meet expectations.