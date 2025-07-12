#!/bin/bash

# Benchmark script to compare SIMD vs scalar reverse RNG performance

echo "Building optimized releases..."
cargo build +nightly --release --features simd

echo ""
echo "=== Benchmarking SIMD Implementation ==="  
time cargo +nightly run --release --features simd -- reverse-rng real_ursa_data_fixed.txt

echo ""
echo "Benchmark completed. Compare the execution times above."