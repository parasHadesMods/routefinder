pub mod data_point;
pub mod search;

use crate::error::Error;
use std::path::PathBuf;

pub fn run(input_file: PathBuf) -> Result<(), Error> {
    println!("Reverse engineering RNG state from: {:?}", input_file);
    
    // Parse input file
    let data_points = data_point::parse_input_file(&input_file)?;
    println!("Loaded {} data points", data_points.len());
    
    // Analyze and recommend optimal data point count
    let recommendation = data_point::analyze_time_optimization(&data_points);
    println!("Time optimization analysis:");
    println!("  Data points: {}", recommendation.data_points);
    println!("  Estimated execution time: {:.2}s", recommendation.estimated_execution_time);
    println!("  Total time (collection + execution): {:.2}s", recommendation.total_time_seconds);
    
    if data_points.len() < 6 {
        println!("Warning: {} data points may not be sufficient for unique identification", data_points.len());
        println!("Recommended: 6-7 data points for confident identification");
    }
    
    // Perform reverse engineering
    match search::find_original_state(&data_points) {
        Ok(candidates) => {
            if candidates.is_empty() {
                println!("No valid RNG states found that match all data points");
            } else {
                println!("Found {} candidate state(s):", candidates.len());
                for (i, candidate) in candidates.iter().enumerate() {
                    println!("  Candidate {}: seed {}", 
                             i + 1, candidate.seed);
                }
            }
        }
        Err(e) => {
            eprintln!("Error during reverse engineering: {}", e);
        }
    }
    
    Ok(())
}