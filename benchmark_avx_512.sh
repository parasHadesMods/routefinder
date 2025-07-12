#!/bin/bash

# Benchmark script to compare SIMD vs scalar reverse RNG performance

echo "Building optimized releases..."
cargo +nightly build --release --features simd_nightly

echo ""
echo "=== Benchmarking AVX-512 SIMD Implementation ==="
time cargo +nightly run --release --features simd_nightly -- reverse-rng real_ursa_data_fixed.txt

echo ""
echo "Benchmark completed. Compare the execution times above."