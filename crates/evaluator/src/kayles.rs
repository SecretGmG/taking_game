use crate::Impartial;

const MAX_REMOVE: usize = 2;

#[derive(Debug, Eq, PartialEq, Hash, Clone, PartialOrd, Ord)]
pub struct Kayles {
    pub kayles: usize,
}

impl Impartial for Kayles {
    fn get_max_nimber(&self) -> Option<usize> {
        Some(self.kayles)
    }

    fn get_split_moves(&self) -> Vec<Vec<Kayles>> {
        let mut moves = vec![];

        for i in 1..=self.kayles.min(MAX_REMOVE) {
            moves.push(vec![Kayles {
                kayles: self.kayles - i,
            }]);
        }
        // i corresponds to the number of kayles to remove
        for i in 1..=self.kayles.saturating_sub(2).min(MAX_REMOVE) {
            // j corresponds to the number of kayles on the left heap
            for j in 1..=((self.kayles - i) / 2) {
                moves.push(vec![
                    Kayles { kayles: j },
                    Kayles {
                        kayles: self.kayles - i - j,
                    },
                ]);
            }
        }
        moves
    }
}
