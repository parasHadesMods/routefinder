pub mod data_point;
pub mod search;
#[cfg(feature = "simd")]
pub mod simd_search;

use crate::error::Error;
use std::path::PathBuf;

pub fn run(input_file: PathBuf) -> Result<(), Error> {
    println!("Reverse engineering RNG state from: {:?}", input_file);
    
    // Parse input file
    let data_points = data_point::parse_input_file(&input_file)?;
    println!("Loaded {} data points", data_points.len());
    
    // Perform reverse engineering
    #[cfg(feature = "simd")]
    let search_result = simd_search::find_original_state_simd(&data_points);
    #[cfg(not(feature = "simd"))]
    let search_result = search::find_original_state(&data_points);
    
    match search_result {
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