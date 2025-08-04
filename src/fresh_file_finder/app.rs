use std::sync::Arc;
use druid::{Data, Lens};

#[derive(Clone, Data, PartialEq)]
pub enum Mode {
    BounceRoute,
    OffRoute,
}

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub offset: u32,
    pub button_history: Arc<Vec<ButtonPress>>,
    pub text_output: String,
    pub save_file_path: String,
    pub scripts_dir_path: String,
    pub script_file: String,
    pub found_seed: Option<i32>,
    
    // New fields for off-route recovery
    pub mode: Mode,
    pub off_route_chamber: String,
    pub offset_off_by: i32,
    pub original_script_file: String,
}

#[derive(Clone, Data)]
pub struct ButtonPress {
    pub name: String,
    pub offset: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            offset: 6,
            button_history: Arc::new(Vec::new()),
            text_output: "Current offset: 6\n".to_string(),
            save_file_path: "FreshFile.sav".to_string(),
            scripts_dir_path: "~/workspace/hades/routefinder/assets/Scripts/".to_string(),
            script_file: "RouteFreshFileIncrementally.lua".to_string(),
            found_seed: None,
            mode: Mode::BounceRoute,
            off_route_chamber: String::new(),
            offset_off_by: 0,
            original_script_file: "RouteFreshFileIncrementally.lua".to_string(),
        }
    }
}

impl AppState {
    pub fn add_button_press(&mut self, button_name: String) {
        self.offset += 1;
        let button_press = ButtonPress {
            name: button_name.clone(),
            offset: self.offset,
        };
        
        let mut history = (*self.button_history).clone();
        history.push(button_press);
        self.button_history = Arc::new(history);
        
        self.text_output.push_str(&format!("Button pressed: {}\nCurrent offset: {}\n", button_name, self.offset));
    }
    
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    
    pub fn get_button_range(button_name: &str) -> (u32, u32) {
        match button_name {
            "Bottom" => (0, 4),
            "Low" => (3, 7),
            "Middle" => (6, 10),
            "High" => (9, 13),
            "Top" => (12, 16),
            _ => (0, 0),
        }
    }
    
    pub fn enter_off_route_mode(&mut self) {
        self.original_script_file = self.script_file.clone();
        self.mode = Mode::OffRoute;
        self.script_file = "RouteFreshFileRecovery.lua".to_string();
        self.off_route_chamber = String::new();
        self.offset_off_by = 0;
    }
    
    pub fn exit_off_route_mode(&mut self) {
        self.mode = Mode::BounceRoute;
        self.script_file = self.original_script_file.clone();
        self.off_route_chamber = String::new();
        self.offset_off_by = 0;
    }
    
    pub fn adjust_offset_off_by(&mut self, delta: i32) {
        self.offset_off_by += delta;
    }
    
    pub fn validate_chamber_input(input: &str) -> bool {
        input.chars().all(|c| c.is_ascii_digit()) && !input.is_empty()
    }
}