use indicatif::ProgressIterator;
use std::time::Instant;
mod helper;
use helper::get_test_games;

fn main() {
    // Time measurement
    let start = Instant::now();

    for (game, _, maybe_expected_symmetry) in get_test_games().into_iter().progress() {
        if let Some(expected_symmetry) = maybe_expected_symmetry {
            if game.find_symmetry().is_some() != expected_symmetry {
                println!("finding symmetry failed");
            }
        }
    }

    let duration = start.elapsed();
    // Output
    println!("Time elapsed: {:.6?}", duration);
}

//cargo run --bin benchmark_symmetry_finder --no-default-features -- --optimized
//Time elapsed: 5ms

//cargo run --bin benchmark_symmetry_finder -- --optimized
//Time elapsed: 118.244500ms
