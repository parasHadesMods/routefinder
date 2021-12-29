mod rng;
mod luabins;
mod read;
mod save;
use save::UncompressedSize;
use rng::SggPcg;
use rand::RngCore;
use structopt::StructOpt;
use rlua::{Lua, Variadic, Value, Context};
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use libm::ldexp;
use lz4;
use std::path::Path;
use std::sync::Arc;

#[derive(StructOpt)]
struct Cli {
  #[structopt(short = "s", long, env)]
  hades_scripts_dir: std::path::PathBuf,
  #[structopt(short = "f", long)]
  hades_save_file: std::path::PathBuf,
  #[structopt(parse(from_os_str))]
  script: std::path::PathBuf
}

#[derive(Debug)]
struct SimpleStringError {
  description: String
}

impl std::fmt::Display for SimpleStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::error::Error for SimpleStringError {
}

#[derive(Debug)]
enum Error {
  Lua {
    error: rlua::Error
  },
  IO {
    error: std::io::Error
  },
  SimpleString {
    error: SimpleStringError
  }
}

type Result<T> = core::result::Result<T, Error>;

impl From<rlua::Error> for Error {
  fn from(error: rlua::Error) -> Self {
    Error::Lua { error: error }
  }
}

impl From<std::io::Error> for Error {
  fn from(error: std::io::Error) -> Self {
    Error::IO { error: error }
  }
}

impl From<String> for Error {
  fn from(description: String) -> Self {
    Error::SimpleString {
      error: SimpleStringError {
        description: description
      }
    }
  }
}

impl From<Error> for rlua::Error {
  fn from(error: Error) -> Self {
     match error {
       Error::Lua { error } => error,
       Error::IO { error } => rlua::Error::ExternalError(Arc::new(error)),
       Error::SimpleString { error } => rlua::Error::ExternalError(Arc::new(error))
     }
  }
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let lua = unsafe {
      Lua::new_with_debug()
    };
    let shared_rng = Rc::new(RefCell::new(SggPcg::new(0)));
    lua.context(|lua_ctx| {
        lua_ctx.scope(|scope| -> Result<()> {
            // Engine callbacks etc.
            load_lua_file(lua_ctx, &"Engine.lua")?;
            // Hooks into the engine for RNG
            let randomseed = scope.create_function(|_, (o_seed, id): (Option<i32>, i32) | {
                let seed = match o_seed {
                    Some(s) => s,
                    None => 0
                };
                let mut rng = shared_rng.borrow_mut(); 
                *rng = SggPcg::new(seed as u64);
                Ok(id)
            })?;
            lua_ctx.globals().set("randomseed", randomseed)?;
            let randomint = scope.create_function(|_, (min, max, _id): (i32, i32, Value)| {
                let mut rng = shared_rng.borrow_mut();
                Ok(rand_int(&mut *rng, min, max))
            })?;
            lua_ctx.globals().set("randomint", randomint)?;
            let random = scope.create_function(|_, _args: Variadic<Value>| {
                let mut rng = shared_rng.borrow_mut();
                Ok(rand_double(&mut *rng))
            })?;
            lua_ctx.globals().set("random", random)?;
            let randomgaussian = scope.create_function(|_, _args: Variadic<Value>| {
                Ok(0.0) // only affects enemy ratios in encounters, but not number of waves or types
            })?;
            lua_ctx.globals().set("randomgaussian", randomgaussian)?;
            // Load lua files
            load_lua_file(lua_ctx, &args.hades_scripts_dir.join("Main.lua"))?;
            load_lua_file(lua_ctx, &args.hades_scripts_dir.join("RoomManager.lua"))?;
            // Load save file
            let save_file = read_file(args.hades_save_file)?;
            let lua_state_lz4 = save::read(&mut save_file.as_slice(), "save".to_string())?.lua_state_lz4;
            let lua_state = lz4::block::decompress(&lua_state_lz4.as_slice(), Some(save::HadesSaveV16::UNCOMPRESSED_SIZE))?;
            let save_data = luabins::load(&mut lua_state.as_slice(), lua_ctx, "luabins".to_string())?;
            lua_ctx.globals().set("RouteFinderSaveFileData", save_data)?;
            // put save file data into globals
            lua_ctx.load(r#"
                for _,savedValues in pairs(RouteFinderSaveFileData) do
                  for key, value in pairs(savedValues) do
                    if not SaveIgnores[key] then
                      _G[key] = value
                    end
                  end
                end
                "#).exec()?;
            // load and run script
            load_lua_file(lua_ctx, &args.script)
        })
    })
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

fn load_lua_file<'lua, P: AsRef<Path>>(lua_ctx: Context<'lua>, path: &P) -> Result<()> {
  let abs_path = path.as_ref().canonicalize()?;
  let parent_path = abs_path.parent().ok_or("No parent path".to_string())?;
  lua_ctx.scope(|scope| {
      let import = scope.create_function(|inner_lua_ctx, import_str: String| {
          let import_n1 = import_str.clone();
          let import_n2 = import_str.clone();
          let import_file = read_file(parent_path.join(import_n1))?;
          inner_lua_ctx.load(&import_file).set_name(&import_n2)?.exec()
      })?;
      let file = read_file(path)?;
      lua_ctx.globals().set("Import", import)?;
      lua_ctx.load(&file).exec()?;
      Ok(())
  })
}

fn rand_int(rng: &mut SggPcg, min: i32, max: i32) -> i32 {
  if max > min {
    let bound = (max as u32).wrapping_sub(min as u32).wrapping_add(1);
    min.wrapping_add(bounded(rng, bound) as i32)
  } else {
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

/* Rough stab at how random gaussian generate works in the Hades code.
   - seems to be an independant SggPcg used only for gaussians
   - the gaussian pcg isn't reseeded on RandomSeed or reset on RandomSynchronize
   - it does seem to be reset to the same value every time when starting the game

struct GaussState {
  has_value: bool,
  value: f64
}

fn rand_gauss(rng: &mut SggPcg, state: &mut GaussState) -> f64 {
  if state.has_value {
      state.has_value = false;
      state.value
   } else {
      let mut u: f64 = 0.0;
      let mut v: f64 = 0.0;
      let mut s: f64 = 0.0;

      // Box-Muller, polar form
      while s >= 1.0 || s == 0.0 {
        u = 2.0 * rand_double(rng) - 1.0;
        v = 2.0 * rand_double(rng) - 1.0;
        s = u * u + v * v;
      }

      let f = libm::sqrt(-2.0 * libm::log(s) / s);
      state.has_value = true; // keep for next call
      state.value = f * u;
      f * v
  }
}
*/
