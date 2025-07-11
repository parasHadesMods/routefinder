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

### Performance Requirements (UPDATED WITH REAL DATA)
- **REALITY: Brute force (2^32 states): ~86 seconds actual runtime** (not 4.3s as estimated)
- **Current Performance**: ~50M seeds/second effective (4.3B / 86s)
- **Target**: Reduce to <30 seconds (need 3x speedup)