use std::io::stdin;

use evaluator::Evaluator;
use taking_game::Constructor;


fn main(){
    let mut eval = Evaluator::new();
        println!("how many kayle nimbers do you want to see?");
        let mut input = String::new();
        stdin().read_line(&mut input).expect("line could not be read");
        let max : usize = input.trim().parse().expect("could not be parsed to integer");

    for i in 0..max {
        let g = Constructor::triangle(i).build();
        println!("{}:{}", i, eval.get_nimber(g).unwrap());
    }
}