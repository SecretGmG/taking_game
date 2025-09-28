use evaluator::Evaluator;
use taking_game::builder::Builder;

fn main() {
    let builder = Builder::hyper_cube(2, 3)
        .sum(Builder::hyper_cube(3, 2))
        .connect_unit_to_all();

    let parts = builder.build();
    for part in &parts {
        println!("{}", part);
    }

    let evaluator = Evaluator::new();
    evaluator.print_nimber_and_stats_of_games(parts);
}
