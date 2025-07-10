# RNG State Reverse Engineering - Implementation Plan

## Problem Overview
Determine the original 64-bit PCG RNG state from observable behavior data points. Each data point contains:
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
- **State Space**: 2^64 possible states (18.4 quintillion)
- **LCG Constants**: MULTIPLIER = 0x5851f42d4c957f2d, INCREMENT = 0xb47c73972972b7b7
- **Output Function**: XSH RR (xorshift high, random rotate)
- **Advance Function**: Efficient O(log n) state jumping

### Data Point Processing
- u32 → fraction: `value / u32::MAX`
- Scale to range: `fraction * (max - min) + min`
- Round to 2 decimals: Creates uncertainty bands
- Multiple data points create constraint system

### Performance Requirements
- Brute force (2^64 states): Infeasible (~584 years at 1B checks/sec)
- Target: Complete search in < 10 seconds
- Need 10^12 to 10^15 speedup over brute force

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

### Phase 2: Mathematical Optimization Strategies

#### Strategy 1: Constraint Propagation
1. **Range Calculation**: For each data point, compute valid u32 ranges
2. **State Validation**: Given a candidate state, verify all constraints
3. **Early Termination**: Eliminate impossible states quickly

#### Strategy 2: Meet-in-the-Middle Attack
1. **Problem Splitting**: Split 64-bit state into two 32-bit halves
2. **Forward Search**: From potential "left half" states, generate expected outputs
3. **Backward Search**: From observed outputs, work backwards to "right half"
4. **Intersection**: Find matching pairs

#### Strategy 3: Differential Analysis
1. **Relative Offsets**: Use differences between data points
2. **State Relationships**: Leverage mathematical properties of LCG
3. **Constraint Tightening**: Each additional data point reduces search space exponentially

### Phase 3: Advanced Search Algorithms

#### Primary Algorithm: Hierarchical Search
1. **Coarse Grid**: Test every 2^32 states (reduces to 2^32 space)
2. **Fine Tuning**: For promising candidates, test nearby states
3. **Verification**: Confirm with all data points

#### Secondary Algorithm: Probabilistic Search
1. **Genetic Algorithm**: Evolve state candidates
2. **Fitness Function**: Minimize error across all data points
3. **Population Management**: Maintain diverse candidate pool

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

## Implementation Phases

### Phase 1: Minimum Viable Product (1-2 days)
- [ ] CLI interface and input parsing
- [ ] Basic brute force search (for small test cases)
- [ ] State validation function
- [ ] Simple test cases
- [ ] Initial time optimization modeling

### Phase 2: Core Algorithms (3-5 days)
- [ ] Constraint range calculation
- [ ] Hierarchical search implementation
- [ ] Meet-in-the-middle algorithm
- [ ] Performance benchmarking framework
- [ ] Execution time estimation based on data point count

### Phase 3: Optimization (3-4 days)
- [ ] SIMD optimizations
- [ ] Multi-threading
- [ ] Advanced search strategies
- [ ] Comprehensive testing
- [ ] Data point recommendation system

### Phase 4: Validation & Documentation (1-2 days)
- [ ] Edge case testing
- [ ] Performance documentation
- [ ] Usage examples
- [ ] Error handling
- [ ] Time optimization analysis and recommendations

## Success Criteria
- **Accuracy**: 95%+ success rate on test cases with 5+ data points
- **Performance**: Complete search in <10 seconds for typical cases
- **Time Optimization**: Provide accurate recommendations for optimal data point count
- **Total Time Minimization**: Minimize (collection time + execution time) for user
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

## Expected Time Optimization Curve
The algorithm should determine the optimal data point count by modeling:
- **Few data points (1-3)**: High execution time due to large search space
- **Medium data points (4-8)**: Rapidly decreasing execution time as constraints narrow search
- **Many data points (9+)**: Diminishing returns, execution time approaches minimum
- **Optimal range**: Likely 5-10 data points where `total_time = (n × 2) + execution_time(n)` is minimized

This plan balances mathematical rigor with practical time optimization, providing multiple fallback strategies and comprehensive testing to ensure the solution minimizes total user time while maintaining accuracy.