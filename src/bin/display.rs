use evaluator::Evaluator;
use taking_game::builder::Builder;

fn main() {
    let builder = Builder::rect(7, 7);

    let parts = builder.build();
    for part in &parts {
        println!("{}", part);
    }

    let evaluator = Evaluator::new();
    evaluator.print_nimber_and_stats_of_games(parts);
}
