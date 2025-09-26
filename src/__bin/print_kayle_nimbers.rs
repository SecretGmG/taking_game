use std::io::stdin;

use evaluator::Evaluator;
use taking_game::Constructor;

fn main() {
    let eval = Evaluator::new();
    println!("how many kayle nimbers do you want to see?");
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("line could not be read");
    let max: usize = input
        .trim()
        .parse()
        .expect("could not be parsed to integer");
    for i in 0..max {
        let g = Constructor::kayles(i).build_one();
        println!("{}:{}", i, eval.get_nimber(&g).unwrap());
        println!("Cache size {:?}", eval.get_cache_size())
    }
}
