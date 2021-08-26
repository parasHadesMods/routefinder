use rand::SeedableRng;
use rand_pcg::Pcg32;
use rand::Rng;
use rand::RngCore;

fn main() {
    let mut rng = Pcg32::seed_from_u64(19986);
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

fn rand_int(rng: &mut Pcg32, min: u32, max: u32) -> u32 {
  if max > min {
    return min + bounded(rng, max - min + 1);
  } else {
    return min;
  }
}

fn bounded(rng: &mut Pcg32, bound: u32) -> u32 {
  let threshold = (u32::MAX - bound + 1) % bound;

  loop {
    let r = rng.next_u32();
    if r >= threshold {
      return r % bound;
    }
  }
}

