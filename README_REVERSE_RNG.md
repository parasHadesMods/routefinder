# RNG Reverse Engineering

This tool can reverse engineer the original RNG seed from observed random number outputs.

## Usage

```bash
# Basic usage
cargo run --bin routefinder -- reverse-rng data_points.txt

# Generate test data from a known seed for verification
cargo run --bin generate_test_data > test_data.txt
cargo run --bin routefinder -- reverse-rng test_data.txt
```

## Input Format

The input file should contain data points in CSV format:

```
# Comments start with #
# Format: name,offset,min,max,observed
chamber1,0,0.0,100.0,52.42
chamber2,5,0.0,1.0,0.59
chamber3,10,-50.0,50.0,42.49
```

Where:
- `name`: Descriptive name for the data point
- `offset`: Number of RNG advances from the original state
- `min,max`: Range used for scaling the random value
- `observed`: The rounded result (2 decimal places) that was observed

## How It Works

1. **Search Space**: Tests all possible i32 seed values (2^32 = ~4.3 billion)
2. **Validation**: For each seed, generates RNG values at specified offsets and checks consistency
3. **Early Termination**: Stops when a high-confidence match is found
4. **Performance**: Typically completes in under 2 minutes on modern hardware

## Data Point Recommendations

- **Minimum**: 6-7 data points for unique identification
- **Collection Time**: ~2 seconds per data point for experienced users
- **Optimal**: Balance collection time with execution time

The tool provides time optimization analysis to help determine the optimal number of data points to collect.

## Example Output

```
Reverse engineering RNG state from: "test_data.txt"
Loaded 7 data points
Time optimization analysis:
  Data points: 7
  Estimated execution time: 0.10s
  Total time (collection + execution): 14.10s
Starting brute force search across 2^32 possible seeds...
Progress: 48.9% (2100000000/4294967296), Elapsed: 92.7s, Remaining: 96.9s
Found candidate: seed 12345, confidence: 1.0000
Found high-confidence match, stopping search early
Search completed in 94.97s, tested 2147495994 seeds
Found 1 candidate state(s):
  Candidate 1: seed 12345 (confidence: 1.00)
```

## Technical Details

- **RNG Algorithm**: PCG XSH RR 64/32 (Supergiant Games implementation)
- **Search Strategy**: Brute force with early termination
- **Precision**: Accounts for 2-decimal-place rounding (±0.005 tolerance)
- **State Space**: 2^32 possible seeds (reduced from 2^64 due to seeding constraint)