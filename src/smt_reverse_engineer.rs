//! SMT-based RNG reverse engineering implementation
//! 
//! This module implements Phase 2.1 from the SMT approach: Basic LCG State Advancement
//! 
//! Key components:
//! - SMTReverseEngineer: Main solver interface
//! - LCG state advancement using Z3 bit-vector operations  
//! - PCG output function encoding
//! - Range constraint encoding for floating-point bounds

#[cfg(feature = "smt")]
use z3::*;
#[cfg(feature = "smt")]
use z3::ast::{Ast, BV};

use crate::reverse_rng::data_point::{DataPoint, StateCandidate};
use crate::error::Error;

/// LCG constants from the original RNG implementation
pub const LCG_MULTIPLIER: u64 = 0x5851f42d4c957f2d;
pub const LCG_INCREMENT: u64 = 0xb47c73972972b7b7;

/// PCG output function constants
pub const ROTATE_BITS: u32 = 59;  // 64 - 5
pub const XSHIFT_BITS: u32 = 18;  // (5 + 32) / 2
pub const SPARE_BITS: u32 = 27;   // 64 - 32 - 5

#[cfg(feature = "smt")]
pub struct SMTReverseEngineer<'ctx> {
    context: &'ctx Context,
    solver: Solver<'ctx>,
    initial_state: BV<'ctx>,
    lcg_multiplier: BV<'ctx>,
    lcg_increment: BV<'ctx>,
}

#[cfg(feature = "smt")]
impl<'ctx> SMTReverseEngineer<'ctx> {
    /// Create a new SMT reverse engineer instance
    pub fn new(context: &'ctx Context) -> Self {
        let solver = Solver::new(context);
        
        // Create the initial state variable (what we're solving for)
        let initial_state = BV::new_const(context, "initial_state", 64);
        
        // LCG constants as bit-vectors
        let lcg_multiplier = BV::from_u64(context, LCG_MULTIPLIER, 64);
        let lcg_increment = BV::from_u64(context, LCG_INCREMENT, 64);
        
        SMTReverseEngineer {
            context,
            solver,
            initial_state,
            lcg_multiplier,
            lcg_increment,
        }
    }
    
    /// Advance LCG state by a single step
    /// Implementation: state' = (state * multiplier + increment) mod 2^64
    fn advance_state_single(&self, state: &BV<'ctx>) -> BV<'ctx> {
        // LCG formula: new_state = (state * MULTIPLIER + INCREMENT) mod 2^64
        // The modulo 2^64 is implicit in 64-bit bit-vector arithmetic
        let multiplied = state.bvmul(&self.lcg_multiplier);
        multiplied.bvadd(&self.lcg_increment)
    }
    
    /// Advance LCG state by multiple steps efficiently
    /// For now, we compute the multiplier and increment values at compile time
    /// and use them directly to avoid complex constraint chains
    fn advance_state_by_steps(&self, initial_state: &BV<'ctx>, steps: u64) -> BV<'ctx> {
        if steps == 0 {
            return initial_state.clone();
        }
        
        if steps == 1 {
            return self.advance_state_single(initial_state);
        }
        
        // For small step counts, just iterate to keep it simple
        if steps <= 5 {
            let mut current_state = initial_state.clone();
            for _ in 0..steps {
                current_state = self.advance_state_single(&current_state);
            }
            return current_state;
        }
        
        // For larger step counts, pre-compute the advancement constants
        // This uses the exact same logic as the original RNG advance() method
        self.advance_state_precomputed(initial_state, steps)
    }
    
    /// Advance state using pre-computed multiplier and increment for the given step count
    fn advance_state_precomputed(&self, initial_state: &BV<'ctx>, steps: u64) -> BV<'ctx> {
        // Pre-compute advancement constants at runtime to avoid complex SMT constraints
        let (final_mult, final_inc) = self.compute_advancement_constants(steps);
        
        // Create SMT constants for the pre-computed values
        let mult_bv = BV::from_u64(self.context, final_mult, 64);
        let inc_bv = BV::from_u64(self.context, final_inc, 64);
        
        // Apply: new_state = initial_state * final_mult + final_inc
        initial_state.bvmul(&mult_bv).bvadd(&inc_bv)
    }
    
    /// Compute the effective multiplier and increment for advancing n steps
    /// This uses the same algorithm as the original RNG advance method
    fn compute_advancement_constants(&self, steps: u64) -> (u64, u64) {
        let mut acc_mult: u64 = 1;
        let mut acc_plus: u64 = 0;
        let mut cur_mult = LCG_MULTIPLIER;
        let mut cur_plus = LCG_INCREMENT;
        let mut delta = steps;
        
        while delta > 0 {
            if (delta & 1) != 0 {
                acc_mult = acc_mult.wrapping_mul(cur_mult);
                acc_plus = acc_plus.wrapping_mul(cur_mult).wrapping_add(cur_plus);
            }
            cur_plus = cur_mult.wrapping_add(1).wrapping_mul(cur_plus);
            cur_mult = cur_mult.wrapping_mul(cur_mult);
            delta /= 2;
        }
        
        (acc_mult, acc_plus)
    }
    
    /// Apply PCG output function (XSH-RR) to get 32-bit output
    /// This implements the core PCG transformation from 64-bit state to 32-bit output
    fn apply_pcg_output(&self, state: &BV<'ctx>) -> BV<'ctx> {
        // Step 1: Extract rotation amount from top 5 bits (bits 63-59)
        let rot = state.extract(63, ROTATE_BITS);  // Extract top 5 bits for rotation
        
        // Step 2: Compute XSH (xorshift high) value
        // xsh = ((state >> XSHIFT_BITS) ^ state) >> SPARE_BITS
        let shift_amount = BV::from_u64(self.context, XSHIFT_BITS as u64, 64);
        let shifted = state.bvlshr(&shift_amount);
        let xored = state.bvxor(&shifted);
        
        let spare_shift = BV::from_u64(self.context, SPARE_BITS as u64, 64);
        let xsh_64 = xored.bvlshr(&spare_shift);
        let xsh = xsh_64.extract(31, 0);  // Extract lower 32 bits
        
        // Step 3: Apply variable right rotation
        // This is the most complex part since SMT doesn't have native variable rotation
        self.apply_right_rotation(&xsh, &rot)
    }
    
    /// Apply right rotation to a 32-bit value by a variable amount (0-31)
    /// Since Z3 doesn't have native variable rotation, we implement it using shifts and OR
    fn apply_right_rotation(&self, value: &BV<'ctx>, rotation_amount: &BV<'ctx>) -> BV<'ctx> {
        // Convert 5-bit rotation to 32-bit for arithmetic
        let rot_32 = rotation_amount.zero_ext(27); // Extend from 5 bits to 32 bits
        
        // Right rotation: rotr(value, n) = (value >> n) | (value << (32 - n))
        let thirty_two = BV::from_u64(self.context, 32, 32);
        
        // Right shift part: value >> rotation_amount
        let right_shifted = value.bvlshr(&rot_32);
        
        // Left shift part: value << (32 - rotation_amount)
        let left_shift_amount = thirty_two.bvsub(&rot_32);
        let left_shifted = value.bvshl(&left_shift_amount);
        
        // Combine with OR
        right_shifted.bvor(&left_shifted)
    }
    
    /// Add a range constraint for a data point
    /// This converts the 32-bit PCG output to a floating-point value and constrains it
    /// to be within the specified range
    fn add_range_constraint(&self, pcg_output: &BV<'ctx>, data_point: &DataPoint) {
        // Convert u32 PCG output to u32 range that would produce the observed value
        let (min_u32, max_u32) = data_point.valid_u32_range();
        
        // Create bit-vector constants for the range bounds
        let min_bound = BV::from_u64(self.context, min_u32 as u64, 32);
        let max_bound = BV::from_u64(self.context, max_u32 as u64, 32);
        
        // Add constraints: min_bound <= pcg_output <= max_bound
        self.solver.assert(&pcg_output.bvuge(&min_bound));
        self.solver.assert(&pcg_output.bvule(&max_bound));
    }
    
    /// Solve the reverse engineering problem for multiple constraints
    pub fn solve_constraints(&self, constraints: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
        // Track the current state and offset as we process constraints
        let mut current_state = self.initial_state.clone();
        let mut last_offset = 0u64;
        
        // Process each constraint in order of offset
        let mut sorted_constraints = constraints.to_vec();
        sorted_constraints.sort_by_key(|dp| dp.offset);
        
        for constraint in &sorted_constraints {
            // Advance state to the constraint's offset
            let steps_to_advance = constraint.offset - last_offset;
            if steps_to_advance > 0 {
                current_state = self.advance_state_by_steps(&current_state, steps_to_advance);
            }
            last_offset = constraint.offset;
            
            // Apply PCG output function to get the 32-bit value
            let pcg_output = self.apply_pcg_output(&current_state);
            
            // Add range constraint for this data point
            self.add_range_constraint(&pcg_output, constraint);
        }
        
        // Solve the constraints
        match self.solver.check() {
            SatResult::Sat => {
                let model = self.solver.get_model().ok_or_else(|| {
                    Error::from("Solver returned SAT but no model available".to_string())
                })?;
                
                // Extract the solution
                let initial_state_value = model.eval(&self.initial_state, true)
                    .ok_or_else(|| Error::from("Could not evaluate initial state from model".to_string()))?;
                
                // Convert Z3 bit-vector to u64
                let state_u64 = initial_state_value.as_u64()
                    .ok_or_else(|| Error::from("Could not convert state to u64".to_string()))?;
                
                // Convert state to seed (reverse the initial seed transformation)
                // In the original RNG: state = seed.wrapping_mul(MULTIPLIER).wrapping_sub(INITIAL_OFFSET)
                // So: seed = (state + INITIAL_OFFSET) / MULTIPLIER (with proper modular arithmetic)
                const INITIAL_OFFSET: u64 = 0x3d657cc62bc341e;
                
                // For now, we'll return the state directly and handle seed conversion later
                // TODO: Implement proper seed recovery
                let seed = state_u64 as i32; // Placeholder conversion
                
                Ok(vec![StateCandidate {
                    seed,
                    state: state_u64,
                }])
            }
            SatResult::Unsat => {
                Ok(vec![]) // No solution exists
            }
            SatResult::Unknown => {
                Err(Error::from("SMT solver returned unknown result (timeout or other issue)".to_string()))
            }
        }
    }
}

/// Main entry point for SMT-based reverse engineering
#[cfg(feature = "smt")]
pub fn smt_reverse_engineer(constraints: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    
    let reverse_engineer = SMTReverseEngineer::new(&ctx);
    reverse_engineer.solve_constraints(constraints)
}

/// Stub implementation when SMT feature is not enabled
#[cfg(not(feature = "smt"))]
pub fn smt_reverse_engineer(_constraints: &[DataPoint]) -> Result<Vec<StateCandidate>, Error> {
    Err(Error::from("SMT feature not enabled. Rebuild with --features smt".to_string()))
}

#[cfg(test)]
#[cfg(feature = "smt")]
mod tests {
    use super::*;
    
    #[test]
    fn test_smt_reverse_engineer_basic() {
        // Test basic SMT reverse engineering with a simple constraint
        let data_points = vec![
            DataPoint {
                offset: 0,
                range_min: 0.0,
                range_max: 1.0,
                observed: 0.5,
                name: "test".to_string(),
            }
        ];
        
        // This should find at least one solution
        let result = smt_reverse_engineer(&data_points);
        assert!(result.is_ok());
        
        let candidates = result.unwrap();
        assert!(!candidates.is_empty(), "Should find at least one candidate");
    }
    
    #[test] 
    fn test_lcg_advancement() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let re = SMTReverseEngineer::new(&ctx);
        
        // Test single step advancement
        let test_state = BV::from_u64(&ctx, 0x1234567890abcdef, 64);
        let advanced = re.advance_state_single(&test_state);
        
        // Verify we get a different state (not a comprehensive test, but basic sanity)
        re.solver.assert(&advanced._eq(&test_state).not());
        assert_eq!(re.solver.check(), SatResult::Sat);
    }
    
    #[test]
    fn test_pcg_output() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let re = SMTReverseEngineer::new(&ctx);
        
        // Test PCG output function
        let test_state = BV::from_u64(&ctx, 0x1234567890abcdef, 64);
        let output = re.apply_pcg_output(&test_state);
        
        // Verify output is 32 bits
        assert_eq!(output.get_size(), 32);
    }
}