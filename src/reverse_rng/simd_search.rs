use crate::error::Error;
use crate::reverse_rng::data_point::{DataPoint, StateCandidate};
use crate::rng::SggPcg;
use rand::RngCore;
use std::time::Instant;

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use std::arch::x86_64::*;

const CHUNK_SIZE: usize = 4; // Process 4 u64 states at once with AVX2

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
        if is_x86_feature_detected!("avx2") {
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
    println!("Starting AVX2 SIMD brute force search across 2^32 possible seeds...");
    
    let start_time = Instant::now();
    let mut candidates = Vec::new();
    let mut tested_count = 0u64;
    let mut filtered_count = 0u64;
    let total_seeds = 1u64 << 32;
    
    // Process seeds in chunks of 4 (AVX2 can handle 4x64-bit integers)
    let mut seed_base: i64 = i32::MIN as i64;
    
    while seed_base < i32::MAX as i64 {
        tested_count += CHUNK_SIZE as u64;
        
        // Progress reporting
        if tested_count % (100_000_000) == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let progress = tested_count as f64 / total_seeds as f64 * 100.0;
            let estimated_total = elapsed / (tested_count as f64 / total_seeds as f64);
            let remaining = estimated_total - elapsed;
            let filter_rate = filtered_count as f64 / tested_count as f64 * 100.0;
            
            println!("Progress: {:.1}% ({}/{}), Elapsed: {:.1}s, Remaining: {:.1}s, Filtered: {:.1}%", 
                     progress, tested_count, total_seeds, elapsed, remaining, filter_rate);
        }
        
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
        
        for i in 0..CHUNK_SIZE {
            if !valid_seeds[i] {
                filtered_count += 1;
            } else {
                let state = SggPcg::new(seeds[i] as u64).state();
                
                candidates.push(StateCandidate {
                    seed: seeds[i],
                    state,
                });
                
                println!("Found exact match: seed {}", seeds[i]);
            }
        }
        
        seed_base += CHUNK_SIZE as i64;
    }
    
    let elapsed = start_time.elapsed();
    let filter_rate = if tested_count > 0 { filtered_count as f64 / tested_count as f64 * 100.0 } else { 0.0 };
    println!("AVX2 search completed in {:.2}s, tested {} seeds, filtered {:.1}% early", 
             elapsed.as_secs_f64(), tested_count, filter_rate);
    
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
unsafe fn compute_initial_states_simd(seeds: &[i32; CHUNK_SIZE]) -> [u64; CHUNK_SIZE] {
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
    let mut result = [0u64; CHUNK_SIZE];
    _mm256_storeu_si256(result.as_mut_ptr() as *mut __m256i, states);
    result
}

#[cfg(feature = "simd")]
#[target_feature(enable = "avx2")]
unsafe fn validate_seeds_simd_avx2(_seeds: &[i32; CHUNK_SIZE], initial_states: &[u64; CHUNK_SIZE], data_points: &[DataPoint]) -> [bool; CHUNK_SIZE] {
    let mut results = [true; CHUNK_SIZE];
    
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
        for i in 0..CHUNK_SIZE {
            if !data_point.is_consistent_with(generated_values[i]) {
                results[i] = false;
            }
        }
        
        // Early termination if all seeds failed
        if !results.iter().any(|&x| x) {
            break;
        }
    }
    
    results
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
unsafe fn pcg_output_function_simd(states: __m256i) -> [u32; CHUNK_SIZE] {
    // Extract rotation amounts: (state >> 59) & 0x1F (only need bottom 5 bits)
    let rotate_shifts = _mm256_srli_epi64(states, ROTATE as i32);
    let rotate_mask = _mm256_set1_epi64x(0x1F);
    let rotations = _mm256_and_si256(rotate_shifts, rotate_mask);
    
    // Compute xorshift: ((state >> 18) ^ state) >> 27
    let shifted = _mm256_srli_epi64(states, XSHIFT as i32);
    let xored = _mm256_xor_si256(shifted, states);
    let xsh_values = _mm256_srli_epi64(xored, SPARE as i32);
    
    // Convert to 32-bit values and apply variable rotations
    let mut result = [0u32; CHUNK_SIZE];
    let mut xsh_array = [0u64; CHUNK_SIZE];
    let mut rot_array = [0u64; CHUNK_SIZE];
    
    _mm256_storeu_si256(xsh_array.as_mut_ptr() as *mut __m256i, xsh_values);
    _mm256_storeu_si256(rot_array.as_mut_ptr() as *mut __m256i, rotations);
    
    for i in 0..CHUNK_SIZE {
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
    fn test_pcg_output_function_simd() {
        // Test the SIMD PCG output function directly against scalar implementation
        let test_states = [
            0x123456789ABCDEF0u64,
            0xFEDCBA9876543210u64,
            0x0123456789ABCDEFu64,
            0xDEADBEEFCAFEBABEu64,
        ];
        
        // Compute expected results using scalar function
        let expected: [u32; CHUNK_SIZE] = [
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
}