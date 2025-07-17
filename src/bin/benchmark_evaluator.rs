use evaluator::Evaluator;
use indicatif::ProgressIterator;
use taking_game::util::get_test_games;
use std::{
    thread::{self},
    time::{Duration, Instant},
};

fn main() {
    // Setup
    let mut eval = Evaluator::new();

    let cancel_flag = eval.get_cancel_flag();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(60));
        cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    });

    // Time measurement
    let start = Instant::now();

    for (game, maybe_expected_nimber, _) in get_test_games().into_iter().progress() {
        let maybe_nimber = eval.get_nimber(game);
        match (maybe_nimber, maybe_expected_nimber) {
            (None, _) => println!("nimber computation failed"),
            (Some(_), None) => (),
            (Some(nimber), Some(expected_nimber)) => {
                if nimber != expected_nimber {
                    println!("Error: expected{expected_nimber}, found {nimber}")
                }
            }
        }
    }

    let duration = start.elapsed();
    // Output
    println!("Time elapsed: {:.6?}", duration);
    println!("Cache entries: {:.6?}", eval.get_cache_size());
}


//cargo run --release --bin benchmark_evaluator
//Time elapsed: 2.227720s
//Cache entries: 4922
