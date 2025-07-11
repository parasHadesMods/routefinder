use crate::error::Error;
use crate::reverse_rng::data_point::{DataPoint, StateCandidate};
use crate::rng::SggPcg;
use rand::RngCore;
use std::time::Instant;

pub fn find_original_state(data_points: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
    println!("Starting brute force search across 2^32 possible seeds...");
    let start_time = Instant::now();
    
    let mut candidates = Vec::new();
    let mut tested_count = 0u64;
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
            
            println!("Progress: {:.1}% ({}/{}), Elapsed: {:.1}s, Remaining: {:.1}s", 
                     progress, tested_count, total_seeds, elapsed, remaining);
        }
        
        if is_valid_seed(seed, data_points) {
            let state = SggPcg::new(seed as u64).state();
            let confidence = calculate_confidence(seed, data_points);
            let error_metrics = calculate_error_metrics(seed, data_points);
            
            candidates.push(StateCandidate {
                seed,
                state,
                confidence,
                error_metrics,
            });
            
            println!("Found candidate: seed {}, confidence: {:.4}", seed, confidence);
            
            // If we find a perfect match (confidence ~1.0), we can stop early
            if confidence > 0.999 {
                println!("Found high-confidence match, stopping search early");
                break;
            }
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("Search completed in {:.2}s, tested {} seeds", elapsed.as_secs_f64(), tested_count);
    
    // Sort candidates by confidence (highest first)
    candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    
    Ok(candidates)
}

fn is_valid_seed(seed: i32, data_points: &[DataPoint]) -> bool {
    let rng = SggPcg::new(seed as u64);
    
    for data_point in data_points {
        // Advance RNG to the offset for this data point
        let mut test_rng = rng.clone();
        test_rng.advance(data_point.offset);
        
        // Generate the value
        let generated_u32 = test_rng.next_u32();
        
        // Check if it's consistent with the observed value
        if !data_point.is_consistent_with(generated_u32) {
            return false;
        }
    }
    
    true
}

fn calculate_confidence(seed: i32, data_points: &[DataPoint]) -> f64 {
    let mut total_error = 0.0;
    let rng = SggPcg::new(seed as u64);
    
    for data_point in data_points {
        let mut test_rng = rng.clone();
        test_rng.advance(data_point.offset);
        
        let generated_u32 = test_rng.next_u32();
        let generated_value = data_point.calculate_generated_value(generated_u32);
        
        let error = (generated_value - data_point.observed).abs();
        total_error += error;
    }
    
    // Convert error to confidence (lower error = higher confidence)
    // Maximum possible error per data point is about 0.005 (rounding precision)
    let max_possible_error = data_points.len() as f64 * 0.005;
    let confidence = 1.0 - (total_error / max_possible_error).min(1.0);
    
    confidence.max(0.0)
}

fn calculate_error_metrics(seed: i32, data_points: &[DataPoint]) -> Vec<f64> {
    let mut errors = Vec::new();
    let rng = SggPcg::new(seed as u64);
    
    for data_point in data_points {
        let mut test_rng = rng.clone();
        test_rng.advance(data_point.offset);
        
        let generated_u32 = test_rng.next_u32();
        let generated_value = data_point.calculate_generated_value(generated_u32);
        
        let error = (generated_value - data_point.observed).abs();
        errors.push(error);
    }
    
    errors
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::reverse_rng::data_point::DataPoint;
    
    #[test]
    fn test_known_seed_recovery() {
        // Create a known RNG state and generate some test data
        let known_seed = 12345i32;
        let _rng = SggPcg::new(known_seed as u64);
        
        // Generate some test data points
        let mut data_points = Vec::new();
        
        for i in 0..3 {
            let offset = i * 10;
            let mut test_rng = SggPcg::new(known_seed as u64);
            test_rng.advance(offset);
            
            let value = test_rng.next_u32();
            let fraction = value as f64 / u32::MAX as f64;
            let range_min = 0.0;
            let range_max = 100.0;
            let scaled = fraction * (range_max - range_min) + range_min;
            let observed = (scaled * 100.0).round() / 100.0;
            
            data_points.push(DataPoint {
                offset,
                range_min,
                range_max,
                observed,
                name: format!("test_{}", i),
            });
        }
        
        // Test validation
        assert!(is_valid_seed(known_seed, &data_points));
        
        // Test confidence calculation
        let confidence = calculate_confidence(known_seed, &data_points);
        assert!(confidence > 0.99, "Confidence should be very high for exact match");
    }
}