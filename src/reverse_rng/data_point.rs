use crate::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub offset: u64,
    pub name: String,
    pub min_u32: u32,
    pub max_u32: u32,
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
    /// Calculate the valid u32 range for observed data point
    pub fn calculate_observed_u32_range(range_min: f64, range_max: f64, observed: f64) -> (u32, u32) {
        let range_size = range_max - range_min;
        
        // Find the range of actual values that would round to our observed value
        let actual_min = observed - 0.005;
        let actual_max = observed + 0.005;
        
        // Convert these actual values back to fractions of the [0,1] range
        let fraction_min = (actual_min - range_min) / range_size;
        let fraction_max = (actual_max - range_min) / range_size;
        
        // Clamp to valid [0, 1] range
        let fraction_min = fraction_min.max(0.0).min(1.0);
        let fraction_max = fraction_max.max(0.0).min(1.0);
        
        // Now find the u32 values that would map to these fractions
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

    /// Calculate the valid u32 range for range-based data point
    pub fn calculate_range_u32_range(range: f64, lower_bound: f64, upper_bound: f64) -> (u32, u32) {
        // Convert the bounds to fractions of the range [0, range]
        let fraction_min = lower_bound / range;
        let fraction_max = upper_bound / range;
        
        // Clamp to valid [0, 1] range
        let fraction_min = fraction_min.max(0.0).min(1.0);
        let fraction_max = fraction_max.max(0.0).min(1.0);
        
        // Find the u32 values that would map to these fractions
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
        value >= self.min_u32 && value <= self.max_u32
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
        
        if line.starts_with("/range ") {
            // New range format: "/range <name> <offset> <range> <lower_bound> <upper_bound>"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 6 {
                return Err(Error::from(format!(
                    "Invalid /range format on line {}: expected '/range <name> <offset> <range> <lower_bound> <upper_bound>'", 
                    line_num + 1
                )));
            }
            
            let name = parts[1].to_string();
            let offset = parts[2].parse::<u64>()
                .map_err(|_| Error::from(format!("Invalid offset on line {}", line_num + 1)))?;
            let range = parts[3].parse::<f64>()
                .map_err(|_| Error::from(format!("Invalid range on line {}", line_num + 1)))?;
            let lower_bound = parts[4].parse::<f64>()
                .map_err(|_| Error::from(format!("Invalid lower_bound on line {}", line_num + 1)))?;
            let upper_bound = parts[5].parse::<f64>()
                .map_err(|_| Error::from(format!("Invalid upper_bound on line {}", line_num + 1)))?;
            
            if lower_bound >= upper_bound {
                return Err(Error::from(format!(
                    "Invalid bounds on line {}: lower_bound must be less than upper_bound", 
                    line_num + 1
                )));
            }
            
            if range <= 0.0 {
                return Err(Error::from(format!(
                    "Invalid range on line {}: range must be positive", 
                    line_num + 1
                )));
            }
            
            if upper_bound > range {
                return Err(Error::from(format!(
                    "Invalid bounds on line {}: upper_bound ({}) cannot be greater than range ({}). Did you swap offset and range fields?", 
                    line_num + 1, upper_bound, range
                )));
            }
            
            if lower_bound < 0.0 {
                return Err(Error::from(format!(
                    "Invalid bounds on line {}: lower_bound ({}) cannot be negative", 
                    line_num + 1, lower_bound
                )));
            }
            
            let (min_u32, max_u32) = DataPoint::calculate_range_u32_range(range, lower_bound, upper_bound);
            data_points.push(DataPoint {
                offset,
                name,
                min_u32,
                max_u32,
            });
        } else {
            // Original format: "name,offset,min,max,observed"
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 5 {
                return Err(Error::from(format!(
                    "Invalid format on line {}: expected 5 comma-separated values or '/range' format", 
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
            
            if observed < range_min || observed > range_max {
                return Err(Error::from(format!(
                    "Invalid observed value on line {}: observed ({}) must be within range [{}, {}]", 
                    line_num + 1, observed, range_min, range_max
                )));
            }
            
            let (min_u32, max_u32) = DataPoint::calculate_observed_u32_range(range_min, range_max, observed);
            data_points.push(DataPoint {
                offset,
                name,
                min_u32,
                max_u32,
            });
        }
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
        let (min_u32, max_u32) = DataPoint::calculate_observed_u32_range(0.0, 1.0, 0.50);
        let _dp = DataPoint {
            offset: 0,
            name: "test".to_string(),
            min_u32,
            max_u32,
        };
        
        // For observed = 0.50 with range [0,1], we expect roughly the middle range
        assert!(min_u32 < max_u32);
        assert!(min_u32 < u32::MAX / 2);
        assert!(max_u32 > u32::MAX / 2);
    }
    
    #[test]
    fn test_consistency_check() {
        let (min_u32, max_u32) = DataPoint::calculate_observed_u32_range(0.0, 10.0, 5.00);
        let dp = DataPoint {
            offset: 0,
            name: "test".to_string(),
            min_u32,
            max_u32,
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
            let (curr_min, curr_max) = DataPoint::calculate_observed_u32_range(range_min, range_max, decimal_values[i]);
            let (next_min, next_max) = DataPoint::calculate_observed_u32_range(range_min, range_max, decimal_values[i + 1]);
            
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
                let (_, curr_max) = DataPoint::calculate_observed_u32_range(range_min, range_max, decimal_values[i]);
                let (next_min, _) = DataPoint::calculate_observed_u32_range(range_min, range_max, decimal_values[i + 1]);
                
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

    #[test]
    fn test_range_data_point_valid_range() {
        let (min_u32, max_u32) = DataPoint::calculate_range_u32_range(100.0, 25.0, 75.0);
        let _dp = DataPoint {
            offset: 0,
            name: "test_range".to_string(),
            min_u32,
            max_u32,
        };
        
        // For bounds [25, 75] in range [0, 100], we expect roughly 25%-75% of u32 space
        assert!(min_u32 < max_u32);
        assert!(min_u32 >= (u32::MAX as f64 * 0.25) as u32);
        assert!(max_u32 <= (u32::MAX as f64 * 0.75) as u32);
    }

    #[test]
    fn test_range_data_point_consistency() {
        let (min_u32, max_u32) = DataPoint::calculate_range_u32_range(1.0, 0.4, 0.6);
        let dp = DataPoint {
            offset: 0,
            name: "test_range".to_string(),
            min_u32,
            max_u32,
        };
        
        let middle_value = u32::MAX / 2;
        assert!(dp.is_consistent_with(middle_value));
        
        // Test edge cases
        assert!(!dp.is_consistent_with(0));
        assert!(!dp.is_consistent_with(u32::MAX));
    }

    #[test]
    fn test_parse_range_format() {
        let test_content = "# Test file with range format\n/range test_chamber 0 100.0 25.0 75.0\nchamber2,5,0.0,1.0,0.5\n";
        
        let mut data_points = Vec::new();
        
        for (_line_num, line) in test_content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if line.starts_with("/range ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                assert_eq!(parts.len(), 6);
                
                let name = parts[1].to_string();
                let offset = parts[2].parse::<u64>().unwrap();
                let range = parts[3].parse::<f64>().unwrap();
                let lower_bound = parts[4].parse::<f64>().unwrap();
                let upper_bound = parts[5].parse::<f64>().unwrap();
                
                let (min_u32, max_u32) = DataPoint::calculate_range_u32_range(range, lower_bound, upper_bound);
                data_points.push(DataPoint {
                    offset,
                    name,
                    min_u32,
                    max_u32,
                });
            } else {
                let parts: Vec<&str> = line.split(',').collect();
                assert_eq!(parts.len(), 5);
                
                let name = parts[0].to_string();
                let offset = parts[1].parse::<u64>().unwrap();
                let range_min = parts[2].parse::<f64>().unwrap();
                let range_max = parts[3].parse::<f64>().unwrap();
                let observed = parts[4].parse::<f64>().unwrap();
                
                let (min_u32, max_u32) = DataPoint::calculate_observed_u32_range(range_min, range_max, observed);
                data_points.push(DataPoint {
                    offset,
                    name,
                    min_u32,
                    max_u32,
                });
            }
        }
        
        assert_eq!(data_points.len(), 2);
        
        assert_eq!(data_points[0].name, "test_chamber");
        assert_eq!(data_points[0].offset, 0);
        
        assert_eq!(data_points[1].name, "chamber2");
        assert_eq!(data_points[1].offset, 5);
    }

    #[test]
    fn test_accessor_methods() {
        let (obs_min_u32, obs_max_u32) = DataPoint::calculate_observed_u32_range(0.0, 1.0, 0.5);
        let observed_dp = DataPoint {
            offset: 10,
            name: "observed_test".to_string(),
            min_u32: obs_min_u32,
            max_u32: obs_max_u32,
        };
        
        let (range_min_u32, range_max_u32) = DataPoint::calculate_range_u32_range(100.0, 25.0, 75.0);
        let range_dp = DataPoint {
            offset: 20,
            name: "range_test".to_string(),
            min_u32: range_min_u32,
            max_u32: range_max_u32,
        };
        
        assert_eq!(observed_dp.offset, 10);
        assert_eq!(observed_dp.name, "observed_test");
        
        assert_eq!(range_dp.offset, 20);
        assert_eq!(range_dp.name, "range_test");
    }
}