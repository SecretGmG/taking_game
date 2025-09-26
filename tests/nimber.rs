use evaluator::Evaluator;
use taking_game::builder::Builder;

#[test]
fn unit_nimber() {
    let g = Builder::unit().build_one().unwrap();
    let evaluator = Evaluator::new();
    let nimber = evaluator.get_nimber(&g);
    assert_eq!(nimber, Some(1));
}
#[test]
fn heap_nimbers() {
    let evaluator = Evaluator::new();
    for i in 1..=100 {
        let g = Builder::heap(i).build_one().unwrap();
        let nimber = evaluator.get_nimber(&g);
        assert_eq!(nimber, Some(i));
    }
    assert_eq!(evaluator.get_cache_size(), 100);
}
const KAYLE_NIMBERS: [[usize; 2]; 10] = [
    [1, 1],
    [2, 2],
    [3, 3],
    [4, 1],
    [5, 4],
    [7, 2],
    [8, 1],
    [9, 4],
    [10, 2],
    [15, 7],
];
#[test]
fn kayle_nimbers() {
    let evaluator = Evaluator::new();
    for [i, ecpected_nimber] in KAYLE_NIMBERS {
        let g = Builder::kayles(i).build_one().unwrap();
        let nimber = evaluator.get_nimber(&g);
        assert_eq!(nimber, Some(ecpected_nimber));
    }
}
#[test]
fn squares() {
    let evaluator = Evaluator::new();
    for i in 2..=6 {
        let g = Builder::rect(i, i).build_one().unwrap();
        let nimber = evaluator.get_nimber(&g);
        assert_eq!(nimber, Some(0));
    }
}
