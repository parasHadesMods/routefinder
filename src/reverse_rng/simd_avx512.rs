use crate::error::Error;
use crate::reverse_rng::data_point::{DataPoint, StateCandidate};
use crate::rng::SggPcg;
use std::time::Instant;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use std::arch::x86_64::*;

pub const CHUNK_SIZE_AVX512: usize = 8; // Process 8 u64 states at once with AVX-512

// SIMD constants for PCG
const MULTIPLIER: u64 = 0x5851f42d4c957f2d;
const INCREMENT: u64 = 0xb47c73972972b7b7;
const INITIAL_OFFSET: u64 = 0x3d657cc62bc341e;

// PCG output function constants
const ROTATE: u32 = 59;
const XSHIFT: u32 = 18;
const SPARE: u32 = 27;

#[target_feature(enable = "avx512f")]
pub unsafe fn find_original_state_avx512(data_points: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
    println!("Starting parallel AVX-512 SIMD brute force search across 2^32 possible seeds...");
    
    let start_time = Instant::now();
    let total_seeds = 1u64 << 32;
    
    // Shared atomic counters for progress tracking
    let tested_count = Arc::new(AtomicU64::new(0));
    
    // Determine chunk size for parallel processing
    // Use larger chunks to reduce overhead while maintaining good load balancing
    let parallel_chunk_size = 1_000_000u64; // Process 1M seeds per parallel chunk
    let num_chunks = (total_seeds + parallel_chunk_size - 1) / parallel_chunk_size;
    
    println!("Using {} parallel chunks of {} seeds each", num_chunks, parallel_chunk_size);
    
    // Create parallel iterator over seed ranges
    let candidates: Vec<StateCandidate> = (0..num_chunks)
        .into_par_iter()
        .flat_map(|chunk_id| {
            let start_seed = chunk_id * parallel_chunk_size;
            let end_seed = std::cmp::min((chunk_id + 1) * parallel_chunk_size, total_seeds);
            
            let mut local_candidates = Vec::new();
            let mut local_tested = 0u64;
            
            // Process this chunk with SIMD
            let mut seed_base = (start_seed as i64) + (i32::MIN as i64);
            let chunk_end = (end_seed as i64) + (i32::MIN as i64);
            
            while seed_base < chunk_end {
                local_tested += CHUNK_SIZE_AVX512 as u64;
                
                // Load 8 consecutive seeds
                let seeds = [
                    seed_base as i32,
                    seed_base as i32 + 1,
                    seed_base as i32 + 2,
                    seed_base as i32 + 3,
                    seed_base as i32 + 4,
                    seed_base as i32 + 5,
                    seed_base as i32 + 6,
                    seed_base as i32 + 7,
                ];
                
                // Convert to u64 and compute initial PCG states using SIMD
                let states = compute_initial_states_simd_avx512(&seeds);
                
                // Use SIMD to validate all 8 seeds at once
                let valid_seeds = validate_seeds_simd_avx512(&seeds, &states, data_points);
                
                for i in 0..CHUNK_SIZE_AVX512 {
                    if valid_seeds[i] {
                        let state = SggPcg::new(seeds[i] as u64).state();
                        
                        local_candidates.push(StateCandidate {
                            seed: seeds[i],
                            state,
                        });
                        
                        println!("Found match: seed {}", seeds[i]);
                    }
                }
                
                seed_base += CHUNK_SIZE_AVX512 as i64;
                
                // Prevent going beyond the chunk boundary
                if seed_base >= chunk_end {
                    break;
                }
            }
            
            // Update global counters
            tested_count.fetch_add(local_tested, Ordering::Relaxed);
            
            local_candidates
        })
        .collect();
    
    let elapsed = start_time.elapsed();
    let final_tested = tested_count.load(Ordering::Relaxed);
    
    println!("Parallel AVX-512 search completed in {:.2}s, tested {} seeds", 
             elapsed.as_secs_f64(), final_tested);
    
    // Report results
    match candidates.len() {
        0 => println!("No valid seeds found that match all data points exactly."),
        1 => println!("Found unique seed: {}", candidates[0].seed),
        n => {
            println!("WARNING: Found {} matching seeds - you need more data points to uniquely identify the seed:", n);
            for candidate in &candidates {
                println!("  Seed: {}", candidate.seed);
            }
        }
    }
    
    Ok(candidates)
}

#[target_feature(enable = "avx512f")]
unsafe fn compute_initial_states_simd_avx512(seeds: &[i32; CHUNK_SIZE_AVX512]) -> [u64; CHUNK_SIZE_AVX512] {
    // Load seeds into AVX-512 register (8x i32 -> 8x u64)
    let seeds_256 = _mm256_loadu_si256(seeds.as_ptr() as *const __m256i);
    let seeds_512 = _mm512_cvtepi32_epi64(seeds_256);
    
    // Load constants
    let multiplier = _mm512_set1_epi64(MULTIPLIER as i64);
    let initial_offset = _mm512_set1_epi64(INITIAL_OFFSET as i64);
    
    // Compute seeds * MULTIPLIER - INITIAL_OFFSET using built-in 64-bit multiply
    let product = _mm512_mullox_epi64(seeds_512, multiplier);
    let states = _mm512_sub_epi64(product, initial_offset);
    
    // Store result
    let mut result = [0u64; CHUNK_SIZE_AVX512];
    _mm512_storeu_si512(result.as_mut_ptr() as *mut __m512i, states);
    result
}

#[target_feature(enable = "avx512f")]
unsafe fn validate_seeds_simd_avx512(_seeds: &[i32; CHUNK_SIZE_AVX512], initial_states: &[u64; CHUNK_SIZE_AVX512], data_points: &[DataPoint]) -> [bool; CHUNK_SIZE_AVX512] {
    // Use i64 format where non-zero represents true, 0 represents false
    let mut results = [1i64; CHUNK_SIZE_AVX512]; // Start with all true (non-zero)
    
    // Load constants for SIMD operations
    let multiplier = _mm512_set1_epi64(MULTIPLIER as i64);
    let increment = _mm512_set1_epi64(INCREMENT as i64);
    
    // Process each data point
    for data_point in data_points {
        // Load initial states
        let mut states = _mm512_loadu_si512(initial_states.as_ptr() as *const __m512i);
        
        // Advance states if needed
        if data_point.offset > 0 {
            states = advance_pcg_states_simd_avx512(states, data_point.offset - 1);
        }
        
        // Generate next values using PCG
        let old_states = states;
        states = _mm512_add_epi64(_mm512_mullox_epi64(states, multiplier), increment);
        
        // Apply PCG output function and check consistency using SIMD
        let generated_values = pcg_output_function_simd_avx512(old_states);
        check_consistency_simd_avx512(&generated_values, data_point, &mut results);
        
        // Early termination if all seeds failed (vectorized check)
        let results_vector = _mm512_loadu_si512(results.as_ptr() as *const __m512i);
        if _mm512_test_epi64_mask(results_vector, results_vector) == 0 {
            break; // All values are zero, terminate early
        }
    }
    
    // Convert i64 results back to bool (non-zero means true)
    [
        results[0] != 0,
        results[1] != 0,
        results[2] != 0,
        results[3] != 0,
        results[4] != 0,
        results[5] != 0,
        results[6] != 0,
        results[7] != 0,
    ]
}

#[target_feature(enable = "avx512f")]
unsafe fn advance_pcg_states_simd_avx512(states: __m512i, delta: u64) -> __m512i {
    // Initialize accumulator vectors
    let mut acc_mult = _mm512_set1_epi64(1);
    let mut acc_plus = _mm512_set1_epi64(0);
    let mut cur_mult = _mm512_set1_epi64(MULTIPLIER as i64);
    let mut cur_plus = _mm512_set1_epi64(INCREMENT as i64);
    let mut mdelta = delta;

    while mdelta > 0 {
        if (mdelta & 1) != 0 {
            // acc_mult = acc_mult * cur_mult
            acc_mult = _mm512_mullox_epi64(acc_mult, cur_mult);
            // acc_plus = acc_plus * cur_mult + cur_plus
            acc_plus = _mm512_add_epi64(_mm512_mullox_epi64(acc_plus, cur_mult), cur_plus);
        }
        // cur_plus = (cur_mult + 1) * cur_plus
        let cur_mult_plus_one = _mm512_add_epi64(cur_mult, _mm512_set1_epi64(1));
        cur_plus = _mm512_mullox_epi64(cur_mult_plus_one, cur_plus);
        // cur_mult = cur_mult * cur_mult
        cur_mult = _mm512_mullox_epi64(cur_mult, cur_mult);
        mdelta /= 2;
    }
    
    // Final result: acc_mult * states + acc_plus
    _mm512_add_epi64(_mm512_mullox_epi64(acc_mult, states), acc_plus)
}

#[target_feature(enable = "avx512f")]
unsafe fn check_consistency_simd_avx512(values: &[u32; CHUNK_SIZE_AVX512], data_point: &DataPoint, results: &mut [i64; CHUNK_SIZE_AVX512]) {
    let (min_u32, max_u32) = data_point.valid_u32_range();
    
    // Load values into SIMD register
    let values_256 = _mm256_loadu_si256(values.as_ptr() as *const __m256i);
    let values_512 = _mm512_cvtepu32_epi64(values_256);
    
    // Load min/max bounds into SIMD registers
    let min_bounds = _mm512_set1_epi64(min_u32 as i64);
    let max_bounds = _mm512_set1_epi64(max_u32 as i64);
    
    // Perform SIMD comparisons: values >= min_u32 && values <= max_u32
    let ge_min = _mm512_cmpge_epi64_mask(values_512, min_bounds);
    let le_max = _mm512_cmple_epi64_mask(values_512, max_bounds);
    
    // Combine conditions with AND
    let valid_mask = ge_min & le_max;
    
    // Load current results and apply mask
    let current_results = _mm512_loadu_si512(results.as_ptr() as *const __m512i);
    let zero_vec = _mm512_setzero_si512();
    let updated_results = _mm512_mask_blend_epi64(valid_mask, zero_vec, current_results);
    
    // Store back the updated results
    _mm512_storeu_si512(results.as_mut_ptr() as *mut __m512i, updated_results);
}

#[target_feature(enable = "avx512f")]
unsafe fn pcg_output_function_simd_avx512(states: __m512i) -> [u32; CHUNK_SIZE_AVX512] {
    // Extract rotation amounts: (state >> 59) & 0x1F (only need bottom 5 bits)
    let rotate_shifts = _mm512_srli_epi64(states, ROTATE);
    let rotate_mask = _mm512_set1_epi64(0x1F);
    let rotations = _mm512_and_epi64(rotate_shifts, rotate_mask);
    
    // Compute xorshift: ((state >> 18) ^ state) >> 27
    let shifted = _mm512_srli_epi64(states, XSHIFT);
    let xored = _mm512_xor_epi64(shifted, states);
    let xsh_values = _mm512_srli_epi64(xored, SPARE);
    
    // Convert to 32-bit values and apply variable rotations
    let mut result = [0u32; CHUNK_SIZE_AVX512];
    let mut xsh_array = [0u64; CHUNK_SIZE_AVX512];
    let mut rot_array = [0u64; CHUNK_SIZE_AVX512];
    
    _mm512_storeu_si512(xsh_array.as_mut_ptr() as *mut __m512i, xsh_values);
    _mm512_storeu_si512(rot_array.as_mut_ptr() as *mut __m512i, rotations);
    
    for i in 0..CHUNK_SIZE_AVX512 {
        let xsh = xsh_array[i] as u32;
        let rot = rot_array[i] as u32;
        result[i] = xsh.rotate_right(rot);
    }
    
    result
}

// Shared utility functions for scalar fallback and testing
pub fn pcg_output_function(state: u64) -> u32 {
    let rot = (state >> ROTATE) as u32;
    let xsh = (((state >> XSHIFT) ^ state) >> SPARE) as u32;
    xsh.rotate_right(rot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reverse_rng::data_point::DataPoint;
    
    #[test]
    fn test_avx512_early_termination_check() {
        // Test the vectorized early termination logic for AVX-512
        unsafe {
            if !is_x86_feature_detected!("avx512f") {
                return; // Skip test if AVX-512 not available
            }
            
            // Test case 1: All zeros (should terminate)
            let all_zeros = [0i64; CHUNK_SIZE_AVX512];
            let results_vector = _mm512_loadu_si512(all_zeros.as_ptr() as *const __m512i);
            assert_eq!(_mm512_test_epi64_mask(results_vector, results_vector), 0, "All zeros should trigger termination");
            
            // Test case 2: Some non-zeros (should not terminate) 
            let mixed = [0i64, 1i64, 0i64, 0i64, 0i64, 0i64, 0i64, 0i64];
            let results_vector = _mm512_loadu_si512(mixed.as_ptr() as *const __m512i);
            assert_ne!(_mm512_test_epi64_mask(results_vector, results_vector), 0, "Mixed values should not trigger termination");
            
            // Test case 3: All non-zeros (should not terminate)
            let all_ones = [1i64; CHUNK_SIZE_AVX512];
            let results_vector = _mm512_loadu_si512(all_ones.as_ptr() as *const __m512i);
            assert_ne!(_mm512_test_epi64_mask(results_vector, results_vector), 0, "All non-zeros should not trigger termination");
        }
    }
    
    #[test]
    fn test_avx512_pcg_output_function() {
        // Test the AVX-512 PCG output function directly against scalar implementation
        if !is_x86_feature_detected!("avx512f") {
            return; // Skip test if AVX-512 not available
        }
        
        let test_states = [
            0x123456789ABCDEF0u64,
            0xFEDCBA9876543210u64,
            0x0123456789ABCDEFu64,
            0xDEADBEEFCAFEBABEu64,
            0x1111111111111111u64,
            0x2222222222222222u64,
            0x3333333333333333u64,
            0x4444444444444444u64,
        ];
        
        // Compute expected results using scalar function
        let expected: [u32; CHUNK_SIZE_AVX512] = [
            pcg_output_function(test_states[0]),
            pcg_output_function(test_states[1]),
            pcg_output_function(test_states[2]),
            pcg_output_function(test_states[3]),
            pcg_output_function(test_states[4]),
            pcg_output_function(test_states[5]),
            pcg_output_function(test_states[6]),
            pcg_output_function(test_states[7]),
        ];
        
        // Compute using SIMD function
        unsafe {
            let states_simd = _mm512_loadu_si512(test_states.as_ptr() as *const __m512i);
            let result = pcg_output_function_simd_avx512(states_simd);
            
            assert_eq!(result, expected, "AVX-512 PCG output function should match scalar implementation");
        }
    }
    
    #[test]
    fn test_avx512_consistency_check() {
        if !is_x86_feature_detected!("avx512f") {
            return; // Skip test if AVX-512 not available
        }
        
        // Create a test data point
        let data_point = DataPoint {
            name: "test".to_string(),
            offset: 1,
            range_min: 0.0,
            range_max: 100.0,
            observed: 50.0,
        };
        
        let (min_u32, max_u32) = data_point.valid_u32_range();
        
        // Test values: some inside range, some outside
        let test_values = [
            min_u32,        // Should be valid (at min boundary)
            max_u32,        // Should be valid (at max boundary)
            min_u32 - 1,    // Should be invalid (below min)
            max_u32 + 1,    // Should be invalid (above max)
            (min_u32 + max_u32) / 2, // Should be valid (middle)
            0,              // Should be invalid (below min)
            u32::MAX,       // Should be invalid (above max)
            min_u32 + 1,    // Should be valid (just above min)
        ];
        
        // Compute expected results using scalar function
        let expected = [
            data_point.is_consistent_with(test_values[0]),
            data_point.is_consistent_with(test_values[1]),
            data_point.is_consistent_with(test_values[2]),
            data_point.is_consistent_with(test_values[3]),
            data_point.is_consistent_with(test_values[4]),
            data_point.is_consistent_with(test_values[5]),
            data_point.is_consistent_with(test_values[6]),
            data_point.is_consistent_with(test_values[7]),
        ];
        
        // Compute using SIMD function
        unsafe {
            let mut result = [1i64; CHUNK_SIZE_AVX512]; // Start with all true (non-zero)
            check_consistency_simd_avx512(&test_values, &data_point, &mut result);
            
            // Convert i64 results to bool for comparison
            let result_bool = [
                result[0] != 0,
                result[1] != 0,
                result[2] != 0,
                result[3] != 0,
                result[4] != 0,
                result[5] != 0,
                result[6] != 0,
                result[7] != 0,
            ];
            assert_eq!(result_bool, expected, "AVX-512 consistency check should match scalar implementation");
        }
    }
}