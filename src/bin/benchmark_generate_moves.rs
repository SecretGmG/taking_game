use indicatif::ProgressIterator;
use evaluator::Impartial;
use taking_game::util::get_test_games;
use std::time::Instant;

fn main() {
    // Setup
    let start = Instant::now();

    for (game, _, _) in get_test_games().into_iter().progress() {
        _ = game.get_moves();
    }

    let duration = start.elapsed();
    // Output
    println!("Time elapsed: {:.6?}", duration);
}