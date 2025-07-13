use routefinder::error::Error;
use routefinder::gui::{AppState, build_ui, ui::{BUTTON_PRESSED, CALCULATE_PRESSED}};
use druid::{AppLauncher, WindowDesc, EventCtx, Event, Env, WidgetExt};
use druid::widget::Controller;
use std::fs::File;
use std::io::Write;
use std::process::Command;

type Result<T, E = Error> = core::result::Result<T, E>;

fn main() -> Result<()> {
    struct AppController;
    
    impl<W: druid::Widget<AppState>> Controller<AppState, W> for AppController {
        fn event(
            &mut self,
            child: &mut W,
            ctx: &mut EventCtx,
            event: &Event,
            data: &mut AppState,
            env: &Env,
        ) {
            match event {
                Event::Command(cmd) if cmd.is(BUTTON_PRESSED) => {
                    if let Some(button_name) = cmd.get::<String>(BUTTON_PRESSED) {
                        data.add_button_press(button_name.clone());
                    }
                }
                Event::Command(cmd) if cmd.is(CALCULATE_PRESSED) => {
                    if let Err(e) = execute_calculate(data) {
                        data.text_output = format!("Error: {}", e);
                    }
                }
                _ => {}
            }
            child.event(ctx, event, data, env);
        }
    }
    
    let main_window = WindowDesc::new(build_ui().controller(AppController))
        .title("Fresh File Finder")
        .window_size((1000.0, 700.0));
    
    let initial_state = AppState::default();
    
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .map_err(|e| Error::from(format!("GUI launch failed: {}", e)))?;
    
    Ok(())
}

fn execute_calculate(data: &mut AppState) -> Result<()> {
    // Generate reverse-rng input file
    let temp_file_path = "/tmp/routefinder_reverse_rng_input.txt";
    let mut file = File::create(temp_file_path)?;
    
    for button_press in data.button_history.iter() {
        let (low, high) = AppState::get_button_range(&button_press.name);
        writeln!(file, "/range {} {} 16 {} {}", 
            button_press.name, button_press.offset, low, high)?;
    }
    file.flush()?;
    
    data.text_output.push_str("\n=== Reverse RNG Input ===\n");
    for button_press in data.button_history.iter() {
        let (low, high) = AppState::get_button_range(&button_press.name);
        data.text_output.push_str(&format!("/range {} {} 16 {} {}\n", 
            button_press.name, button_press.offset, low, high));
    }
    
    // Execute reverse-rng
    data.text_output.push_str("\n=== Running Reverse RNG ===\n");
    let reverse_rng_output = Command::new("cargo")
        .args(&["+nightly", "run", "--release", "--features", "simd_nightly", "--bin", "routefinder", "--", "reverse-rng", temp_file_path])
        .output()
        .or_else(|_| {
            // Fallback to scalar version if SIMD fails
            Command::new("cargo")
                .args(&["run", "--release", "--bin", "routefinder", "--", "reverse-rng", temp_file_path])
                .output()
        })?;
        
    let reverse_rng_stdout = String::from_utf8_lossy(&reverse_rng_output.stdout);
    data.text_output.push_str(&reverse_rng_stdout);
    
    // Parse seed from output
    let seed = extract_seed_from_output(&reverse_rng_stdout)?;
    data.text_output.push_str(&format!("\n=== Found Seed: {} ===\n", seed));
    
    // Execute route analysis
    let offset = data.button_history.last().map(|bp| bp.offset + 1).unwrap_or(data.offset);
    let route_output = Command::new("cargo")
        .args(&["run", "--release", "--bin", "routefinder", "--", "run", &data.script_file, 
               "--save-file", &data.save_file_path,
               "--scripts-dir", &data.scripts_dir_path,
               "--lua-var", &format!("AthenaSeed={}", seed),
               "--lua-var", &format!("AthenaOffset={}", offset)])
        .output()?;
        
    let route_stdout = String::from_utf8_lossy(&route_output.stdout);
    data.text_output.push_str("\n=== Route Analysis ===\n");
    data.text_output.push_str(&route_stdout);
    
    // Clean up temp file
    std::fs::remove_file(temp_file_path).unwrap_or(());
    
    Ok(())
}

fn extract_seed_from_output(output: &str) -> Result<i32> {
    for line in output.lines() {
        if line.contains("Candidate") && line.contains("seed") {
            if let Some(seed_part) = line.split("seed ").nth(1) {
                if let Ok(seed) = seed_part.trim().parse::<i32>() {
                    return Ok(seed);
                }
            }
        }
    }
    Err(Error::from(format!("Could not extract seed from reverse-rng output: {}", output)))
}