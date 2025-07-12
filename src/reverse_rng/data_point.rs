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
    
    #[test]
    fn test_complete_coverage_without_overlap() {
        // Test that consecutive two-decimal values have adjacent u32 ranges
        // without gaps or overlaps by checking just the endpoints
        
        let range_min = 1.3;
        let range_max = 1.5;
        
        // Generate consecutive decimal values to test adjacency
        let start_hundredths = (range_min * 100.0_f64).round() as i32;
        let end_hundredths = (range_max * 100.0_f64).round() as i32;
        
        let mut decimal_values = Vec::new();
        for hundredths in start_hundredths..=end_hundredths {
            let decimal_val = hundredths as f64 / 100.0;
            if decimal_val >= range_min && decimal_val <= range_max {
                decimal_values.push(decimal_val);
            }
        }
        
        println!("Testing {} consecutive decimal values in [{:.2}, {:.2}]", 
                 decimal_values.len(), range_min, range_max);
        
        // Check that consecutive values have adjacent ranges
        for i in 0..decimal_values.len() - 1 {
            let curr_dp = DataPoint {
                offset: 0,
                range_min,
                range_max,
                observed: decimal_values[i],
                name: format!("test_{:.2}", decimal_values[i]),
            };
            
            let next_dp = DataPoint {
                offset: 0,
                range_min,
                range_max,
                observed: decimal_values[i + 1],
                name: format!("test_{:.2}", decimal_values[i + 1]),
            };
            
            let (curr_min, curr_max) = curr_dp.valid_u32_range();
            let (next_min, next_max) = next_dp.valid_u32_range();
            
            // Check for perfect adjacency: curr_max + 1 should equal next_min
            if curr_max + 1 != next_min {
                if curr_max + 1 < next_min {
                    panic!("Gap found between {:.2} (ends at {}) and {:.2} (starts at {}): missing range [{}, {}]",
                           decimal_values[i], curr_max, decimal_values[i + 1], next_min, curr_max + 1, next_min - 1);
                } else {
                    panic!("Overlap found between {:.2} (ends at {}) and {:.2} (starts at {}): overlap range [{}, {}]",
                           decimal_values[i], curr_max, decimal_values[i + 1], next_min, next_min, curr_max);
                }
            }
            
            println!("  {:.2}: [{}, {}] -> {:.2}: [{}, {}] ✓", 
                     decimal_values[i], curr_min, curr_max,
                     decimal_values[i + 1], next_min, next_max);
        }
        
        println!("All consecutive decimal values have perfectly adjacent u32 ranges");
    }
    
    #[test]
    fn test_real_ursa_data_coverage() {
        // Test coverage for each range from real_ursa_data.txt using endpoint checking
        let test_ranges = vec![
            (1.3, 1.5, "nassault range"),
            (1.6, 1.8, "nambush range"), 
            (0.11, 0.2, "nfavor range"),
            (1.4, 1.6, "nlunge range"),
            (1.3, 1.4, "nstrike range"),
            (1.5, 1.8, "neclipse range"),
        ];
        
        for (range_min, range_max, description) in test_ranges {
            println!("\nTesting {}: [{:.2}, {:.2}]", description, range_min, range_max);
            
            // Generate consecutive decimal values in this range
            let start_hundredths = (range_min * 100.0_f64).round() as i32;
            let end_hundredths = (range_max * 100.0_f64).round() as i32;
            
            let mut decimal_values = Vec::new();
            for hundredths in start_hundredths..=end_hundredths {
                let decimal_val = hundredths as f64 / 100.0;
                if decimal_val >= range_min && decimal_val <= range_max {
                    decimal_values.push(decimal_val);
                }
            }
            
            println!("  Testing {} consecutive decimal values", decimal_values.len());
            
            // Check all adjacencies
            let mut issues_found = 0;
            
            for i in 0..decimal_values.len() - 1 {
                let curr_dp = DataPoint {
                    offset: 0, range_min, range_max,
                    observed: decimal_values[i],
                    name: format!("test_{:.2}", decimal_values[i]),
                };
                let next_dp = DataPoint {
                    offset: 0, range_min, range_max,
                    observed: decimal_values[i + 1],
                    name: format!("test_{:.2}", decimal_values[i + 1]),
                };
                
                let (_, curr_max) = curr_dp.valid_u32_range();
                let (next_min, _) = next_dp.valid_u32_range();
                
                if curr_max + 1 != next_min {
                    issues_found += 1;
                    if curr_max + 1 < next_min {
                        println!("  Gap: {:.2} ends at {}, {:.2} starts at {}", 
                                decimal_values[i], curr_max, decimal_values[i + 1], next_min);
                    } else {
                        println!("  Overlap: {:.2} ends at {}, {:.2} starts at {}", 
                                decimal_values[i], curr_max, decimal_values[i + 1], next_min);
                    }
                }
            }
            
            assert_eq!(issues_found, 0, "Found {} coverage issues in {}", issues_found, description);
            println!("  ✓ No gaps or overlaps found in all adjacencies");
        }
    }
}