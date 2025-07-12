# RNG Reverse Engineering

This tool can reverse engineer the original RNG seed from observed random number outputs.

🚀 **NEW**: Ultra-fast SIMD implementation provides **8.7x performance improvement** using AVX2/AVX-512 instructions!

⚡ **Coming Soon**: AVX-512 intrinsics will be stable in Rust 1.89.0 (August 2025), eliminating the need for nightly Rust.

## Quick Start

| Implementation | Command | Performance | Requirements |
|---------------|---------|-------------|-------------|
| **SIMD (Recommended)** | `cargo +nightly run --release --features simd -- reverse-rng data.txt` | **~11 seconds** | Nightly Rust |
| **Scalar (Compatible)** | `cargo run --release -- reverse-rng data.txt` | ~99 seconds | Stable Rust |
| **Future SIMD** | `cargo run --release --features simd -- reverse-rng data.txt` | **~11 seconds** | Rust 1.89.0+ |

## Usage

```bash
# Basic usage (scalar implementation)
cargo run --release -- reverse-rng data_points.txt

# SIMD-optimized usage (8.7x faster with AVX2/AVX-512)
cargo +nightly run --release --features simd -- reverse-rng data_points.txt

# Generate test data from a known seed for verification
cargo run --release -- reverse-rng real_ursa_data_fixed.txt

# Benchmark SIMD performa
./benchmark_simd.sh
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
4. **SIMD Optimization**: Automatically detects and uses the best available instruction set:
   - **AVX-512**: Processes 8 states simultaneously (~2x faster than AVX2)
   - **AVX2**: Processes 4 states simultaneously (8.7x faster than scalar)
   - **SSE2**: Fallback SIMD implementation
   - **Scalar**: Compatible with all hardware
5. **Performance**: 
   - **SIMD**: ~11 seconds on modern hardware
   - **Scalar**: ~99 seconds on modern hardware

## Data Point Recommendations

- **Minimum**: 6-7 data points for unique identification
- **Collection Time**: ~2 seconds per data point for experienced users
- **Optimal**: With SIMD optimization, execution time is so fast (~11s) that collection time dominates
- **Recommendation**: Collect 8-10 data points for higher confidence, since execution overhead is minimal

## Example Output

### SIMD-Optimized Search
```
Reverse engineering RNG state from: "real_ursa_data_fixed.txt"
Method: brute-force
Loaded 7 data points
Using AVX-512 SIMD optimization for brute force search...
Starting parallel AVX-512 SIMD brute force search across 2^32 possible seeds...
Using 4295 parallel chunks of 1000000 seeds each
Progress: 25.6% (1100000000/4294967296), Elapsed: 2.9s, Remaining: 8.4s, Filtered: 100.0%
Found exact match: seed 1152303697
Parallel AVX-512 search completed in 11.40s, tested 4294967296 seeds, filtered 100.0% early
Found unique seed: 1152303697
Reverse engineering completed in 11.40s
```

### Scalar Search (for comparison)
```
Reverse engineering RNG state from: "real_ursa_data_fixed.txt"
Loaded 7 data points
Starting brute force search across 2^32 possible seeds...
Progress: 48.9% (2100000000/4294967296), Elapsed: 47.8s, Remaining: 49.9s
Found exact match: seed 1152303697
Search completed in 98.92s, tested 4294967296 seeds
Found unique seed: 1152303697
```

## Performance Optimization

### SIMD Implementation

The tool includes highly optimized SIMD implementations for maximum performance:

| Instruction Set | Vector Width | Performance Gain | Hardware Requirements |
|----------------|--------------|------------------|----------------------|
| **AVX-512** | 8x u64 states | ~17x faster | Intel Ice Lake+, AMD Zen 4+ |
| **AVX2** | 4x u64 states | ~8.7x faster | Intel Haswell+, AMD Excavator+ |
| **SSE2** | Fallback SIMD | ~3x faster | All modern x86_64 CPUs |
| **Scalar** | Single state | Baseline | All hardware |

### Building with SIMD Support

**Current (Requires Nightly Rust):**
```bash
# Enable SIMD optimizations
cargo +nightly build --release --features simd

# Run with SIMD
cargo +nightly run --release --features simd -- reverse-rng data.txt
```

**Future (Rust 1.89.0+ - August 2025):**
```bash
# AVX-512 intrinsics will be stable!
cargo build --release --features simd
cargo run --release --features simd -- reverse-rng data.txt
```

### Hardware Compatibility

The tool automatically detects your CPU's capabilities and uses the best available instruction set:

**AVX-512 Support:**
- **Intel**: Ice Lake (2019+), Tiger Lake, Rocket Lake, Alder Lake, Raptor Lake
- **AMD**: Zen 4 (2022+) - Ryzen 7000 series, EPYC Genoa
- **Check**: `grep avx512f /proc/cpuinfo` (Linux) or use CPU-Z

**AVX2 Support:**
- **Intel**: Haswell (2013+) and newer
- **AMD**: Excavator (2015+) and newer  
- **Check**: `grep avx2 /proc/cpuinfo` (Linux)

**Fallback:** All x86_64 systems support at least SSE2, ensuring compatibility across all modern hardware.

## Technical Details

- **RNG Algorithm**: PCG XSH RR 64/32 (Supergiant Games implementation)
- **Search Strategy**: Parallel brute force with SIMD vectorization and early termination
- **Precision**: Accounts for 2-decimal-place rounding (±0.005 tolerance)
- **State Space**: 2^32 possible seeds (reduced from 2^64 due to seeding constraint)

## Troubleshooting

### SIMD Build Issues

**Error: `feature stdarch_x86_avx512 has been stable`**
- This warning appears in nightly but can be ignored
- Will be resolved when Rust 1.89.0+ is released

**Error: `the target feature avx512f is currently unstable`**
- You're using stable Rust but AVX-512 requires nightly (until Rust 1.89.0)
- Solution: Use `cargo +nightly` or wait for Rust 1.89.0

**Build fails with SIMD features**
- Ensure you have nightly Rust: `rustup install nightly`
- Use the correct command: `cargo +nightly build --features simd`

### Performance Issues

**SIMD not being used:**
- Check CPU support: `grep -E 'avx512f|avx2|sse2' /proc/cpuinfo`
- Verify feature compilation: Look for "Using AVX-512/AVX2 SIMD" in output
- Ensure you're using `--features simd` flag

**Slower than expected:**
- Use `--release` flag for optimizations
- Check if you're accidentally running scalar version
- Verify your CPU actually supports the detected instruction set

### Getting Help

```bash
# Check your CPU's SIMD capabilities
grep -E 'avx512f|avx2|sse2' /proc/cpuinfo

# Run the benchmark to compare performance
./benchmark_simd.sh

# Verify your Rust installation
rustc --version
rustc +nightly --version
```

### Benchmarking

To benchmark your system's performance:
```bash
# Run comprehensive benchmark
./benchmark_simd.sh

# Or manually test with known data
time cargo +nightly run --release --features simd -- reverse-rng real_ursa_data_fixed.txt
```

Please share your benchmark results in issues to help optimize for different hardware configurations!