use std::time::Instant;
use rand::SeedableRng;
use number_place_rs::puzzle::presets::preset_16x16;
use number_place_rs::generator::{generate, GeneratorConstraints};

fn main() {

    let puzzle = preset_16x16();
    let constraints = GeneratorConstraints::default();
    // warm-up
    {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        let _ = generate(&puzzle, &constraints, &mut rng);
    }
    let n = 1u32;
    let start = Instant::now();
    for i in 0..n {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(i as u64 + 1);
        let r = generate(&puzzle, &constraints, &mut rng);
        assert!(r.is_ok(), "generation failed at seed {i}");
    }
    let total = start.elapsed();
    println!("{n} runs: total={total:.2?}, avg={:.2?}", total / n);
}
