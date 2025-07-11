use crate::error::Error;
use crate::reverse_rng::data_point::{DataPoint, StateCandidate};
use crate::rng::SggPcg;
use rand::RngCore;
use std::time::Instant;

pub fn find_original_state(data_points: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
    println!("Starting brute force search across 2^32 possible seeds...");
    
    // Sort data points by constraint strength (most restrictive first)
    let mut sorted_data_points = data_points.to_vec();
    sorted_data_points.sort_by(|a, b| {
        let constraint_a = constraint_strength(a);
        let constraint_b = constraint_strength(b);
        constraint_b.partial_cmp(&constraint_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    println!("Constraint analysis:");
    for (i, dp) in sorted_data_points.iter().enumerate() {
        let (min_u32, max_u32) = dp.valid_u32_range();
        let range_size = (max_u32 as u64).saturating_sub(min_u32 as u64) + 1;
        let selectivity = range_size as f64 / (u32::MAX as f64 + 1.0);
        println!("  {}: {} offset={}, range_size={}, selectivity={:.6}", 
                 i+1, dp.name, dp.offset, range_size, selectivity);
    }
    
    let start_time = Instant::now();
    let mut candidates = Vec::new();
    let mut tested_count = 0u64;
    let mut filtered_count = 0u64;
    let total_seeds = 1u64 << 32; // 2^32
    
    // Test all possible i32 seed values
    for seed in i32::MIN..=i32::MAX {
        tested_count += 1;
        
        // Progress reporting every 100M iterations
        if tested_count % 100_000_000 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let progress = tested_count as f64 / total_seeds as f64 * 100.0;
            let estimated_total = elapsed / (tested_count as f64 / total_seeds as f64);
            let remaining = estimated_total - elapsed;
            let filter_rate = filtered_count as f64 / tested_count as f64 * 100.0;
            
            println!("Progress: {:.1}% ({}/{}), Elapsed: {:.1}s, Remaining: {:.1}s, Filtered: {:.1}%", 
                     progress, tested_count, total_seeds, elapsed, remaining, filter_rate);
        }
        
        // Pre-filter using constraint analysis
        if !quick_constraint_check(seed, &sorted_data_points) {
            filtered_count += 1;
            continue;
        }
        
        if is_valid_seed(seed, &sorted_data_points) {
            let state = SggPcg::new(seed as u64).state();
            
            candidates.push(StateCandidate {
                seed,
                state,
            });
            
            println!("Found exact match: seed {}", seed);
        }
    }
    
    let elapsed = start_time.elapsed();
    let filter_rate = if tested_count > 0 { filtered_count as f64 / tested_count as f64 * 100.0 } else { 0.0 };
    println!("Search completed in {:.2}s, tested {} seeds, filtered {:.1}% early", 
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

fn constraint_strength(data_point: &DataPoint) -> f64 {
    let (min_u32, max_u32) = data_point.valid_u32_range();
    let range_size = (max_u32 as u64).saturating_sub(min_u32 as u64) + 1;
    // Return selectivity (smaller = more constraining)
    range_size as f64 / (u32::MAX as f64 + 1.0)
}


fn quick_constraint_check(seed: i32, data_points: &[DataPoint]) -> bool {
    // Check the top 2-3 most constraining data points for quick elimination
    let num_to_check = data_points.len().min(3);
    let rng = SggPcg::new(seed as u64);
    
    for i in 0..num_to_check {
        let data_point = &data_points[i];
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

fn is_valid_seed(seed: i32, data_points: &[DataPoint]) -> bool {
    let rng = SggPcg::new(seed as u64);
    
    // Process data points in order of constraint strength (most restrictive first)
    // This allows early termination when a constraint fails
    for data_point in data_points {
        // Advance RNG to one position before the offset, since the offset 
        // represents the position where the value was generated (not where to advance to)
        let mut test_rng = rng.clone();
        if data_point.offset > 0 {
            test_rng.advance(data_point.offset - 1);
        }
        
        // Generate the value
        let generated_u32 = test_rng.next_u32();
        
        // Check if it's consistent with the observed value - early termination on first failure
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
    fn test_known_seed_validation() {
        // Create a known RNG state and generate some test data
        let known_seed = 12345i32;
        let _rng = SggPcg::new(known_seed as u64);
        
        // Generate some test data points
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
            
            // The offset in the data point represents the position where the value was generated
            // So we need to add 1 to account for the next_u32() call
            data_points.push(DataPoint {
                offset: rng_position + 1,
                range_min,
                range_max,
                observed,
                name: format!("test_{}", i),
            });
        }
        
        // Test that our validation works correctly - the known seed should validate
        assert!(is_valid_seed(known_seed, &data_points));
        
        // Test that a different seed should not validate (with high probability)
        assert!(!is_valid_seed(known_seed + 1, &data_points));
    }
}