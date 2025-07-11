# RNG State Reverse Engineering - Implementation Plan

## Problem Overview
Determine the original 64-bit PCG RNG state from observable behavior data points. **NEW CONSTRAINT**: The solution is guaranteed to be the result of seeding the RNG with a specific signed 32-bit integer value (i.e., equivalent to using the set-seed function in the CLI).

Each data point contains:
- Offset from original state (accumulated RNG advances)
- Range (min, max) for random number generation
- Observed rounded result (2 decimal places)

### Data Collection Trade-offs
- **Data Collection Cost**: 2 seconds per data point for experienced user
- **Execution Time**: Decreases with more data points (better constraints)
- **Optimization Goal**: Minimize total time = (data points × 2 seconds) + execution time
- **Solution Output**: Recommend optimal number of data points to collect

## Technical Constraints & Analysis

### RNG Implementation Details
- **Generator**: PCG XSH RR 64/32 (64-bit state, 32-bit output)
- **State Space**: ~~2^64 possible states~~ **REDUCED TO 2^32 possible states** (4.3 billion) due to seed constraint
- **Seed Constraint**: Solution must be reachable via `SggPcg::seed(i32_value as u64)`
- **LCG Constants**: MULTIPLIER = 0x5851f42d4c957f2d, INCREMENT = 0xb47c73972972b7b7
- **Output Function**: XSH RR (xorshift high, random rotate)
- **Advance Function**: Efficient O(log n) state jumping

### Data Point Processing
- u32 → fraction: `value / u32::MAX`
- Scale to range: `fraction * (max - min) + min`
- Round to 2 decimals: Creates uncertainty bands
- Multiple data points create constraint system

### Performance Requirements
- ~~Brute force (2^64 states): Infeasible (~584 years at 1B checks/sec)~~
- **NEW: Brute force (2^32 states): Feasible (~4.3 seconds at 1B checks/sec)**
- Target: Complete search in < 10 seconds **EASILY ACHIEVABLE**
- **Dramatic improvement**: 2^32 reduction in search space eliminates need for complex algorithms

## Implementation Plan

### Phase 1: Core Infrastructure
1. **CLI Interface**
   - New subcommand: `cargo run -- reverse-rng input.txt`
   - Input parser for data point format
   - Output original state candidate(s)
   - **Data Point Recommendation**: Suggest optimal number of data points to collect

2. **Data Structures**
   ```rust
   struct DataPoint {
       offset: u64,
       range_min: f64,
       range_max: f64,
       observed: f64,
       name: String, // for debugging
   }
   
   struct StateCandidate {
       state: u64,
       confidence: f64,
       error_metrics: Vec<f64>,
   }
   
   struct TimeOptimization {
       data_points: usize,
       estimated_execution_time: f64,
       total_time_seconds: f64, // collection + execution
   }
   ```

3. **Constraint Analysis**
   - For each data point, calculate possible u32 value ranges
   - Account for rounding errors (±0.005 precision)
   - Create validity checking function for state candidates
   - **Performance Modeling**: Estimate execution time based on constraint strength

### Phase 2: Simplified Search Strategy (Due to Reduced Search Space)

#### Primary Strategy: Direct Brute Force with Early Termination
1. **Seed Space Iteration**: Test all i32 values (-2^31 to 2^31-1)
2. **State Generation**: For each seed, compute `SggPcg::seed(seed as u64)` to get initial state
3. **Constraint Validation**: Verify all data points against the seeded state
4. **Early Termination**: Stop on first invalid data point for each candidate

#### Secondary Strategy: Constraint Pre-filtering (Optional Optimization)
1. **Range Calculation**: For each data point, compute valid u32 ranges
2. **Seed Elimination**: Quickly eliminate impossible seeds before full validation
3. **Parallel Processing**: Test multiple seeds simultaneously

### Phase 3: Performance Optimizations (Now Optional)

#### Simple Optimizations (Low Priority)
1. **Parallel Processing**: Multi-threaded seed testing
2. **Early Termination**: Skip validation on first constraint failure
3. **SIMD Instructions**: Batch process multiple seeds simultaneously

#### Advanced Optimizations (Likely Unnecessary)
1. **Smart Ordering**: Test more likely seed ranges first (e.g., smaller absolute values)
2. **Constraint Scoring**: Order data points by selectivity
3. **Adaptive Batching**: Adjust chunk sizes based on constraint hit rates

### Phase 4: Testing & Benchmarking Framework

#### Test Data Generation
1. **Known State Tests**: Generate test data from known RNG states
2. **Synthetic Datasets**: Various numbers of data points (1-100)
3. **Edge Cases**: Near-boundary values, minimal precision differences
4. **Adversarial Cases**: Maximum spacing between data points

#### Performance Benchmarks
1. **Accuracy Metrics**: 
   - Success rate for finding correct state
   - Time to first correct candidate
   - False positive rate
2. **Performance Metrics**:
   - States tested per second
   - Memory usage
   - Scaling with number of data points
3. **Time Optimization Analysis**:
   - Execution time vs. number of data points curve
   - Identify optimal data point count for minimum total time
   - Model: `total_time = (data_points × 2) + execution_time(data_points)`

#### Validation Strategy
```rust
fn validate_solution(original_state: u64, data_points: &[DataPoint]) -> bool {
    let mut test_rng = SggPcg::new(original_state);
    for point in data_points {
        test_rng.advance(point.offset);
        let generated = test_rng.next_u32();
        let fraction = generated as f64 / u32::MAX as f64;
        let scaled = fraction * (point.range_max - point.range_min) + point.range_min;
        let rounded = (scaled * 100.0).round() / 100.0;
        if (rounded - point.observed).abs() > 0.005 {
            return false;
        }
    }
    true
}
```

### Phase 5: Optimization & Refinement

#### Performance Optimizations
1. **SIMD Instructions**: Parallel state testing
2. **Cache Optimization**: Minimize memory access patterns
3. **Early Termination**: Skip impossible branches quickly
4. **Parallel Processing**: Multi-threaded search

#### Algorithmic Improvements
1. **Smart Sampling**: Focus on high-probability state regions
2. **Adaptive Precision**: Adjust search granularity based on constraints
3. **Constraint Ordering**: Process most restrictive constraints first

## Implementation Phases (REVISED - Much Simpler)

### Phase 1: Core Implementation (1 day)
- [ ] CLI interface and input parsing
- [ ] Simple brute force seed iteration (-2^31 to 2^31-1)
- [ ] State validation function using `SggPcg::seed()`
- [ ] Basic test cases
- [ ] Progress reporting

### Phase 2: Basic Optimizations (0.5-1 day)
- [ ] Early termination on constraint failures
- [ ] Multi-threading for parallel seed testing
- [ ] Simple performance benchmarking
- [ ] Error handling and edge cases

### Phase 3: Polish & Documentation (0.5 day)
- [ ] Usage examples
- [ ] Performance documentation
- [ ] Input validation and error messages
- [ ] Final testing

**TOTAL ESTIMATED TIME: 2-2.5 days (down from 7-12 days)**

## Success Criteria (REVISED)
- **Accuracy**: 100% success rate when correct seed exists in search space
- **Performance**: Complete search in <5 seconds for typical cases (was <10 seconds)
- **Simplicity**: Straightforward brute force approach, no complex algorithms needed
- **Data Points**: Need 6-7 data points for unique identification (each eliminates 95% of remaining search space)
- **Robustness**: Handle edge cases and invalid inputs gracefully
- **Usability**: Clear CLI interface and error messages

## Risk Mitigation
- **Mathematical Feasibility**: Prototype constraint analysis early
- **Performance Bottlenecks**: Implement benchmarking from start
- **False Positives**: Implement rigorous validation
- **Scope Creep**: Focus on core functionality first

## File Structure
```
src/
├── main.rs (add reverse-rng subcommand)
├── rng.rs (existing)
├── reverse_rng/
│   ├── mod.rs
│   ├── cli.rs (argument parsing)
│   ├── data_point.rs (data structures)
│   ├── constraint.rs (range calculations)
│   ├── search.rs (core algorithms)
│   ├── validate.rs (solution verification)
│   ├── benchmark.rs (performance testing)
│   └── optimizer.rs (time optimization & recommendations)
└── tests/
    └── reverse_rng_tests.rs
```

## Expected Time Optimization Curve (REVISED - CORRECTED)
With the 2^32 search space and severe rounding constraints:
- **1 data point**: ~2^32/20 ≈ 200M possible seeds → insufficient
- **2 data points**: ~200M/20 ≈ 10M possible seeds → still too many
- **3 data points**: ~10M/20 ≈ 500K possible seeds → getting manageable
- **4 data points**: ~500K/20 ≈ 25K possible seeds → small search space
- **5 data points**: ~25K/20 ≈ 1.25K possible seeds → very manageable
- **6 data points**: ~1.25K/20 ≈ 62 possible seeds → nearly unique
- **7 data points**: ~62/20 ≈ 3 possible seeds → unique identification

**Constraint Math**: Each data point divides search space by ~20 (due to rounding into ~20 equal regions)
- Need log₂₀(2^32) ≈ log₂₀(4.3B) ≈ 7.3 data points for unique identification
- **Practical optimum**: 6-7 data points for confident unique identification

**Execution time**: Still ~2-5 seconds regardless of data point count
**New formula**: `total_time = (n × 2 seconds) + ~3 seconds`
**Optimal range**: 6-7 data points balancing collection time vs. identification confidence