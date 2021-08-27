use rand::RngCore;
mod rng;
use rng::SggPcg;

fn main() {
    let mut rng = SggPcg::new(19986);
    println!("{}", rand_int(&mut rng, 1, 10));
    println!("{}", rand_int(&mut rng, 1, 10));
    println!("{}", rand_int(&mut rng, 1, 10));
    println!("{}", rand_int(&mut rng, 1, 10));
    println!("{}", rand_int(&mut rng, 1, 10));
    println!("{}", rand_int(&mut rng, 1, 10));
    println!("{}", rand_int(&mut rng, 1, 10));
    println!("{}", rand_int(&mut rng, 1, 10));
    println!("{}", rand_int(&mut rng, 1, 10));
    println!("{}", rand_int(&mut rng, 1, 10));
}

fn rand_int(rng: &mut SggPcg, min: u32, max: u32) -> u32 {
  if max > min {
    return min + bounded(rng, max - min + 1);
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

