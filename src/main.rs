mod error;
mod luabins;
mod read;
mod reverse_rng;
mod rng;
mod save;
mod smt_reverse_engineer;
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
        
        /// Timeout in milliseconds for SMT solver
        #[arg(long, default_value = "10000")]
        timeout_ms: u64,
    },
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
        Commands::Run { script, save_file, scripts_dir } => {
            run_script(script, save_file, scripts_dir)
        }
        Commands::Rng { rng_command } => {
            handle_rng_command(rng_command)
        }
        Commands::ReverseRng { input_file, method, timeout_ms } => {
            handle_reverse_rng_command(input_file, method, timeout_ms)
        }
    }
}

fn run_script(route_finder_script: PathBuf, save_file_path: PathBuf, hades_scripts_dir: PathBuf) -> Result<()> {
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

fn handle_reverse_rng_command(input_file: PathBuf, method: String, timeout_ms: u64) -> Result<()> {
    use reverse_rng::data_point;
    
    println!("Reverse engineering RNG state from: {:?}", input_file);
    println!("Method: {}", method);
    if method == "smt" {
        println!("SMT Solver timeout: {}ms", timeout_ms);
    }
    
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
        "smt" => {
            println!("Using SMT solver method...");
            smt_reverse_engineer::smt_reverse_engineer(&data_points)?
        }
        _ => {
            return Err(error::Error::from(format!("Unknown method: {}. Use 'brute-force' or 'smt'", method)));
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
