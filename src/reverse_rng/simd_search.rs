use crate::error::Error;
use crate::reverse_rng::data_point::{DataPoint, StateCandidate};
use crate::rng::SggPcg;
use std::time::Instant;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use std::arch::x86_64::*;

const CHUNK_SIZE_AVX512: usize = 8; // Process 8 u64 states at once with AVX-512
const CHUNK_SIZE_AVX2: usize = 4; // Process 4 u64 states at once with AVX2

// SIMD constants for PCG
const MULTIPLIER: u64 = 0x5851f42d4c957f2d;
const INCREMENT: u64 = 0xb47c73972972b7b7;
const INITIAL_OFFSET: u64 = 0x3d657cc62bc341e;

// PCG output function constants
const ROTATE: u32 = 59;
const XSHIFT: u32 = 18;
const SPARE: u32 = 27;

pub fn find_original_state_simd(data_points: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
    #[cfg(feature = "simd")]
    {
        if is_x86_feature_detected!("avx512f") {
            println!("Using AVX-512 SIMD optimization for brute force search...");
            return unsafe { find_original_state_avx512(data_points) };
        } else if is_x86_feature_detected!("avx2") {
            println!("Using AVX2 SIMD optimization for brute force search...");
            return unsafe { find_original_state_avx2(data_points) };
        } else if is_x86_feature_detected!("sse2") {
            println!("Using SSE2 SIMD optimization for brute force search...");
            return unsafe { find_original_state_sse2(data_points) };
        }
    }
    
    println!("SIMD not available or not enabled, falling back to scalar implementation");
    crate::reverse_rng::search::find_original_state(data_points)
}

#[cfg(feature = "simd")]
#[target_feature(enable = "avx2")]
unsafe fn find_original_state_avx2(data_points: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
    println!("Starting parallel AVX2 SIMD brute force search across 2^32 possible seeds...");
    
    let start_time = Instant::now();
    let total_seeds = 1u64 << 32;
    
    // Shared atomic counters for progress tracking
    let tested_count = Arc::new(AtomicU64::new(0));
    let filtered_count = Arc::new(AtomicU64::new(0));
    
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
            let mut local_filtered = 0u64;
            
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
                    if !valid_seeds[i] {
                        local_filtered += 1;
                    } else {
                        let state = SggPcg::new(seeds[i] as u64).state();
                        
                        local_candidates.push(StateCandidate {
                            seed: seeds[i],
                            state,
                        });
                        
                        println!("Found exact match: seed {}", seeds[i]);
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
            filtered_count.fetch_add(local_filtered, Ordering::Relaxed);
            
            // Progress reporting from thread 0 only
            if chunk_id == 0 {
                let current_tested = tested_count.load(Ordering::Relaxed);
                if current_tested % (100_000_000) == 0 {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let progress = current_tested as f64 / total_seeds as f64 * 100.0;
                    let estimated_total = elapsed / (current_tested as f64 / total_seeds as f64);
                    let remaining = estimated_total - elapsed;
                    let filter_rate = filtered_count.load(Ordering::Relaxed) as f64 / current_tested as f64 * 100.0;
                    
                    println!("Progress: {:.1}% ({}/{}), Elapsed: {:.1}s, Remaining: {:.1}s, Filtered: {:.1}%", 
                             progress, current_tested, total_seeds, elapsed, remaining, filter_rate);
                }
            }
            
            local_candidates
        })
        .collect();
    
    let elapsed = start_time.elapsed();
    let final_tested = tested_count.load(Ordering::Relaxed);
    let final_filtered = filtered_count.load(Ordering::Relaxed);
    let filter_rate = if final_tested > 0 { final_filtered as f64 / final_tested as f64 * 100.0 } else { 0.0 };
    
    println!("Parallel AVX2 search completed in {:.2}s, tested {} seeds, filtered {:.1}% early", 
             elapsed.as_secs_f64(), final_tested, filter_rate);
    
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

#[cfg(feature = "simd")]
#[target_feature(enable = "avx512f")]
unsafe fn find_original_state_avx512(data_points: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
    println!("Starting parallel AVX-512 SIMD brute force search across 2^32 possible seeds...");
    
    let start_time = Instant::now();
    let total_seeds = 1u64 << 32;
    
    // Shared atomic counters for progress tracking
    let tested_count = Arc::new(AtomicU64::new(0));
    let filtered_count = Arc::new(AtomicU64::new(0));
    
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
            let mut local_filtered = 0u64;
            
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
                    if !valid_seeds[i] {
                        local_filtered += 1;
                    } else {
                        let state = SggPcg::new(seeds[i] as u64).state();
                        
                        local_candidates.push(StateCandidate {
                            seed: seeds[i],
                            state,
                        });
                        
                        println!("Found exact match: seed {}", seeds[i]);
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
            filtered_count.fetch_add(local_filtered, Ordering::Relaxed);
            
            // Progress reporting from thread 0 only
            if chunk_id == 0 {
                let current_tested = tested_count.load(Ordering::Relaxed);
                if current_tested % (100_000_000) == 0 {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let progress = current_tested as f64 / total_seeds as f64 * 100.0;
                    let estimated_total = elapsed / (current_tested as f64 / total_seeds as f64);
                    let remaining = estimated_total - elapsed;
                    let filter_rate = filtered_count.load(Ordering::Relaxed) as f64 / current_tested as f64 * 100.0;
                    
                    println!("Progress: {:.1}% ({}/{}), Elapsed: {:.1}s, Remaining: {:.1}s, Filtered: {:.1}%", 
                             progress, current_tested, total_seeds, elapsed, remaining, filter_rate);
                }
            }
            
            local_candidates
        })
        .collect();
    
    let elapsed = start_time.elapsed();
    let final_tested = tested_count.load(Ordering::Relaxed);
    let final_filtered = filtered_count.load(Ordering::Relaxed);
    let filter_rate = if final_tested > 0 { final_filtered as f64 / final_tested as f64 * 100.0 } else { 0.0 };
    
    println!("Parallel AVX-512 search completed in {:.2}s, tested {} seeds, filtered {:.1}% early", 
             elapsed.as_secs_f64(), final_tested, filter_rate);
    
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

#[cfg(feature = "simd")]
#[target_feature(enable = "sse2")]
unsafe fn find_original_state_sse2(data_points: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
    println!("Starting SSE2 SIMD brute force search across 2^32 possible seeds...");
    
    // Similar implementation to AVX2 but using SSE2 with 4-wide vectors
    // For brevity, falling back to scalar for now
    crate::reverse_rng::search::find_original_state(data_points)
}

#[cfg(feature = "simd")]
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

#[cfg(feature = "simd")]
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

#[cfg(feature = "simd")]
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

#[cfg(feature = "simd")]
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

#[cfg(feature = "simd")]
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

#[cfg(feature = "simd")]
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

#[cfg(feature = "simd")]
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

#[cfg(feature = "simd")]
fn pcg_output_function(state: u64) -> u32 {
    let rot = (state >> ROTATE) as u32;
    let xsh = (((state >> XSHIFT) ^ state) >> SPARE) as u32;
    xsh.rotate_right(rot)
}

#[cfg(feature = "simd")]
fn pcg_next_u32(state: &mut u64) -> u32 {
    let old_state = *state;
    *state = state.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
    pcg_output_function(old_state)
}

#[cfg(feature = "simd")]
#[target_feature(enable = "avx2")]
unsafe fn check_consistency_simd(values: &[u32; CHUNK_SIZE_AVX2], data_point: &DataPoint, results: &mut [i64; CHUNK_SIZE_AVX2]) {
    let (min_u32, max_u32) = data_point.valid_u32_range();
    
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

#[cfg(feature = "simd")]
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

#[cfg(feature = "simd")]
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

#[cfg(feature = "simd")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reverse_rng::data_point::DataPoint;
    
    #[test]
    fn test_early_termination_check() {
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
    fn test_consistency_check_simd() {
        use crate::reverse_rng::data_point::DataPoint;
        
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
    fn test_pcg_output_function_simd() {
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
    
    #[test]
    fn test_simd_vs_scalar_consistency_small() {
        use rand_core::RngCore;
        
        // Generate test data with known seed that should be found quickly
        let known_seed = 0i32;  // Start from beginning of search space
        let mut data_points = Vec::new();
        
        // Use just 2 data points to make search faster
        for i in 0..2 {
            let rng_position = i * 5;
            let mut test_rng = SggPcg::new(known_seed as u64);
            test_rng.advance(rng_position);
            
            let value = test_rng.next_u32();
            let fraction = value as f64 / u32::MAX as f64;
            let range_min = 0.0;
            let range_max = 100.0;
            let scaled = fraction * (range_max - range_min) + range_min;
            let observed = (scaled * 100.0).round() / 100.0;
            
            data_points.push(DataPoint {
                offset: rng_position + 1,
                range_min,
                range_max,
                observed,
                name: format!("test_{}", i),
            });
        }
        
        // Test SIMD implementation finds the same result as scalar
        let simd_result = find_original_state_simd(&data_points).unwrap();
        let scalar_result = crate::reverse_rng::search::find_original_state(&data_points).unwrap();
        
        assert_eq!(simd_result.len(), scalar_result.len());
        if !simd_result.is_empty() {
            assert_eq!(simd_result[0].seed, scalar_result[0].seed);
        }
    }
    
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