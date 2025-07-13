use std::sync::Arc;
use druid::{Data, Lens};

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub offset: u32,
    pub button_history: Arc<Vec<ButtonPress>>,
    pub text_output: String,
    pub save_file_path: String,
    pub scripts_dir_path: String,
    pub script_file: String,
}

#[derive(Clone, Data)]
pub struct ButtonPress {
    pub name: String,
    pub offset: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            offset: 7,
            button_history: Arc::new(Vec::new()),
            text_output: "Current offset: 7".to_string(),
            save_file_path: "FreshFile.sav".to_string(),
            scripts_dir_path: "~/legendary/Hades/Content/Scripts/".to_string(),
            script_file: "RouteFreshFile.lua".to_string(),
        }
    }
}

impl AppState {
    pub fn add_button_press(&mut self, button_name: String) {
        let button_press = ButtonPress {
            name: button_name,
            offset: self.offset,
        };
        
        let mut history = (*self.button_history).clone();
        history.push(button_press);
        self.button_history = Arc::new(history);
        
        self.offset += 1;
        self.text_output = format!("Current offset: {}", self.offset);
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
}