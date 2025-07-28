use druid::{Data, Lens};

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub assault: String,
    pub grasp: String,
    pub ambush: String,
    pub favor: String,
    pub lunge: String,
    pub soul: String,
    pub strike: String,
    pub eclipse: String,
    pub affluence: String,
    pub shot: String,
    pub flourish: String,
    pub defiance: String,
    pub text_output: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            assault: String::new(),
            grasp: "1".to_string(), // Prefilled, uneditable 
            ambush: String::new(),
            favor: String::new(),
            lunge: String::new(),
            soul: String::new(),
            strike: String::new(),
            eclipse: String::new(),
            affluence: "Not needed.".to_string(), // Prefilled, uneditable
            shot: String::new(),
            flourish: String::new(),
            defiance: "1".to_string(), // Prefilled, uneditable
            text_output: "Enter percentage values for each field.\n".to_string(),
        }
    }
}

impl AppState {
    pub fn is_valid(&self) -> bool {
        self.is_assault_valid() && 
        self.is_ambush_valid() && 
        self.is_favor_valid() && 
        self.is_lunge_valid() && 
        self.is_soul_valid() && 
        self.is_strike_valid() && 
        self.is_eclipse_valid() &&
        self.is_shot_valid() &&
        self.is_flourish_valid()
    }
    
    pub fn is_assault_valid(&self) -> bool {
        if let Ok(value) = self.assault.parse::<f64>() {
            value >= 30.0 && value <= 50.0
        } else {
            false
        }
    }
    
    pub fn is_ambush_valid(&self) -> bool {
        if let Ok(value) = self.ambush.parse::<f64>() {
            value >= 60.0 && value <= 80.0
        } else {
            false
        }
    }
    
    pub fn is_favor_valid(&self) -> bool {
        if let Ok(value) = self.favor.parse::<f64>() {
            value >= 11.0 && value <= 20.0
        } else {
            false
        }
    }
    
    pub fn is_lunge_valid(&self) -> bool {
        if let Ok(value) = self.lunge.parse::<f64>() {
            value >= 40.0 && value <= 60.0
        } else {
            false
        }
    }
    
    pub fn is_soul_valid(&self) -> bool {
        if let Ok(value) = self.soul.parse::<f64>() {
            value >= 30.0 && value <= 40.0
        } else {
            false
        }
    }
    
    pub fn is_strike_valid(&self) -> bool {
        if let Ok(value) = self.strike.parse::<f64>() {
            value >= 30.0 && value <= 40.0
        } else {
            false
        }
    }
    
    pub fn is_eclipse_valid(&self) -> bool {
        if let Ok(value) = self.eclipse.parse::<f64>() {
            value >= 50.0 && value <= 80.0
        } else {
            false
        }
    }
    
    pub fn is_shot_valid(&self) -> bool {
        if let Ok(value) = self.shot.parse::<f64>() {
            value >= 30.0 && value <= 40.0
        } else {
            false
        }
    }
    
    pub fn is_flourish_valid(&self) -> bool {
        if let Ok(value) = self.flourish.parse::<f64>() {
            value >= 30.0 && value <= 60.0
        } else {
            false
        }
    }
    
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    
    pub fn generate_range_format(&self) -> String {
        let mut output = String::new();
        
        if let Ok(assault_val) = self.assault.parse::<f64>() {
            output.push_str(&format!("assault,{},{:.2},{:.2},{:.2}\n", 14, 0.3, 0.5, assault_val / 100.0));
        }
        
        if let Ok(ambush_val) = self.ambush.parse::<f64>() {
            output.push_str(&format!("ambush,{},{:.2},{:.2},{:.2}\n", 18, 0.6, 0.8, ambush_val / 100.0));
        }
                
        if let Ok(favor_val) = self.favor.parse::<f64>() {
            output.push_str(&format!("favor,{},{:.2},{:.2},{:.2}\n", 20, 0.11, 0.2, favor_val / 100.0));
        }
        
        if let Ok(lunge_val) = self.lunge.parse::<f64>() {
            output.push_str(&format!("lunge,{},{:.2},{:.2},{:.2}\n", 22, 0.4, 0.6, lunge_val / 100.0));
        }
        
        if let Ok(soul_val) = self.soul.parse::<f64>() {
            output.push_str(&format!("soul,{},{:.2},{:.2},{:.2}\n", 24, 0.3, 0.4, soul_val / 100.0));
        }
        
        if let Ok(strike_val) = self.strike.parse::<f64>() {
            output.push_str(&format!("strike,{},{:.2},{:.2},{:.2}\n", 26, 0.3, 0.4, strike_val / 100.0));
        }
        
        if let Ok(eclipse_val) = self.eclipse.parse::<f64>() {
            output.push_str(&format!("eclipse,{},{:.2},{:.2},{:.2}\n", 28, 0.5, 0.8, eclipse_val / 100.0));
        }
        
        if let Ok(shot_val) = self.shot.parse::<f64>() {
            output.push_str(&format!("shot,{},{:.2},{:.2},{:.2}\n", 32, 0.3, 0.4, shot_val / 100.0));
        }
        
        if let Ok(flourish_val) = self.flourish.parse::<f64>() {
            output.push_str(&format!("flourish,{},{:.2},{:.2},{:.2}\n", 34, 0.3, 0.6, flourish_val / 100.0));
        }
        
        output
    }
}