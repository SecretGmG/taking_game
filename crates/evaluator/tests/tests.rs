#![cfg(test)]
use evaluator::{Evaluator, Impartial, kayles::Kayles};

#[test]
fn test_simple_kayle_nimbers() {
    let nimbers: Vec<usize> = vec![0, 1, 2, 3];
    let eval: Evaluator<Kayles> = Evaluator::new();

    // test the later half of the nimbers, to make sure that the evaluator can handle inputs even if
    // smaller nimbers arent already cached.
    (nimbers.len() / 2..nimbers.len()).for_each(|i| {
        assert_eq!(nimbers[i], eval.get_nimber(&Kayles { kayles: i }).unwrap());
    });
}

#[test]
fn test_aperiodic_kayles_nimbers() {
    // taken from the OEIS A002186
    let nimbers: Vec<usize> = vec![
        0, 1, 2, 3, 1, 4, 3, 2, 1, 4, 2, 6, 4, 1, 2, 7, 1, 4, 3, 2, 1, 4, 6, 7, 4, 1, 2, 8, 5, 4,
        7, 2, 1, 8, 6, 7, 4, 1, 2, 3, 1, 4, 7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1, 4, 2, 7,
        4, 1, 2, 8, 1, 4, 7, 2, 1, 8, 6, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4,
        7, 2, 1, 8, 2, 7, 4, 1, 2, 8, 1, 4, 7, 2, 1,
    ];
    let eval: Evaluator<Kayles> = Evaluator::new();

    // test the later half of the nimbers, to make sure that the evaluator can handle inputs even if
    // smaller nimbers arent already cached.
    (nimbers.len() / 2..nimbers.len()).for_each(|i| {
        assert_eq!(nimbers[i], eval.get_nimber(&Kayles { kayles: i }).unwrap());
    });
}
#[test]
fn test_cancellation() {
    use std::thread;
    use std::time::Duration;

    // Start with a fresh evaluator and evaluate a complex Kayles position
    let eval = Evaluator::new();
    let target = Kayles { kayles: 200 };

    // Spawn a thread to simulate cancellation after a short delay
    let cloned_eval = eval.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(5));
        cloned_eval.stop();
    });

    // Attempt to evaluate, expecting it to be cancelled
    let result = eval.get_nimber(&target);
    assert_eq!(result, None, "Evaluation should be cancelled");
    eval.resume();

    // Try again — should continue from cached state
    let result2 = eval.get_nimber(&target);
    assert!(
        result2.is_some(),
        "Evaluation should complete after resuming"
    );

    // Cache should now be valid; validate result against a clean evaluator
    let fresh_eval = Evaluator::new();
    let expected = fresh_eval.get_nimber(&target).unwrap();
    assert_eq!(
        result2.unwrap(),
        expected,
        "Nimber after cancellation-resume should match fresh evaluation"
    );

    // Do another cancellation-resume cycle on a new, larger input
    let cloned_eval = eval.clone();
    let new_target = Kayles { kayles: 300 };
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(5));
        cloned_eval.stop();
    });

    eval.resume();
    let result3 = eval.get_nimber(&new_target);
    assert_eq!(result3, None, "Second cancellation should also interrupt");

    let cloned_eval = eval.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(5));
        cloned_eval.stop();
    });
    eval.resume();
    let result4 = eval.get_nimber(&new_target);

    assert_eq!(result4, None, "Third cancellation should still interrupt");
    eval.resume();
    let result5 = eval.get_nimber(&new_target).unwrap();

    let fresh_eval2 = Evaluator::new();
    let expected2 = fresh_eval2.get_nimber(&new_target).unwrap();
    assert_eq!(
        result5, expected2,
        "Result after second resume should match fresh computation"
    );
}

#[derive(PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
struct ZeroGame {}

impl Impartial for ZeroGame {
    fn get_split_moves(&self) -> Vec<Vec<Self>> {
        panic!("moves should never be generated for this game");
    }
    fn get_max_nimber(&self) -> Option<usize> {
        Some(0)
    }
}

#[test]
fn test_early_cancellation() {
    let e = Evaluator::new();
    assert_eq!(e.get_nimber(&ZeroGame {}), Some(0));
}
