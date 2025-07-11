use crate::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub offset: u64,
    pub range_min: f64,
    pub range_max: f64,
    pub observed: f64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct StateCandidate {
    pub seed: i32,
    pub state: u64,
}

#[derive(Debug, Clone)]
pub struct TimeOptimization {
    pub data_points: usize,
    pub estimated_execution_time: f64,
    pub total_time_seconds: f64,
}

impl DataPoint {
    /// Calculate the valid u32 range for this data point given rounding constraints
    pub fn valid_u32_range(&self) -> (u32, u32) {
        // We need to find which u32 values would produce our observed value
        // when converted through: u32 -> [0,1] fraction -> scaled -> rounded to 2 decimal places
        
        // The observed value represents values that round to this when rounded to 2 decimal places
        // This means the actual value before rounding was in the range [observed - 0.005, observed + 0.005)
        // But we need to be more precise about the boundaries
        
        let range_size = self.range_max - self.range_min;
        
        // Values that would round to our observed value:
        // If observed = X.YZ, then actual values in [X.YZ - 0.005, X.YZ + 0.005) would round to X.YZ
        
        // Find the range of actual values that would round to our observed value
        let actual_min = self.observed - 0.005;
        let actual_max = self.observed + 0.005;
        
        // Convert these actual values back to fractions of the [0,1] range
        let fraction_min = (actual_min - self.range_min) / range_size;
        let fraction_max = (actual_max - self.range_min) / range_size;
        
        // Clamp to valid [0, 1] range
        let fraction_min = fraction_min.max(0.0).min(1.0);
        let fraction_max = fraction_max.max(0.0).min(1.0);
        
        // Now find the u32 values that would map to these fractions
        // u32 value V maps to fraction V / u32::MAX
        // So we want u32 values V where fraction_min <= V / u32::MAX <= fraction_max
        // Which means: fraction_min * u32::MAX <= V <= fraction_max * u32::MAX
        
        let min_u32_exact = fraction_min * u32::MAX as f64;
        let max_u32_exact = fraction_max * u32::MAX as f64;
        
        // Convert to actual u32 bounds
        let min_u32 = min_u32_exact.ceil() as u32;
        let max_u32 = max_u32_exact.floor() as u32;
        
        // Handle edge case where the range is too narrow
        if min_u32 > max_u32 {
            let mid_u32 = ((min_u32_exact + max_u32_exact) * 0.5) as u32;
            (mid_u32, mid_u32)
        } else {
            (min_u32, max_u32)
        }
    }
    
    /// Check if a given u32 value is consistent with this data point
    pub fn is_consistent_with(&self, value: u32) -> bool {
        let (min_u32, max_u32) = self.valid_u32_range();
        value >= min_u32 && value <= max_u32
    }
    
}

pub fn parse_input_file(file_path: &Path) -> Result<Vec<DataPoint>, Error> {
    let contents = fs::read_to_string(file_path)
        .map_err(|e| Error::IO { error: e })?;
    
    let mut data_points = Vec::new();
    
    for (line_num, line) in contents.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Expected format: "name,offset,min,max,observed"
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 5 {
            return Err(Error::from(format!(
                "Invalid format on line {}: expected 5 comma-separated values", 
                line_num + 1
            )));
        }
        
        let name = parts[0].to_string();
        let offset = parts[1].parse::<u64>()
            .map_err(|_| Error::from(format!("Invalid offset on line {}", line_num + 1)))?;
        let range_min = parts[2].parse::<f64>()
            .map_err(|_| Error::from(format!("Invalid range_min on line {}", line_num + 1)))?;
        let range_max = parts[3].parse::<f64>()
            .map_err(|_| Error::from(format!("Invalid range_max on line {}", line_num + 1)))?;
        let observed = parts[4].parse::<f64>()
            .map_err(|_| Error::from(format!("Invalid observed value on line {}", line_num + 1)))?;
        
        if range_min >= range_max {
            return Err(Error::from(format!(
                "Invalid range on line {}: min must be less than max", 
                line_num + 1
            )));
        }
        
        data_points.push(DataPoint {
            offset,
            range_min,
            range_max,
            observed,
            name,
        });
    }
    
    if data_points.is_empty() {
        return Err(Error::from("No valid data points found in input file".to_string()));
    }
    
    Ok(data_points)
}

pub fn analyze_time_optimization(data_points: &[DataPoint]) -> TimeOptimization {
    let num_points = data_points.len();
    
    // Estimate execution time based on constraint strength
    // Each data point eliminates approximately 95% of candidates (reduces by factor of ~20)
    let remaining_candidates = (4_300_000_000_u64 as f64) / (20_f64.powi(num_points as i32));
    
    // Assume we can check ~1 billion states per second
    let estimated_execution_time = (remaining_candidates / 1_000_000_000.0).max(0.1);
    
    // Total time = collection time + execution time
    let collection_time = num_points as f64 * 2.0; // 2 seconds per data point
    let total_time = collection_time + estimated_execution_time;
    
    TimeOptimization {
        data_points: num_points,
        estimated_execution_time,
        total_time_seconds: total_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_point_valid_range() {
        let dp = DataPoint {
            offset: 0,
            range_min: 0.0,
            range_max: 1.0,
            observed: 0.50,
            name: "test".to_string(),
        };
        
        let (min, max) = dp.valid_u32_range();
        
        // For observed = 0.50 with range [0,1], we expect roughly the middle range
        assert!(min < max);
        assert!(min < u32::MAX / 2);
        assert!(max > u32::MAX / 2);
    }
    
    #[test]
    fn test_consistency_check() {
        let dp = DataPoint {
            offset: 0,
            range_min: 0.0,
            range_max: 10.0,
            observed: 5.00,
            name: "test".to_string(),
        };
        
        let middle_value = u32::MAX / 2;
        assert!(dp.is_consistent_with(middle_value));
    }
}