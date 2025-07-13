mod error;
mod gui;
mod luabins;
mod read;
mod reverse_rng;
mod rng;
mod save;
use clap::{Parser, Subcommand};
use libm::ldexp;
use lz4;
use mlua::{Lua, LuaOptions, Table, Value, Variadic};
use rand::RngCore;
use rng::SggPcg;
use save::UncompressedSize;
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Lua script with game simulation
    Run {
        script: std::path::PathBuf,

        /// Save file to use as starting point
        #[arg(short = 'f', long, value_name = "FILE")]
        save_file: PathBuf,

        /// Hades Scripts directory
        #[arg(short = 's', long, value_name = "FILE")]
        scripts_dir: PathBuf,

        /// Set Lua variables (format: variable=value)
        #[arg(long = "lua-var", value_name = "VAR=VALUE")]
        lua_vars: Vec<String>,
    },
    /// RNG operations
    Rng {
        #[command(subcommand)]
        rng_command: RngCommands,
    },
    /// Reverse engineer RNG state from observation data
    ReverseRng {
        /// Input file containing data points
        input_file: PathBuf,
        
        /// Method to use for reverse engineering
        #[arg(long, default_value = "brute-force")]
        method: String,
        
    },
    /// Launch GUI for route analysis
    Gui,
}

#[derive(Subcommand)]
enum RngCommands {
    /// Set RNG seed
    SetSeed {
        /// Seed value (signed integer, will be converted to unsigned)
        seed: i64,
    },
    /// Advance RNG by specified steps
    Advance {
        /// Number of steps to advance (signed integer, will be converted to unsigned)
        steps: i64,
    },
}

type Result<T, E = error::Error> = core::result::Result<T, E>;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { script, save_file, scripts_dir, lua_vars } => {
            run_script(script, save_file, scripts_dir, lua_vars)
        }
        Commands::Rng { rng_command } => {
            handle_rng_command(rng_command)
        }
        Commands::ReverseRng { input_file, method } => {
            handle_reverse_rng_command(input_file, method)
        }
        Commands::Gui => {
            handle_gui_command()
        }
    }
}

fn run_script(route_finder_script: PathBuf, save_file_path: PathBuf, hades_scripts_dir: PathBuf, lua_vars: Vec<String>) -> Result<()> {
    let lua = unsafe { Lua::unsafe_new_with(mlua::StdLib::ALL, LuaOptions::new()) };

    let shared_rng = Rc::new(RefCell::new(SggPcg::new(0)));

    // Load save file
    let save_file = read_file(save_file_path)?;
    let lua_state_lz4 = save::read(&mut save_file.as_slice(), "save".to_string())?.lua_state_lz4;
    let lua_state = lz4::block::decompress(
        &lua_state_lz4.as_slice(),
        Some(save::HadesSaveV16::UNCOMPRESSED_SIZE),
    )?;

    lua.scope(|scope: &mlua::Scope| -> Result<(), mlua::Error> {
        let getmetatable = scope.create_function(|_, table: Table| {
            Ok(table.get_metatable())
        })?;
        lua.globals().set("getmetatable", getmetatable)?;

        // Engine callbacks etc.
        let _ = load_lua_file(&lua, &"Engine.lua")?;

        // Hooks into the engine for RNG
        let randomseed = scope.create_function(|_, (o_seed, id): (Option<i32>, i32)| {
            let seed = match o_seed {
                Some(s) => s,
                None => 0,
            };
            let mut rng = shared_rng.borrow_mut();
            *rng = SggPcg::new(seed as u64);
            Ok(id)
        })?;
        lua.globals().set("randomseed", randomseed)?;

        let randomint = scope.create_function(|_, (min, max, _id): (i32, i32, Value)| {
            let mut rng = shared_rng.borrow_mut();
            Ok(rand_int(&mut *rng, min, max))
        })?;
        lua.globals().set("randomint", randomint)?;

        let random = scope.create_function(|_, _args: Variadic<Value>| {
            let mut rng = shared_rng.borrow_mut();
            Ok(rand_double(&mut *rng))
        })?;
        lua.globals().set("random", random)?;

        let randomgaussian = scope.create_function(|_, _args: Variadic<Value>| {
            Ok(0.0) // only affects enemy ratios in encounters, but not number of waves or types
        })?;
        lua.globals().set("randomgaussian", randomgaussian)?;


        // Load lua files
        load_lua_file(&lua, &hades_scripts_dir.join("Main.lua"))?;
        load_lua_file(&lua, &hades_scripts_dir.join("RoomManager.lua"))?;

        let save_data = luabins::load(&lua, &mut lua_state.as_slice(), "luabins".to_string()).unwrap();

        lua.globals().set("RouteFinderSaveFileData", save_data)?;

        // put save file data into globals
        lua.load(
            r#"
            for _,savedValues in pairs(RouteFinderSaveFileData) do
                for key, value in pairs(savedValues) do
                if not SaveIgnores[key] then
                    _G[key] = value
                end
                end
            end
            "#,
        )
        .exec()?;

        // Set custom Lua variables from command line
        for lua_var in &lua_vars {
            if let Some((var_name, var_value)) = lua_var.split_once('=') {
                // Try to parse as different types
                if let Ok(int_val) = var_value.parse::<i64>() {
                    lua.globals().set(var_name, int_val)?;
                } else if let Ok(float_val) = var_value.parse::<f64>() {
                    lua.globals().set(var_name, float_val)?;
                } else if var_value.eq_ignore_ascii_case("true") {
                    lua.globals().set(var_name, true)?;
                } else if var_value.eq_ignore_ascii_case("false") {
                    lua.globals().set(var_name, false)?;
                } else {
                    // Treat as string
                    lua.globals().set(var_name, var_value)?;
                }
            } else {
                eprintln!("Warning: Invalid lua-var format '{}'. Expected format: variable=value", lua_var);
            }
        }

        // load and run script
        match load_lua_file(&lua, &route_finder_script) {
            Ok(()) => {}
            Err(err) => {
                println!("Error: {}", err.to_string());
            }
        };
        Ok(())
    })?;

    Ok(())
}

fn handle_rng_command(rng_command: RngCommands) -> Result<()> {
    const STATE_FILE: &str = ".rng.json";

    match rng_command {
        RngCommands::SetSeed { seed } => {
            let rng = SggPcg::new(seed as u64);
            println!("RNG seed set to: {}", seed);

            let mut preview_rng: SggPcg = rng.clone();
            for i in 0..3 {
                let value = preview_rng.next_u32();
                println!("  {}: {}", i, value);
            }

            // Save state to file
            if let Err(e) = rng.save_to_file(STATE_FILE) {
                eprintln!("Warning: Failed to save RNG state: {}", e);
            }
        }
        RngCommands::Advance { steps } => {
            // Load existing state from file, or create new if file doesn't exist
            let mut rng = match SggPcg::load_from_file(STATE_FILE) {
                Ok(loaded_rng) => {
                    println!("Loaded RNG state from {}", STATE_FILE);
                    loaded_rng
                }
                Err(_) => {
                    println!("No existing RNG state found, creating new with seed 0");
                    SggPcg::new(0)
                }
            };

            println!("Current RNG state before advancing:");
            let mut preview_rng: SggPcg = rng.clone();
            for i in 0..3 {
                let value = preview_rng.next_u32();
                println!("  {}: {}", i, value);
            }

            rng.advance(steps as u64);

            println!("RNG advanced by {} steps", steps);
            println!("Values after advance:");
            preview_rng = rng.clone();
            for i in 0..3 {
                let value = preview_rng.next_u32();
                println!("  {}: {}", i, value);
            }

            // Save state to file
            if let Err(e) = rng.save_to_file(STATE_FILE) {
                eprintln!("Warning: Failed to save RNG state: {}", e);
            }
        }
    }
    
    Ok(())
}

fn handle_gui_command() -> Result<()> {
    use druid::{AppLauncher, WindowDesc, EventCtx, Event, Env, WidgetExt};
    use druid::widget::Controller;
    use gui::{AppState, build_ui, ui::{BUTTON_PRESSED, CALCULATE_PRESSED}};
    use std::fs::File;
    use std::io::Write;
    use std::process::Command;
    
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
                    if let Some(button_name) = cmd.get(BUTTON_PRESSED) {
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
            data.text_output.push_str(&format!("/range {} {} 200 {} {}\n", 
                button_press.name, button_press.offset, low, high));
        }
        
        // Execute reverse-rng
        data.text_output.push_str("\n=== Running Reverse RNG ===\n");
        let reverse_rng_output = Command::new("cargo")
            .args(&["+nightly", "run", "--release", "--features", "simd_nightly", "--", "reverse-rng", temp_file_path])
            .output()
            .or_else(|_| {
                // Fallback to scalar version if SIMD fails
                Command::new("cargo")
                    .args(&["run", "--release", "--", "reverse-rng", temp_file_path])
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
            .args(&["run", "--release", "--", "run", &data.script_file, 
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
        Err(error::Error::from(format!("Could not extract seed from reverse-rng output: {}", output)))
    }
    
    let main_window = WindowDesc::new(build_ui().controller(AppController))
        .title("Hades Route Finder")
        .window_size((1000.0, 700.0));
    
    let initial_state = AppState::default();
    
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .map_err(|e| error::Error::from(format!("GUI launch failed: {}", e)))?;
    
    Ok(())
}

fn handle_reverse_rng_command(input_file: PathBuf, method: String) -> Result<()> {
    use reverse_rng::data_point;
    
    println!("Reverse engineering RNG state from: {:?}", input_file);
    println!("Method: {}", method);
    
    // Parse input file
    let data_points = data_point::parse_input_file(&input_file)?;
    println!("Loaded {} data points", data_points.len());
    
    // Perform reverse engineering based on method
    let start_time = std::time::Instant::now();
    
    let candidates = match method.as_str() {
        "brute-force" => {
            println!("Using brute force method...");
            #[cfg(feature = "simd")]
            {
                reverse_rng::simd_search::find_original_state_simd(&data_points)?
            }
            #[cfg(not(feature = "simd"))]
            {
                reverse_rng::search::find_original_state(&data_points)?
            }
        }
        _ => {
            return Err(error::Error::from(format!("Unknown method: {}. Use 'brute-force'", method)));
        }
    };
    
    let elapsed = start_time.elapsed();
    println!("Reverse engineering completed in {:.2}s", elapsed.as_secs_f64());
    
    if candidates.is_empty() {
        println!("No valid RNG states found that match all data points");
    } else {
        println!("Found {} candidate state(s):", candidates.len());
        for (i, candidate) in candidates.iter().enumerate() {
            println!("  Candidate {}: seed {}", i + 1, candidate.seed);
        }
    }
    
    Ok(())
}

const BYTE_ORDER_MARK: &[u8] = "\u{feff}".as_bytes();
fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let file = fs::read(path)?;
    if file.starts_with(BYTE_ORDER_MARK) {
        Ok(file[3..].to_vec())
    } else {
        Ok(file.to_vec())
    }
}

fn load_lua_file<'lua, P: AsRef<Path>>(lua: &Lua, path: &P) -> Result<(), mlua::Error> {
    let abs_path = path.as_ref().canonicalize()?;
    let parent_path = abs_path.parent().ok_or("No parent path".to_string()).unwrap();

    lua.scope(|scope| {
        let import = scope.create_function(|inner_lua, import_str: String| {
            let import_n1 = import_str.clone();
            let import_n2 = import_str.clone();
            let import_file = read_file(parent_path.join(import_n1))?;
            inner_lua.load(&import_file).set_name(&import_n2).exec()
        })?;

        let file = read_file(path)?;
        lua.globals().set("Import", import)?;
        lua.load(&file).exec()?;
        Ok(())
    })
}

fn rand_int(rng: &mut SggPcg, min: i32, max: i32) -> i32 {
    // println!("rand_int min {} max {}", min, max);
    if max > min {
        let bound = (max as u32).wrapping_sub(min as u32).wrapping_add(1);
        min.wrapping_add(bounded(rng, bound) as i32)
    } else {
        rng.next_u32(); // advance and ignore result (to keep in sync)
        min
    }
}

fn bounded(rng: &mut SggPcg, bound: u32) -> u32 {
    let threshold = (u32::MAX - bound + 1) % bound;

    loop {
        let r = rng.next_u32();
        if r >= threshold {
            return r % bound;
        }
    }
}

fn rand_double(rng: &mut SggPcg) -> f64 {
    ldexp(rng.next_u32() as f64, -32)
}
