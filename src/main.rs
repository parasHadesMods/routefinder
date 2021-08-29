mod rng;
mod luabins;
mod read;
mod save;
use rng::SggPcg;
use rand::RngCore;
use structopt::StructOpt;
use rlua::{Lua, Result, Variadic, Value, Context, Function, Table};
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use libm::ldexp;

#[derive(StructOpt)]
struct Cli {
  #[structopt(short = "s", long = "seed")]
  seed: i32,
  #[structopt(parse(from_os_str))]
  lua_path: std::path::PathBuf,
  #[structopt(short = "w", long = "weapon")]
  weapon: String,
  #[structopt(short = "i", long = "aspect_index")]
  aspect_index: i32,
  #[structopt(short = "f", long = "save_file")]
  save_file: std::path::PathBuf
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let lua = unsafe {
      Lua::new_with_debug()
    };
    let shared_rng = Rc::new(RefCell::new(SggPcg::new(args.seed as u64)));
    let seed = args.seed;
    let parent_path = args.lua_path.clone();
    lua.context(|lua_ctx| {
        lua_ctx.scope(|scope| {
            let import = scope.create_function(|inner_lua_ctx, import_str: String| {
                println!("{}", import_str);
                let import_file = fs::read(parent_path.clone().join(import_str)).expect("unable to read file");
                let cleaned_file = if import_file.starts_with("\u{feff}".as_bytes()) {
                  &import_file[3..]
                } else {
                  &import_file
                };
                inner_lua_ctx.load(cleaned_file).exec()?;
                Ok(())
            })?;
            lua_ctx.globals().set("Import", import)?;
            // Callbacks from the engine that we will never use; can just be nops
            lua_ctx.globals().set("Using", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnPreThingCreation", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnAnyLoad", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnUsed", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnActivationFinished", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnAutoUseFailed", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnMenuOpened", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnMenuCloseFinished", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnPlayerMoveStarted", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnPlayerMoveStopped", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnControlPressed", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnActiveUseTarget", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnActiveUseTargetLost", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnMouseOver", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnMouseOff", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnControlHotSwap", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnMusicMarker", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnKeyPressed", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnWeaponFired", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnWeaponTriggerRelease", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnComeToRest", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnRamWeaponComplete", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnWeaponCharging", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnWeaponChargeCanceled", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnWeaponFailedToFire", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnPerfectChargeWindowEntered", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnHit", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnProjectileReflect", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnProjectileBlock", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnProjectileDeath", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnDodge", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnSpawn", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnHealed", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnCollisionReaction", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnCollisionEnd", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnObstacleCollision", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnUnitCollision", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnMovementReaction", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnAllegianceFlip", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnTouchdown", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnEffectApply", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnEffectCleared", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnEffectStackDecrease", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnEffectDelayedKnockbackForce", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnEffectCanceled", nop(lua_ctx)?)?;
            lua_ctx.globals().set("DebugPrint", nop(lua_ctx)?)?;
            lua_ctx.globals().set("DebugAssert", nop(lua_ctx)?)?;
            // Time is not relevant, it's only used to set the fresh file seed and
            // we will overwrite that.
            let get_time = lua_ctx.create_function(|_, _args: Variadic<Value>| {
                Ok(0)
            })?;
            lua_ctx.globals().set("GetTime", get_time)?;
            // Use english for localization.
            let get_language = lua_ctx.create_function(|_, _args: Variadic<Value>| {
                Ok("en")
            })?;
            lua_ctx.globals().set("GetLanguage", get_language)?;
            // For now we don't care about these config options, but we might later.
            let get_config_option_value = lua_ctx.create_function(|_, table: Table| {
                if table.get::<&str, String>("Name")? == "DebugRNGSeed" {
                    Ok(Value::Integer(0))
                } else {
                    Ok(Value::Nil)
                }
            })?;
            lua_ctx.globals().set("GetConfigOptionValue", get_config_option_value)?;
            // Hooks into the engine for RNG
            let randomseed = scope.create_function(|_, (o_seed, _id): (Option<i32>, Value) | {
                let seed = match o_seed {
                    Some(s) => s,
                    None => 0
                };
                let mut rng = shared_rng.borrow_mut(); 
                *rng = SggPcg::new(seed as u64);
                Ok(())
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
            // Load lua files
            let mut main_path = args.lua_path.clone();
            main_path.push("Main.lua");
            let main = fs::read(main_path).expect("unable to read file");
            lua_ctx.load(&main).exec()?;
            let mut room_manager_path = args.lua_path.clone();
            room_manager_path.push("RoomManager.lua");
            let room_manager = fs::read(room_manager_path).expect("unable to read file");
            lua_ctx.load(&room_manager).exec()?;
            println!("Done Loading Scripts");
            let save_file = fs::read(args.save_file).expect("unable to read file");
            let cleaned_save = if save_file.starts_with("\u{feff}".as_bytes()) {
              &save_file[3..]
            } else {
              &save_file
            };
            match luabins::load_value(&mut save_file.as_slice(), lua_ctx, "save".to_string()) {
              Ok(r) => match r {
                Value::String(s) => println!("{}", s.to_str().unwrap()),
                _ => println!("other")
              },
              Err(s) => {
                println!("{}", s)
              }
            };
            println!("Done Loading Save");
            /*for r in results {
              match r {
                Value::Table(t) => println!("table"),
                _ => println!("unknown")
              };
            };*/
            lua_ctx.globals().set("RouteFinderSeed", seed)?;
            lua_ctx.load(r#"RandomInit()"#).exec()?;
            println!("Prediction");
            // Set equipped weapon from cmd line
            lua_ctx.globals().set("RouteFinderWeapon", args.weapon);
            lua_ctx.globals().set("RouteFinderAspectIndex", args.aspect_index);
            lua_ctx.load(r#"
                if not GameState then
                  GameState =  {}
                end
                if not GameState.MetaUpgrades then
                  GameState.MetaUpgrades = {}
                end
                if not GameState.ActiveMutators then
                  GameState.ActiveMutators = {}
                end
                if not GameState.LastWeaponUpgradeData then
                  GameState.LastWeaponUpgradeData = {}
                end
                GameState.LastWeaponUpgradeData[RouteFinderWeapon] = { Index = RouteFinderAspectIndex }
                if not CurrentRun then
                  CurrentRun = {}
                end
                if not CurrentRun.Hero then
                  CurrentRun.Hero = {}
                end
                if not CurrentRun.Hero.Weapons then
                  CurrentRun.Hero.Weapons = {}
                end
                CurrentRun.Hero.Weapons[RouteFinderWeapon] = true
                --NextSeeds[1] = RouteFinderSeed
                --RandomSynchronize()
                RouteFinderRoomReward = PredictStartingRoomReward(RouteFinderSeed)
                function deep_print(t, indent)
                  local indentString = ""
                  for i = 1, indent do
                    indentString = indentString .. "  "
                  end
                  for k,v in pairs(t) do
                    if type(v) == "table" then
                      print(indentString..k)
                      deep_print(v, indent + 1)
                    else
                      print(indentString..k, v)
                    end
                  end
                end
                deep_print(RouteFinderRoomReward, 0)
                "#).exec()?;
            Ok(())
        })?;
        Ok(())
    })
}

fn nop<'lua>(lua_ctx: Context<'lua>) -> Result<Function<'lua>> {
   lua_ctx.create_function(|_, _args: Variadic<Value>| {
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
