use rand::SeedableRng;
use rand_pcg::Pcg64;
use rand::RngCore;

fn main() {
    let mut rng = Pcg64::seed_from_u64(123);
    let value = rng.next_u32();
    println!("{}", value);
}
