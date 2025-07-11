use routefinder::rng::SggPcg;
use rand::RngCore;

fn main() {
    let known_seed = 12345i32;
    println!("# Test data generated from known seed: {}", known_seed);
    println!("# Format: name,offset,min,max,observed");
    
    let data_points = [
        ("chamber1", 0, 0.0, 100.0),
        ("chamber2", 5, 0.0, 1.0),
        ("chamber3", 10, -50.0, 50.0),
        ("chamber4", 15, 0.0, 10.0),
        ("chamber5", 20, 100.0, 200.0),
        ("chamber6", 25, 0.0, 1000.0),
        ("chamber7", 30, 0.0, 5.0),
    ];
    
    for (name, offset, min, max) in data_points.iter() {
        let mut rng = SggPcg::new(known_seed as u64);
        rng.advance(*offset);
        
        let value = rng.next_u32();
        let fraction = value as f64 / u32::MAX as f64;
        let scaled = fraction * (max - min) + min;
        let observed = (scaled * 100.0).round() / 100.0;
        
        println!("{},{},{},{},{:.2}", name, offset, min, max, observed);
    }
    
    println!("# Expected seed to find: {}", known_seed);
}