use std::io::stdin;

use evaluator::{Evaluator, kayles::Kayles};

fn main() {
    println!("how many kayles does your game have?");
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("line could not be read");
    let kayles: usize = input
        .trim()
        .parse()
        .expect("could not be parsed to integer");
    let eval = Evaluator::new();
    eval.print_nimber_and_stats_of_game(Kayles { kayles });
}
