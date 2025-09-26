use evaluator::Impartial;
use indicatif::ProgressIterator;
use std::time::Instant;
use taking_game::util::get_test_games;

fn main() {
    // Setup
    let start = Instant::now();

    for (game, _, _) in get_test_games().into_iter().progress() {
        _ = game.get_split_moves();
    }

    let duration = start.elapsed();
    // Output
    println!("Time elapsed: {:.6?}", duration);
}

