use taking_game::{builder::Builder, taking_game::TakingGame};

pub fn get_test_games() -> Vec<(TakingGame, Option<usize>, Option<bool>)> {
    //game , nimber , symmetry
    vec![
        (
            Builder::rect(1, 3).build_one().unwrap(),
            Some(3),
            Some(false),
        ),
        (
            Builder::rect(4, 1).build_one().unwrap(),
            Some(4),
            Some(false),
        ),
        (
            Builder::rect(100, 1).build_one().unwrap(),
            Some(100),
            Some(false),
        ),
        (
            Builder::rect(1, 101).build_one().unwrap(),
            Some(101),
            Some(false),
        ),
        (
            Builder::rect(2, 2).build_one().unwrap(),
            Some(0),
            Some(true),
        ),
        (
            Builder::rect(3, 3).build_one().unwrap(),
            Some(0),
            Some(false),
        ),
        (Builder::rect(3, 4).build_one().unwrap(), None, Some(false)),
        (
            Builder::rect(4, 4).build_one().unwrap(),
            Some(0),
            Some(true),
        ),
        (Builder::rect(5, 4).build_one().unwrap(), None, Some(false)),
        (
            Builder::hyper_cube(3, 2).build_one().unwrap(),
            Some(0),
            Some(true),
        ),
        (
            Builder::rect(5, 5).build_one().unwrap(),
            Some(0),
            Some(false),
        ),
    ]
}
