use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use evaluator::{Evaluator, Impartial};
use taking_game::builder::get_known_games;

fn bench_nimber_computation(c: &mut Criterion) {
    let known_games = get_known_games();
    let mut group = c.benchmark_group("nimber computation");
    group.sample_size(20);
    group.bench_function("nimber computation", |b| {
        b.iter(|| {
            let evaluator = Evaluator::new(); // one evaluator per iteration
            for k in &known_games {
                let nimber = black_box(evaluator.get_nimber_by_parts(k.get_parts()));
                assert!(k.check_nimber(nimber.unwrap()));
            }
        })
    });
}

fn bench_symmetry(c: &mut Criterion) {
    let known_games = get_known_games();

    c.bench_function("symmetry", |b| {
        b.iter(|| {
            for k in &known_games {
                assert!(black_box(k.check_symmetry()));
            }
        })
    });
}

fn bench_move_generation(c: &mut Criterion) {
    let known_games = get_known_games();

    c.bench_function("move generation", |b| {
        b.iter(|| {
            for k in &known_games {
                _ = black_box(k.get_parts().iter().map(|p| p.get_split_moves()));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_nimber_computation,
    bench_symmetry,
    bench_move_generation
);
criterion_main!(benches);
