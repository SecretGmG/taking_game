use super::DenseTakingGame;
use evaluator::Impartial;
use itertools::Itertools;

impl Impartial for DenseTakingGame {
    /// Return the maximum possible nimber for this game.
    ///
    /// If the game has a symmetry, the nimber is 0. Otherwise, it is
    /// bounded above by the number of nodes.
    fn get_max_nimber(&self) -> Option<usize> {
        match self.find_symmetry() {
            Some(_) => Some(0),
            None => Some(self.nodes.len()),
        }
    }

    /// Generate move splits by considering one representative
    /// from each structural equivalence class of edges.
    fn get_split_moves(&self) -> Vec<Vec<DenseTakingGame>> {
        if self.hyperedges.is_empty() {
            return vec![];
        }
        self.edge_structure_partitions
            .iter()
            .rev()
            .skip(1) // the last index in the partition is always = hyperedges.len()
            .flat_map(|e| self.get_moves_of_edge(self.hyperedges[*e]))
            .collect()
    }
}

impl DenseTakingGame {
    /// Generate all moves resulting from removing nodes belonging
    /// to a given hyperedge, partitioned by structural equivalence.
    fn get_moves_of_edge(
        &self,
        hyperedge: u128,
    ) -> impl Iterator<Item = Vec<DenseTakingGame>> + '_ {
        let partition_masks = self.get_partition_masks();

        let partitioned_edge = partition_masks
            .iter()
            .map(|partition_mask| hyperedge & *partition_mask)
            .filter(|mask| *mask != 0);

        let nodes_to_remove_per_part = partitioned_edge.map(|mut part| {
            let mut nodes_to_remove = Vec::with_capacity(part.count_ones() as usize + 1);
            let mut mask = 1;
            nodes_to_remove.push(0); //at first the `do nothing` move
            while part != 0 {
                if part & mask != 0 {
                    nodes_to_remove.push(part);
                    part &= !mask;
                }
                mask <<= 1;
            }
            nodes_to_remove
        });

        let masks = nodes_to_remove_per_part
            .multi_cartesian_product()
            .map(|nodes_to_remove| nodes_to_remove.into_iter().fold(0, |a, b| a | b))
            .skip(1);
        masks.map(|mask| self.with_nodes_removed(mask))
    }

    fn get_partition_masks(&self) -> Vec<u128> {
        let mut partition_masks = Vec::new();

        // Convert each node partition into a bitmask
        for p in self.node_structure_partitions.windows(2) {
            let mut mask = 0u128;
            for n in p[0]..p[1] {
                mask |= 1 << n;
            }
            partition_masks.push(mask);
        }
        partition_masks
    }

    /// Return new game states with the given nodes removed.
    ///
    /// Each hyperedge is filtered to exclude the removed nodes.
    pub fn with_nodes_removed(&self, mask: u128) -> Vec<Self> {
        Self::from_dense_hyperedges_with_nodes(
            self.hyperedges.iter().map(|e| e & !mask).collect(),
            self.nodes.clone(),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::dense::DenseConstructor;
    use crate::dense::DenseTakingGame;
    use crate::Constructor;
    use evaluator::{Evaluator, Impartial};

    #[test]
    fn test_simple_move_generation() {
        let g = DenseTakingGame::from_hyperedges(vec![(0..5).collect()])
            .into_iter()
            .next()
            .unwrap();
        //assert_eq!(g.get_split_moves(), Vec::<Vec<TakingGame>>::new());
        assert_eq!(g.get_split_moves().len(), 5);
    }
    #[test]
    fn test_empty_game_move_generation() {
        let g = DenseTakingGame::empty();
        assert_eq!(g.get_split_moves(), Vec::<Vec<DenseTakingGame>>::new());
    }
    #[test]
    fn test_unit_move_generation() {
        let g = DenseConstructor::unit().build_one();
        assert_eq!(
            g.get_split_moves(),
            vec![vec![DenseConstructor::empty().build_one()]]
        )
    }
    #[test]
    fn test_l_move_generation() {
        let g = DenseConstructor::from_hyperedges(vec![vec![0, 1], vec![1, 2]]).build_one();
        let mut moves: Vec<usize> = g.get_split_moves().iter().map(|m| m.len()).collect();
        moves.sort();
        assert_eq!(moves, vec![1, 1, 2])
    }

    #[test]
    fn test_nimber_heaps() {
        let eval = Evaluator::new();

        for size in [0, 1, 2, 5, 10, 50] {
            let g = DenseConstructor::heap(size).build_one();
            assert_eq!(eval.get_nimber(&g), Some(size));
        }
    }
    #[test]
    fn test_s() {
        let eval = Evaluator::new();
        let g =
            DenseConstructor::from_hyperedges(vec![vec![0, 1], vec![1, 2], vec![2, 3]]).build_one();
        assert_eq!(eval.get_nimber(&g), Some(1));
    }
    #[test]
    fn test_w() {
        let eval = Evaluator::new();
        let g =
            DenseConstructor::from_hyperedges(vec![vec![0, 1], vec![1, 2], vec![2, 3], vec![3, 4]])
                .build_one();
        dbg!(g.get_split_moves());
        assert_eq!(eval.get_nimber(&g), Some(4));
    }
    #[test]
    fn testaalksdf() {
        let eval = Evaluator::new();
        let g = Constructor::from_hyperedges(vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![0, 8, 9, 10, 11, 12, 13, 14],
        ])
        .build_one();
        assert_eq!(eval.get_nimber(&g), Some(15));
    }
    #[test]
    fn test_kayles() {
        let g = DenseConstructor::kayles(9).build_one();
        let mut moves: Vec<Vec<usize>> = g
            .get_split_moves()
            .iter()
            .map(|m| m.iter().flat_map(|s| s.nodes.clone()).collect())
            .collect();
        moves.iter_mut().for_each(|m| m.sort());
    }
    #[test]
    fn test_kayles_16() {
        let g = DenseConstructor::kayles(9).build_one();
        let _move = g.with_nodes_removed(16);
    }
    #[test]
    fn test_many() {
        let eval = Evaluator::new();
        let g = DenseConstructor::hyper_cube(2, 4).build_one();
        assert_eq!(eval.get_nimber(&g), Some(0));
        let g = DenseConstructor::rect(1, 2).build_one();
        assert_eq!(eval.get_nimber(&g), Some(2));
        let g = DenseConstructor::hyper_cube(2, 3).build_one();
        assert_eq!(eval.get_nimber(&g), Some(0));
        let g = DenseConstructor::kayles(40).build_one();
        assert_eq!(eval.get_nimber(&g), Some(1));
        let g = DenseConstructor::hyper_tetrahedron(2).build_one();
        assert_eq!(eval.get_nimber(&g), Some(0));
        let g = DenseConstructor::hyper_cuboid(vec![2, 2, 3]).build_one();
        assert_eq!(eval.get_nimber(&g), Some(0));
    }
}
