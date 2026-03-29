use itertools::Itertools;

use crate::{builder::Builder, taking_game::TakingGame};

pub struct KnownGame {
    parts: Vec<TakingGame>,
    symmetry: Option<bool>,
    nimber: Option<usize>,
}

impl KnownGame {
    pub fn from_builder(b: Builder) -> Self {
        Self {
            parts: b.build(),
            symmetry: None,
            nimber: None,
        }
    }
    pub fn nimber(mut self, nimber: usize) -> Self {
        self.nimber = Some(nimber);
        self
    }
    pub fn symmetric(mut self) -> Self {
        self.symmetry = Some(true);
        self
    }
    pub fn not_symmetric(mut self) -> Self {
        self.symmetry = Some(false);
        self
    }
    pub fn check_nimber(&self, nimber: usize) -> bool {
        self.nimber.map(|n| n == nimber).unwrap_or(true)
    }
    pub fn check_symmetry(&self) -> bool {
        if let Some(expected_symmetry) = self.symmetry {
            let mut parts = self.parts.clone();
            parts.sort();
            // Pair-cancel identical adjacent components in O(n):
            // components appearing an odd number of times remain unpaired.
            let all_symmetric = parts
                .iter()
                .dedup_with_count()
                .filter(|(count, _)| count % 2 == 1)
                .all(|(_, p)| p.find_symmetry().is_some());
            expected_symmetry == all_symmetric
        } else {
            true
        }
    }
    pub fn get_parts(&self) -> &[TakingGame] {
        &self.parts
    }
}

pub fn get_known_games() -> Vec<KnownGame> {
    vec![
        KnownGame::from_builder(Builder::rect(1, 3))
            .nimber(3)
            .not_symmetric(),
        KnownGame::from_builder(Builder::rect(4, 1))
            .nimber(4)
            .not_symmetric(),
        KnownGame::from_builder(Builder::heap(100))
            .nimber(100)
            .not_symmetric(),
        KnownGame::from_builder(Builder::heap(101))
            .nimber(101)
            .not_symmetric(),
        KnownGame::from_builder(Builder::heap(16).sum(Builder::heap(8).sum(Builder::heap(7))))
            .nimber(31)
            .not_symmetric(),
        KnownGame::from_builder(Builder::rect(2, 2))
            .nimber(0)
            .symmetric(),
        KnownGame::from_builder(Builder::rect(3, 3))
            .nimber(0)
            .not_symmetric(),
        KnownGame::from_builder(Builder::rect(3, 4)).not_symmetric(),
        KnownGame::from_builder(Builder::rect(4, 4))
            .nimber(0)
            .symmetric(),
        KnownGame::from_builder(Builder::rect(5, 4)).not_symmetric(),
        KnownGame::from_builder(Builder::hyper_cube(3, 2))
            .nimber(0)
            .symmetric(),
        KnownGame::from_builder(Builder::rect(5, 5))
            .nimber(0)
            .not_symmetric(),
        KnownGame::from_builder(Builder::hyper_tetrahedron(10)).not_symmetric(),
    ]
}
