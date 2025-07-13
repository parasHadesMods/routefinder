use routefinder::error::Error;
use routefinder::gui::{AppState, build_ui, ui::{BUTTON_PRESSED, CALCULATE_PRESSED}, app::ButtonPress};
use druid::{AppLauncher, WindowDesc, EventCtx, Event, Env, WidgetExt, ExtEventSink, Target, Selector};
use druid::widget::Controller;
use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use std::process::{Command, Stdio};

type Result<T, E = Error> = core::result::Result<T, E>;

fn expand_tilde(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home_dir) = std::env::var("HOME").ok() {
            path.replacen('~', &home_dir, 1)
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    }
}

// Custom events for background thread communication
pub const OUTPUT_UPDATE: Selector<String> = Selector::new("output-update");
pub const CALCULATION_COMPLETE: Selector<()> = Selector::new("calculation-complete");
pub const CALCULATION_ERROR: Selector<String> = Selector::new("calculation-error");

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
                    if let Err(e) = execute_calculate(data, ctx.get_external_handle()) {
                        data.text_output = format!("Error: {}", e);
                    }
                }
                Event::Command(cmd) if cmd.is(OUTPUT_UPDATE) => {
                    if let Some(text) = cmd.get::<String>(OUTPUT_UPDATE) {
                        data.text_output.push_str(&text);
                    }
                }
                Event::Command(cmd) if cmd.is(CALCULATION_COMPLETE) => {
                    // Calculation finished successfully
                }
                Event::Command(cmd) if cmd.is(CALCULATION_ERROR) => {
                    if let Some(error_msg) = cmd.get::<String>(CALCULATION_ERROR) {
                        data.text_output.push_str(&format!("\nError: {}\n", error_msg));
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

fn execute_calculate(data: &mut AppState, event_sink: ExtEventSink) -> Result<()> {
    // Clear previous output
    data.text_output.clear();
    
    // Clone data needed for background thread
    let button_history = (*data.button_history).clone();
    let script_file = data.script_file.clone();
    let save_file_path = data.save_file_path.clone();
    let scripts_dir_path = data.scripts_dir_path.clone();
    let offset = data.button_history.last().map(|bp| bp.offset + 1).unwrap_or(data.offset as u32) as i32;
    
    std::thread::spawn(move || {
        execute_calculate_background(button_history, script_file, save_file_path, scripts_dir_path, offset, event_sink);
    });
    
    Ok(())
}

fn execute_calculate_background(
    button_history: Vec<ButtonPress>,
    script_file: String,
    save_file_path: String,
    scripts_dir_path: String,
    offset: i32,
    event_sink: ExtEventSink,
) {
    // Generate reverse-rng input file
    let temp_file_path = "/tmp/routefinder_reverse_rng_input.txt";
    let mut file = match File::create(temp_file_path) {
        Ok(f) => f,
        Err(e) => {
            event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
            return;
        }
    };
    
    for button_press in button_history.iter() {
        let (low, high) = AppState::get_button_range(&button_press.name);
        if let Err(e) = writeln!(file, "/range {} {} 16 {} {}", 
            button_press.name, button_press.offset, low, high) {
            event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
            return;
        }
    }
    if let Err(e) = file.flush() {
        event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
        return;
    }
    
    event_sink.submit_command(OUTPUT_UPDATE, "\n=== Reverse RNG Input ===\n".to_string(), Target::Auto).ok();
    for button_press in button_history.iter() {
        let (low, high) = AppState::get_button_range(&button_press.name);
        event_sink.submit_command(OUTPUT_UPDATE, format!("/range {} {} 16 {} {}\n", 
            button_press.name, button_press.offset, low, high), Target::Auto).ok();
    }
    
    // Execute reverse-rng with streaming output
    event_sink.submit_command(OUTPUT_UPDATE, "\n=== Running Reverse RNG ===\n".to_string(), Target::Auto).ok();
    
    let mut reverse_rng_child = match Command::new("cargo")
        .args(&["+nightly", "run", "--release", "--features", "simd_nightly", "--bin", "routefinder", "--", "reverse-rng", temp_file_path])
        .stdout(Stdio::piped())
        .spawn() {
        Ok(child) => child,
        Err(e) => {
            event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
            return;
        }
    };
    
    let stdout = reverse_rng_child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut reverse_rng_output = String::new();
    
    for line in reader.lines() {
        match line {
            Ok(line_str) => {
                reverse_rng_output.push_str(&line_str);
                reverse_rng_output.push('\n');
                event_sink.submit_command(OUTPUT_UPDATE, format!("{}\n", line_str), Target::Auto).ok();
            }
            Err(e) => {
                event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
                return;
            }
        }
    }
    
    if let Err(e) = reverse_rng_child.wait() {
        event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
        return;
    }
    
    // Parse seed from output
    let seed = match extract_seed_from_output(&reverse_rng_output) {
        Ok(s) => s,
        Err(e) => {
            event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
            return;
        }
    };
    event_sink.submit_command(OUTPUT_UPDATE, format!("\n=== Found Seed: {} ===\n", seed), Target::Auto).ok();
    
    // Execute route analysis with streaming output
    event_sink.submit_command(OUTPUT_UPDATE, "\n=== Route Analysis ===\n".to_string(), Target::Auto).ok();
    
    let expanded_scripts_dir = expand_tilde(&scripts_dir_path);
    let mut route_child = match Command::new("cargo")
        .args(&["run", "--release", "--bin", "routefinder", "--", "run", &script_file, 
               "--save-file", &save_file_path,
               "--scripts-dir", &expanded_scripts_dir,
               "--lua-var", &format!("AthenaSeed={}", seed),
               "--lua-var", &format!("AthenaOffset={}", offset)])
        .stdout(Stdio::piped())
        .spawn() {
        Ok(child) => child,
        Err(e) => {
            event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
            return;
        }
    };
    
    let stdout = route_child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    
    for line in reader.lines() {
        match line {
            Ok(line_str) => {
                event_sink.submit_command(OUTPUT_UPDATE, format!("{}\n", line_str), Target::Auto).ok();
            }
            Err(e) => {
                event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
                return;
            }
        }
    }
    
    if let Err(e) = route_child.wait() {
        event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
        return;
    }
    
    // Clean up temp file and signal completion
    std::fs::remove_file(temp_file_path).ok();
    event_sink.submit_command(CALCULATION_COMPLETE, (), Target::Auto).ok();
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