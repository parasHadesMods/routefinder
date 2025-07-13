#!/bin/bash

# Benchmark script for reverse_rng optimization tracking
# Uses test/real_rng_pre_styx.txt as test data (expected seed: 1152303697)

echo "=== Reverse RNG Benchmark ==="
echo "Data file: test/real_rng_pre_styx.txt"
echo "Expected seed: 1152303697"
echo "Starting benchmark..."
echo

# Record start time
start_time=$(date +%s.%N)

# Run the reverse_rng command
cargo run --release -- reverse-rng test/real_rng_pre_styx.txt

# Record end time
end_time=$(date +%s.%N)

# Calculate duration
duration=$(echo "$end_time - $start_time" | bc -l)

echo
echo "=== Benchmark Results ==="
echo "Duration: ${duration} seconds"
echo "Date: $(date)"
echo "========================="