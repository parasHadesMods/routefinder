mod rng;
use rng::SggPcg;
use rand::RngCore;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
  #[structopt(short = "s", long = "seed")]
  seed: i32,
  #[structopt(short = "n", long = "offset")]
  offset: u32
}

fn main() {
    let args = Cli::from_args();
    let mut rng = SggPcg::new(args.seed as u64);
    if args.offset > 1 {
      rng.advance((args.offset - 1).into());
    }
    println!("{}", rand_int(&mut rng, -2147483647, 2147483646));
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

