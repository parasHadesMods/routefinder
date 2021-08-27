mod rng;
use rng::SggPcg;
use rand::RngCore;
use structopt::StructOpt;
use rlua::Lua;
use std::fs;

#[derive(StructOpt)]
struct Cli {
  #[structopt(short = "s", long = "seed")]
  seed: i32,
  #[structopt(parse(from_os_str))]
  lua_path: std::path::PathBuf
}

fn main() {
    let args = Cli::from_args();
    let lua = Lua::new();
    let mut rng = SggPcg::new(args.seed as u64);
    let file = fs::read(args.lua_path).expect("unable to read file");
    let seed = args.seed;
    lua.context(|lua_ctx| {
        lua_ctx.load(&file).exec().expect("unable to exec lua file");
        lua_ctx.globals().set("RouteFinderSeed", seed);
        lua_ctx.load(r#"RouteFinderFirstRoomChaos = PredictStartingRoomReward(RouteFinderSeed).FirstRoomChaos"#)
            .exec().expect("unable to predict rewards!");
        println!("{}", lua_ctx.globals().get::<_, bool>("RouteFinderFirstRoomChaos").expect("not working"));
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

