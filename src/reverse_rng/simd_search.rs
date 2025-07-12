use crate::error::Error;
use crate::reverse_rng::data_point::{DataPoint, StateCandidate};

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
use crate::reverse_rng::simd_avx2::find_original_state_avx2;
#[cfg(all(feature = "simd_nightly", target_arch = "x86_64"))]
use crate::reverse_rng::simd_avx512::find_original_state_avx512;

pub fn find_original_state_simd(data_points: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
    #[cfg(feature = "simd")]
    {
        #[cfg(feature = "simd_nightly")]
        {
            if is_x86_feature_detected!("avx512f") {
                println!("Using AVX-512 SIMD optimization for brute force search...");
                return unsafe { find_original_state_avx512(data_points) };
            }
        }

        if is_x86_feature_detected!("avx2") {
            println!("Using AVX2 SIMD optimization for brute force search...");
            return unsafe { find_original_state_avx2(data_points) };
        }
    }
    
    println!("SIMD not available or not enabled, falling back to scalar implementation");
    crate::reverse_rng::search::find_original_state(data_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reverse_rng::data_point::DataPoint;
    use crate::rng::SggPcg;
    use rand_core::RngCore;
    
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