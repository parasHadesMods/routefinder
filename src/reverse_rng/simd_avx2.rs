use crate::error::Error;
use crate::reverse_rng::data_point::{DataPoint, StateCandidate};
use crate::rng::SggPcg;
use std::time::Instant;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use std::arch::x86_64::*;

pub const CHUNK_SIZE_AVX2: usize = 4; // Process 4 u64 states at once with AVX2

// SIMD constants for PCG
const MULTIPLIER: u64 = 0x5851f42d4c957f2d;
const INCREMENT: u64 = 0xb47c73972972b7b7;
const INITIAL_OFFSET: u64 = 0x3d657cc62bc341e;

// PCG output function constants
const ROTATE: u32 = 59;
const XSHIFT: u32 = 18;
const SPARE: u32 = 27;

#[target_feature(enable = "avx2")]
pub unsafe fn find_original_state_avx2(data_points: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
    println!("Starting parallel AVX2 SIMD brute force search across 2^32 possible seeds...");
    
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
                local_tested += CHUNK_SIZE_AVX2 as u64;
                
                // Load 4 consecutive seeds
                let seeds = [
                    seed_base as i32,
                    seed_base as i32 + 1,
                    seed_base as i32 + 2,
                    seed_base as i32 + 3,
                ];
                
                // Convert to u64 and compute initial PCG states using SIMD
                let states = compute_initial_states_simd(&seeds);
                
                // Use SIMD to validate all 4 seeds at once
                let valid_seeds = validate_seeds_simd_avx2(&seeds, &states, data_points);
                
                for i in 0..CHUNK_SIZE_AVX2 {
                    if valid_seeds[i] {
                        let state = SggPcg::new(seeds[i] as u64).state();
                        
                        local_candidates.push(StateCandidate {
                            seed: seeds[i],
                            state,
                        });
                        
                        println!("Found match: seed {}", seeds[i]);
                    }
                }
                
                seed_base += CHUNK_SIZE_AVX2 as i64;
                
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

    println!("Parallel AVX2 search completed in {:.2}s, tested {} seeds", 
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

#[target_feature(enable = "avx2")]
unsafe fn mul_epi64_avx2(a: __m256i, b: __m256i) -> __m256i {
    // 64-bit multiplication using 32-bit operations
    // Split into 32-bit components: a = a_lo + (a_hi << 32), b = b_lo + (b_hi << 32)
    // Result = a_lo * b_lo + (a_lo * b_hi + a_hi * b_lo) << 32
    
    // Extract low and high 32-bit parts
    let a_lo = _mm256_and_si256(a, _mm256_set1_epi64x(0xFFFFFFFF));
    let a_hi = _mm256_srli_epi64(a, 32);
    let b_lo = _mm256_and_si256(b, _mm256_set1_epi64x(0xFFFFFFFF));
    let b_hi = _mm256_srli_epi64(b, 32);
    
    // Compute partial products
    let lo_lo = _mm256_mul_epu32(a_lo, b_lo);
    let lo_hi = _mm256_mul_epu32(a_lo, b_hi);
    let hi_lo = _mm256_mul_epu32(a_hi, b_lo);
    
    // Combine: lo_lo + (lo_hi + hi_lo) << 32
    let mid = _mm256_add_epi64(lo_hi, hi_lo);
    let mid_shifted = _mm256_slli_epi64(mid, 32);
    _mm256_add_epi64(lo_lo, mid_shifted)
}

#[target_feature(enable = "avx2")]
unsafe fn compute_initial_states_simd(seeds: &[i32; CHUNK_SIZE_AVX2]) -> [u64; CHUNK_SIZE_AVX2] {
    // Load seeds into AVX2 register (4x i32 -> 4x u64)
    let seeds_128 = _mm_loadu_si128(seeds.as_ptr() as *const __m128i);
    let seeds_256 = _mm256_cvtepi32_epi64(seeds_128);
    
    // Load constants
    let multiplier = _mm256_set1_epi64x(MULTIPLIER as i64);
    let initial_offset = _mm256_set1_epi64x(INITIAL_OFFSET as i64);
    
    // Compute seeds * MULTIPLIER - INITIAL_OFFSET using custom 64-bit multiply
    let product = mul_epi64_avx2(seeds_256, multiplier);
    let states = _mm256_sub_epi64(product, initial_offset);
    
    // Store result
    let mut result = [0u64; CHUNK_SIZE_AVX2];
    _mm256_storeu_si256(result.as_mut_ptr() as *mut __m256i, states);
    result
}

#[target_feature(enable = "avx2")]
unsafe fn validate_seeds_simd_avx2(_seeds: &[i32; CHUNK_SIZE_AVX2], initial_states: &[u64; CHUNK_SIZE_AVX2], data_points: &[DataPoint]) -> [bool; CHUNK_SIZE_AVX2] {
    // Use i64 format where non-zero represents true, 0 represents false
    let mut results = [1i64; CHUNK_SIZE_AVX2]; // Start with all true (non-zero)
    
    // Load constants for SIMD operations
    let multiplier = _mm256_set1_epi64x(MULTIPLIER as i64);
    let increment = _mm256_set1_epi64x(INCREMENT as i64);
    
    // Process each data point
    for data_point in data_points {
        // Load initial states
        let mut states = _mm256_loadu_si256(initial_states.as_ptr() as *const __m256i);
        
        // Advance states if needed
        if data_point.offset > 0 {
            states = advance_pcg_states_simd(states, data_point.offset - 1);
        }
        
        // Generate next values using PCG
        let old_states = states;
        states = _mm256_add_epi64(mul_epi64_avx2(states, multiplier), increment);
        
        // Apply PCG output function and check consistency using SIMD
        let generated_values = pcg_output_function_simd(old_states);
        check_consistency_simd(&generated_values, data_point, &mut results);
        
        // Early termination if all seeds failed (vectorized check)
        let results_vector = _mm256_loadu_si256(results.as_ptr() as *const __m256i);
        if _mm256_testz_si256(results_vector, results_vector) != 0 {
            break; // All values are zero, terminate early
        }
    }
    
    // Convert i64 results back to bool (non-zero means true)
    [
        results[0] != 0,
        results[1] != 0,
        results[2] != 0,
        results[3] != 0,
    ]
}

#[target_feature(enable = "avx2")]
unsafe fn advance_pcg_states_simd(states: __m256i, delta: u64) -> __m256i {
    // Initialize accumulator vectors
    let mut acc_mult = _mm256_set1_epi64x(1);
    let mut acc_plus = _mm256_set1_epi64x(0);
    let mut cur_mult = _mm256_set1_epi64x(MULTIPLIER as i64);
    let mut cur_plus = _mm256_set1_epi64x(INCREMENT as i64);
    let mut mdelta = delta;

    while mdelta > 0 {
        if (mdelta & 1) != 0 {
            // acc_mult = acc_mult * cur_mult
            acc_mult = mul_epi64_avx2(acc_mult, cur_mult);
            // acc_plus = acc_plus * cur_mult + cur_plus
            acc_plus = _mm256_add_epi64(mul_epi64_avx2(acc_plus, cur_mult), cur_plus);
        }
        // cur_plus = (cur_mult + 1) * cur_plus
        let cur_mult_plus_one = _mm256_add_epi64(cur_mult, _mm256_set1_epi64x(1));
        cur_plus = mul_epi64_avx2(cur_mult_plus_one, cur_plus);
        // cur_mult = cur_mult * cur_mult
        cur_mult = mul_epi64_avx2(cur_mult, cur_mult);
        mdelta /= 2;
    }
    
    // Final result: acc_mult * states + acc_plus
    _mm256_add_epi64(mul_epi64_avx2(acc_mult, states), acc_plus)
}

#[target_feature(enable = "avx2")]
unsafe fn check_consistency_simd(values: &[u32; CHUNK_SIZE_AVX2], data_point: &DataPoint, results: &mut [i64; CHUNK_SIZE_AVX2]) {
    let min_u32 = data_point.min_u32;
    let max_u32 = data_point.max_u32;
    
    // Load values into SIMD register
    let values_128 = _mm_loadu_si128(values.as_ptr() as *const __m128i);
    let values_256 = _mm256_cvtepu32_epi64(values_128);
    
    // Load min/max bounds into SIMD registers
    let min_bounds = _mm256_set1_epi64x(min_u32 as i64);
    let max_bounds = _mm256_set1_epi64x(max_u32 as i64);
    
    // Perform SIMD comparisons: values >= min_u32 && values <= max_u32
    let ge_min = _mm256_cmpgt_epi64(values_256, _mm256_sub_epi64(min_bounds, _mm256_set1_epi64x(1)));
    let le_max = _mm256_cmpgt_epi64(_mm256_add_epi64(max_bounds, _mm256_set1_epi64x(1)), values_256);
    
    // Combine conditions with AND
    let valid = _mm256_and_si256(ge_min, le_max);
    
    // Load current results and perform vectorized AND with validity mask
    let current_results = _mm256_loadu_si256(results.as_ptr() as *const __m256i);
    let updated_results = _mm256_and_si256(current_results, valid);
    
    // Store back the updated results
    _mm256_storeu_si256(results.as_mut_ptr() as *mut __m256i, updated_results);
}

#[target_feature(enable = "avx2")]
unsafe fn pcg_output_function_simd(states: __m256i) -> [u32; CHUNK_SIZE_AVX2] {
    // Extract rotation amounts: (state >> 59) & 0x1F (only need bottom 5 bits)
    let rotate_shifts = _mm256_srli_epi64(states, ROTATE as i32);
    let rotate_mask = _mm256_set1_epi64x(0x1F);
    let rotations = _mm256_and_si256(rotate_shifts, rotate_mask);
    
    // Compute xorshift: ((state >> 18) ^ state) >> 27
    let shifted = _mm256_srli_epi64(states, XSHIFT as i32);
    let xored = _mm256_xor_si256(shifted, states);
    let xsh_values = _mm256_srli_epi64(xored, SPARE as i32);
    
    // Convert to 32-bit values and apply variable rotations
    let mut result = [0u32; CHUNK_SIZE_AVX2];
    let mut xsh_array = [0u64; CHUNK_SIZE_AVX2];
    let mut rot_array = [0u64; CHUNK_SIZE_AVX2];
    
    _mm256_storeu_si256(xsh_array.as_mut_ptr() as *mut __m256i, xsh_values);
    _mm256_storeu_si256(rot_array.as_mut_ptr() as *mut __m256i, rotations);
    
    for i in 0..CHUNK_SIZE_AVX2 {
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
    fn test_early_termination_check_avx2() {
        // Test the vectorized early termination logic
        unsafe {
            // Test case 1: All zeros (should terminate)
            let all_zeros = [0i64; CHUNK_SIZE_AVX2];
            let results_vector = _mm256_loadu_si256(all_zeros.as_ptr() as *const __m256i);
            assert_ne!(_mm256_testz_si256(results_vector, results_vector), 0, "All zeros should trigger termination");
            
            // Test case 2: Some non-zeros (should not terminate) 
            let mixed = [0i64, 1i64, 0i64, 0i64];
            let results_vector = _mm256_loadu_si256(mixed.as_ptr() as *const __m256i);
            assert_eq!(_mm256_testz_si256(results_vector, results_vector), 0, "Mixed values should not trigger termination");
            
            // Test case 3: All non-zeros (should not terminate)
            let all_ones = [1i64; CHUNK_SIZE_AVX2];
            let results_vector = _mm256_loadu_si256(all_ones.as_ptr() as *const __m256i);
            assert_eq!(_mm256_testz_si256(results_vector, results_vector), 0, "All non-zeros should not trigger termination");
        }
    }

    #[test]
    fn test_consistency_check_simd_avx2() {
        // Create a test data point
        let data_point = DataPoint {
            name: "test".to_string(),
            offset: 1,
            range_min: 0.0,
            range_max: 100.0,
            observed: 50.0,
        };
        
        let min_u32 = data_point.min_u32;
    let max_u32 = data_point.max_u32;
        
        // Test values: some inside range, some outside
        let test_values = [
            min_u32,        // Should be valid (at min boundary)
            max_u32,        // Should be valid (at max boundary)
            min_u32 - 1,    // Should be invalid (below min)
            max_u32 + 1,    // Should be invalid (above max)
        ];
        
        // Compute expected results using scalar function
        let expected = [
            data_point.is_consistent_with(test_values[0]),
            data_point.is_consistent_with(test_values[1]),
            data_point.is_consistent_with(test_values[2]),
            data_point.is_consistent_with(test_values[3]),
        ];
        
        // Compute using SIMD function
        unsafe {
            let mut result = [1i64; CHUNK_SIZE_AVX2]; // Start with all true (non-zero)
            check_consistency_simd(&test_values, &data_point, &mut result);
            
            // Convert i64 results to bool for comparison
            let result_bool = [
                result[0] != 0,
                result[1] != 0,
                result[2] != 0,
                result[3] != 0,
            ];
            assert_eq!(result_bool, expected, "SIMD consistency check should match scalar implementation");
        }
    }

    #[test]
    fn test_pcg_output_function_simd_avx2() {
        // Test the SIMD PCG output function directly against scalar implementation
        let test_states = [
            0x123456789ABCDEF0u64,
            0xFEDCBA9876543210u64,
            0x0123456789ABCDEFu64,
            0xDEADBEEFCAFEBABEu64,
        ];
        
        // Compute expected results using scalar function
        let expected: [u32; CHUNK_SIZE_AVX2] = [
            pcg_output_function(test_states[0]),
            pcg_output_function(test_states[1]),
            pcg_output_function(test_states[2]),
            pcg_output_function(test_states[3]),
        ];
        
        // Compute using SIMD function
        unsafe {
            let states_simd = _mm256_loadu_si256(test_states.as_ptr() as *const __m256i);
            let result = pcg_output_function_simd(states_simd);
            
            assert_eq!(result, expected, "SIMD PCG output function should match scalar implementation");
        }
    }
}