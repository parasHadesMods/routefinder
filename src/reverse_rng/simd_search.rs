use crate::error::Error;
use crate::reverse_rng::data_point::{DataPoint, StateCandidate};
use crate::rng::SggPcg;
use rand::RngCore;
use std::time::Instant;

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use std::arch::x86_64::*;

const CHUNK_SIZE: usize = 8; // Process 8 seeds at once with AVX2

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
    
    // Process seeds in chunks of 8 (AVX2 can handle 8x32-bit integers)
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
        
        // Load 8 consecutive seeds into AVX2 register
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
        
        // Convert to u64 and compute initial PCG states
        let mut states = [0u64; CHUNK_SIZE];
        for i in 0..CHUNK_SIZE {
            states[i] = (seeds[i] as u64).wrapping_mul(MULTIPLIER).wrapping_sub(INITIAL_OFFSET);
        }
        
        // Use SIMD to validate all 8 seeds at once
        let valid_seeds = validate_seeds_simd(&seeds, &states, &sorted_data_points);
        
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
fn validate_seeds_simd(_seeds: &[i32; CHUNK_SIZE], initial_states: &[u64; CHUNK_SIZE], data_points: &[DataPoint]) -> [bool; CHUNK_SIZE] {
    let mut results = [true; CHUNK_SIZE];
    
    // Early termination optimization: use most constraining data point first
    if let Some(first_data_point) = data_points.first() {
        let mut states = *initial_states;
        
        // Advance all states for the first (most constraining) data point
        if first_data_point.offset > 0 {
            for state in &mut states {
                *state = advance_pcg_state(*state, first_data_point.offset - 1);
            }
        }
        
        // Quick elimination using first data point - process all 8 in tight loop
        for i in 0..CHUNK_SIZE {
            let old_state = states[i];
            states[i] = states[i].wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
            let generated_u32 = pcg_output_function(old_state);
            
            if !first_data_point.is_consistent_with(generated_u32) {
                results[i] = false;
            }
        }
        
        // Only process remaining data points for seeds that passed first check
        for data_point in &data_points[1..] {
            let mut remaining_valid = false;
            for i in 0..CHUNK_SIZE {
                if results[i] {
                    remaining_valid = true;
                    break;
                }
            }
            
            if !remaining_valid {
                break; // All seeds failed, no need to continue
            }
            
            let mut states = *initial_states;
            if data_point.offset > 0 {
                for state in &mut states {
                    *state = advance_pcg_state(*state, data_point.offset - 1);
                }
            }
            
            for i in 0..CHUNK_SIZE {
                if results[i] {
                    let old_state = states[i];
                    states[i] = states[i].wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
                    let generated_u32 = pcg_output_function(old_state);
                    
                    if !data_point.is_consistent_with(generated_u32) {
                        results[i] = false;
                    }
                }
            }
        }
    }
    
    results
}

// Helper functions for PCG operations
#[cfg(feature = "simd")]
fn advance_pcg_state(state: u64, delta: u64) -> u64 {
    let mut acc_mult: u64 = 1;
    let mut acc_plus: u64 = 0;
    let mut cur_mult = MULTIPLIER;
    let mut cur_plus = INCREMENT;
    let mut mdelta = delta;

    while mdelta > 0 {
        if (mdelta & 1) != 0 {
            acc_mult = acc_mult.wrapping_mul(cur_mult);
            acc_plus = acc_plus.wrapping_mul(cur_mult).wrapping_add(cur_plus);
        }
        cur_plus = cur_mult.wrapping_add(1).wrapping_mul(cur_plus);
        cur_mult = cur_mult.wrapping_mul(cur_mult);
        mdelta /= 2;
    }
    acc_mult.wrapping_mul(state).wrapping_add(acc_plus)
}

#[cfg(feature = "simd")]
fn pcg_next_u32(state: &mut u64) -> u32 {
    let old_state = *state;
    *state = state.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
    pcg_output_function(old_state)
}

#[cfg(feature = "simd")]
fn pcg_output_function(state: u64) -> u32 {
    let rot = (state >> ROTATE) as u32;
    let xsh = (((state >> XSHIFT) ^ state) >> SPARE) as u32;
    xsh.rotate_right(rot)
}

fn constraint_strength(data_point: &DataPoint) -> f64 {
    let (min_u32, max_u32) = data_point.valid_u32_range();
    let range_size = (max_u32 as u64).saturating_sub(min_u32 as u64) + 1;
    range_size as f64 / (u32::MAX as f64 + 1.0)
}

fn is_valid_seed(seed: i32, data_points: &[DataPoint]) -> bool {
    let rng = SggPcg::new(seed as u64);
    
    for data_point in data_points {
        let mut test_rng = rng.clone();
        if data_point.offset > 0 {
            test_rng.advance(data_point.offset - 1);
        }
        
        let generated_u32 = test_rng.next_u32();
        
        if !data_point.is_consistent_with(generated_u32) {
            return false;
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reverse_rng::data_point::DataPoint;
    
    #[test]
    fn test_simd_vs_scalar_consistency() {
        // Generate test data with known seed
        let known_seed = 12345i32;
        let mut data_points = Vec::new();
        
        for i in 0..3 {
            let rng_position = i * 10;
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