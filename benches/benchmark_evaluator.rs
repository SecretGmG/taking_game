use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use evaluator::{Evaluator, Impartial};
use taking_game::builder::{Builder, get_known_games};

const RECT_CASES: &[(usize, usize)] = &[(5, 5), (6, 6), (7, 7), (6, 7), (4, 8)];
const QUICK_SAMPLE_SIZE: usize = 10;
const QUICK_WARM_UP: Duration = Duration::from_millis(500);
const QUICK_MEASUREMENT: Duration = Duration::from_secs(3);
const SLOW_MEASUREMENT: Duration = Duration::from_secs(8);

fn build_rect_parts(x: usize, y: usize) -> Vec<taking_game::taking_game::TakingGame> {
    Builder::rect(x, y).build()
}

fn bench_known_game_nimbers(c: &mut Criterion) {
    let known_games = get_known_games();
    let mut group = c.benchmark_group("nimber/known_games");
    group.sample_size(QUICK_SAMPLE_SIZE);
    group.warm_up_time(QUICK_WARM_UP);
    group.measurement_time(SLOW_MEASUREMENT);

    group.bench_function("all", |b| {
        b.iter(|| {
            let evaluator = Evaluator::new();
            for game in &known_games {
                let nimber = evaluator
                    .get_nimber_by_parts(black_box(game.get_parts()))
                    .expect("known-game nimber computation should finish");
                assert!(game.check_nimber(nimber));
            }
            black_box(evaluator.get_cache_stats())
        })
    });

    group.finish();
}

fn bench_finished_square_nimbers(c: &mut Criterion) {
    let mut group = c.benchmark_group("nimber/square_rects");
    group.sample_size(QUICK_SAMPLE_SIZE);
    group.warm_up_time(QUICK_WARM_UP);
    group.measurement_time(SLOW_MEASUREMENT);

    for size in [4, 5, 6] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let parts = build_rect_parts(size, size);
            b.iter(|| {
                let evaluator = Evaluator::new();
                let nimber = evaluator
                    .get_nimber_by_parts(black_box(&parts))
                    .expect("square rect nimber computation should finish");
                assert_eq!(nimber, 0);
                black_box(evaluator.get_cache_stats())
            });
        });
    }

    group.finish();
}

fn bench_rect_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("rect/build");
    group.sample_size(QUICK_SAMPLE_SIZE);
    group.warm_up_time(QUICK_WARM_UP);
    group.measurement_time(QUICK_MEASUREMENT);

    group.bench_function("rect_suite", |b| {
        b.iter(|| {
            let mut total_parts = 0;
            for &(x, y) in RECT_CASES {
                total_parts += Builder::rect(x, y).build().len();
            }
            black_box(total_parts)
        });
    });

    group.finish();
}

fn bench_root_move_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("moves/root_rects");
    group.sample_size(QUICK_SAMPLE_SIZE);
    group.warm_up_time(QUICK_WARM_UP);
    group.measurement_time(SLOW_MEASUREMENT);

    let rects: Vec<_> = RECT_CASES
        .iter()
        .map(|&(x, y)| {
            let parts = build_rect_parts(x, y);
            assert_eq!(parts.len(), 1, "rectangles should start connected");
            ((x, y), parts)
        })
        .collect();

    group.bench_function("rect_suite", |b| {
        b.iter(|| {
            let mut total_moves = 0;
            for (_, parts) in &rects {
                total_moves += parts[0].get_split_moves().len();
            }
            black_box(total_moves)
        });
    });

    let parts_7x7 = build_rect_parts(7, 7);
    group.bench_function(BenchmarkId::new("rect", "7x7"), |b| {
        b.iter(|| {
            let moves = parts_7x7[0].get_split_moves();
            black_box(moves.len())
        });
    });

    group.finish();
}

fn bench_symmetry(c: &mut Criterion) {
    let mut group = c.benchmark_group("symmetry/rects");
    group.sample_size(QUICK_SAMPLE_SIZE);
    group.warm_up_time(QUICK_WARM_UP);
    group.measurement_time(QUICK_MEASUREMENT);

    let games: Vec<_> = RECT_CASES
        .iter()
        .map(|&(x, y)| Builder::rect(x, y).build_one().unwrap())
        .collect();

    group.bench_function("rect_suite", |b| {
        b.iter(|| {
            let symmetry_count = games
                .iter()
                .filter(|game| black_box(game.find_symmetry()).is_some())
                .count();
            black_box(symmetry_count)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_known_game_nimbers,
    bench_finished_square_nimbers,
    bench_rect_building,
    bench_root_move_generation,
    bench_symmetry
);
criterion_main!(benches);
