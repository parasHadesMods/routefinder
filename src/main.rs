mod rng;
use rng::SggPcg;
use rand::RngCore;
use structopt::StructOpt;
use rlua::{Lua, Result, Variadic, Value, Context, Function};
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(StructOpt)]
struct Cli {
  #[structopt(short = "s", long = "seed")]
  seed: i32,
  #[structopt(parse(from_os_str))]
  lua_path: std::path::PathBuf
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
                inner_lua_ctx.load(&import_file).exec()?;
                Ok(())
            })?;
            lua_ctx.globals().set("Import", import)?;
            lua_ctx.globals().set("Using", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnPreThingCreation", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnAnyLoad", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnUsed", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnActivationFinished", nop(lua_ctx)?)?;
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
            lua_ctx.globals().set("OnComeToRest", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnRamWeaponComplete", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnWeaponCharging", nop(lua_ctx)?)?;
            lua_ctx.globals().set("OnWeaponChargeCanceled", nop(lua_ctx)?)?;
            let get_time = lua_ctx.create_function(|_, _args: Variadic<Value>| {
                Ok(0)
            })?;
            lua_ctx.globals().set("GetTime", get_time)?;
            let get_language = lua_ctx.create_function(|_, _args: Variadic<Value>| {
                Ok("en")
            })?;
            lua_ctx.globals().set("GetLanguage", get_language)?;
            lua_ctx.globals().set("GetConfigOptionValue", nop(lua_ctx)?)?;
            let rs = |_, _args: Variadic<Value>| {
                let mut rng = shared_rng.borrow_mut(); 
                *rng = SggPcg::new(0 as u64);
                Ok(())
            };
            let randomseed = scope.create_function(rs)?;
            lua_ctx.globals().set("randomseed", randomseed)?;
            let mut main_path = args.lua_path.clone();
            main_path.push("Main.lua");
            let main = fs::read(main_path).expect("unable to read file");
            lua_ctx.load(&main).exec()?;
            let mut room_manager_path = args.lua_path.clone();
            room_manager_path.push("RoomManager.lua");
            let room_manager = fs::read(room_manager_path).expect("unable to read file");
            lua_ctx.load(&room_manager).exec()?;
            lua_ctx.globals().set("RouteFinderSeed", seed)?;
            lua_ctx.load(r#"RouteFinderFirstRoomChaos = PredictStartingRoomReward(RouteFinderSeed).FirstRoomChaos"#).exec()?;
            println!("{}", lua_ctx.globals().get::<_, bool>("RouteFinderFirstRoomChaos")?);
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
    return min.wrapping_add(bounded(rng, bound) as i32);
  } else {
    return min;
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

