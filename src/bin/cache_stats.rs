use std::time::Instant;

use evaluator::Evaluator;
use taking_game::builder::{Builder, get_known_games};
use taking_game::taking_game::TakingGame;

fn print_stats(label: &str, evaluator: &Evaluator<TakingGame>, started: Instant) {
    let (stubs, processing, done) = evaluator.get_cache_stats();
    println!(
        "{label}: stubs={stubs}, processing={processing}, done={done}, total={}, elapsed={:?}",
        evaluator.get_cache_size(),
        started.elapsed()
    );
}

fn main() {
    let evaluator = Evaluator::new();
    let started = Instant::now();

    for game in get_known_games() {
        let nimber = evaluator
            .get_nimber_by_parts(game.get_parts())
            .expect("known-game nimber computation should finish");
        assert!(game.check_nimber(nimber));
    }
    print_stats("known_games", &evaluator, started);

    for size in [4, 5, 6] {
        let parts = Builder::rect(size, size).build();
        let nimber = evaluator
            .get_nimber_by_parts(&parts)
            .expect("square rect nimber computation should finish");
        assert_eq!(nimber, 0, "{size}x{size} should have nimber 0");
        print_stats(&format!("after_rect_{size}x{size}"), &evaluator, started);
    }
}
