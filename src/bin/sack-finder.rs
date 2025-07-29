use routefinder::error::Error;
use routefinder::sack_finder::{AppState, build_ui, ui::{CALCULATE_PRESSED, CLEAR_PRESSED}};
use druid::{AppLauncher, WindowDesc, EventCtx, Event, Env, WidgetExt, ExtEventSink, Target};
use druid::widget::Controller;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};

type Result<T, E = Error> = core::result::Result<T, E>;

// Custom events for background thread communication
pub const OUTPUT_UPDATE: druid::Selector<String> = druid::Selector::new("output-update");
pub const CALCULATION_COMPLETE: druid::Selector<()> = druid::Selector::new("calculation-complete");
pub const CALCULATION_ERROR: druid::Selector<String> = druid::Selector::new("calculation-error");

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
                Event::Command(cmd) if cmd.is(CALCULATE_PRESSED) => {
                    if let Err(e) = execute_calculate(data, ctx.get_external_handle()) {
                        data.text_output = format!("Error: {}", e);
                    }
                }
                Event::Command(cmd) if cmd.is(CLEAR_PRESSED) => {
                    data.clear();
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
        .title("Sack Finder")
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
    let range_data = data.generate_range_format();
    
    std::thread::spawn(move || {
        execute_calculate_background(range_data, event_sink);
    });
    
    Ok(())
}

fn execute_calculate_background(range_data: String, event_sink: ExtEventSink) {
    // Generate range format file
    let temp_file_path = "/tmp/sack_finder_range_input.txt";
    let mut file = match File::create(temp_file_path) {
        Ok(f) => f,
        Err(e) => {
            event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
            return;
        }
    };
    
    if let Err(e) = file.write_all(range_data.as_bytes()) {
        event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
        return;
    }
    
    if let Err(e) = file.flush() {
        event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
        return;
    }
    
    event_sink.submit_command(OUTPUT_UPDATE, "=== Generated Range Format ===\n".to_string(), Target::Auto).ok();
    event_sink.submit_command(OUTPUT_UPDATE, range_data, Target::Auto).ok();
    
    // Execute reverse-rng with the generated file
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
    let reader = std::io::BufReader::new(stdout);
    
    use std::io::BufRead;
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
    
    if let Err(e) = reverse_rng_child.wait() {
        event_sink.submit_command(CALCULATION_ERROR, e.to_string(), Target::Auto).ok();
        return;
    }
    
    // Clean up temp file and signal completion
    std::fs::remove_file(temp_file_path).ok();
    event_sink.submit_command(CALCULATION_COMPLETE, (), Target::Auto).ok();
}