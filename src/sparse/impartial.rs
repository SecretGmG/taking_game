use std::collections::HashSet;

use super::TakingGame;
use evaluator::Impartial;
use itertools::Itertools;

impl Impartial for TakingGame {
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
    fn get_split_moves(&self) -> Vec<Vec<TakingGame>> {
        self.edge_structure_partitions
            .iter()
            .rev()
            .skip(1) // the last index in the partition is always = hyperedges.len()
            .flat_map(|e| self.get_moves_of_set(&self.hyperedges[*e]))
            .collect()
    }
}

impl TakingGame {
    /// Generate all moves resulting from removing nodes belonging
    /// to a given hyperedge, partitioned by structural equivalence.
    fn get_moves_of_set(&self, hyperedge: &[usize]) -> impl Iterator<Item = Vec<TakingGame>> + '_ {
        let mut partitioned_edge = vec![];

        let mut start = 0;
        let mut p = 1;

        for i in 0..hyperedge.len() {
            if hyperedge[i] >= self.node_structure_partitions[p] {
                partitioned_edge.push(hyperedge[start..i].to_vec());
                start = i;
                p += 1;
            }
        }
        partitioned_edge.push(hyperedge[start..].to_vec());

        let nr_nodes_to_remove = (0..partitioned_edge.len())
            .map(|part| 0..=partitioned_edge[part].len()) //remove 0 to all from each structural equivalnve class of nodes in this edge
            .multi_cartesian_product();

        let nodes_to_remove = nr_nodes_to_remove
            .map(move |nr_nodes_to_remove| {
                nr_nodes_to_remove
                    .iter()
                    .enumerate()
                    .flat_map(|(i, n)| partitioned_edge[i][0..*n].iter().copied())
                    .collect()
            })
            .skip(1);
        nodes_to_remove.map(|nodes_to_remove| self.with_nodes_removed(nodes_to_remove))
    }

    /// Return new game states with the given nodes removed.
    ///
    /// Each hyperedge is filtered to exclude the removed nodes.
    pub fn with_nodes_removed(&self, nodes: HashSet<usize>) -> Vec<Self> {
        TakingGame::from_hyperedges_with_nodes(
            self.hyperedges
                .iter()
                .map(|e| e.iter().filter(|n| !nodes.contains(n)).copied().collect())
                .collect(),
            self.nodes.clone(),
        )
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{Constructor, TakingGame};
    use evaluator::{Evaluator, Impartial};

    #[test]
    fn test_simple_move_generation() {
        let g = TakingGame::from_hyperedges(vec![(0..5).collect()])
            .into_iter()
            .next()
            .unwrap();
        //assert_eq!(g.get_split_moves(), Vec::<Vec<TakingGame>>::new());
        assert_eq!(g.get_split_moves().len(), 5);
    }
    #[test]
    fn test_empty_game_move_generation() {
        let g = TakingGame::empty();
        assert_eq!(g.get_split_moves(), Vec::<Vec<TakingGame>>::new());
    }
    #[test]
    fn test_unit_move_generation() {
        let g = Constructor::unit().build_one();
        assert_eq!(g.get_split_moves(), vec![Vec::<TakingGame>::new()])
    }
    #[test]
    fn test_split_node_remove() {
        let g = Constructor::from_hyperedges(vec![vec![0, 2], vec![1, 2]]).build_one();
        let mut nodes_to_remove = HashSet::new();
        nodes_to_remove.insert(2);
        let with_node_removed = g.with_nodes_removed(nodes_to_remove);
        assert_eq!(with_node_removed.len(), 2);
    }
    #[test]
    fn test_l_move_generation() {
        let g = Constructor::from_hyperedges(vec![vec![0, 1], vec![1, 2]]);
        let mut splits: Vec<usize> = g
            .build_one()
            .get_split_moves()
            .iter()
            .map(|split| split.len())
            .collect();
        splits.sort();
        assert_eq!(splits, vec![1, 1, 2]);
    }

    #[test]
    fn test_nimber_heaps() {
        let eval = Evaluator::new();

        for size in [0, 1, 2, 5, 10, 50] {
            let g = Constructor::heap(size).build_one();
            assert_eq!(eval.get_nimber(&g), Some(size));
        }
    }
    #[test]
    fn test_kayles() {
        let eval = Evaluator::new();
        for (size, nimber) in [0, 1, 2, 3, 1, 4, 3, 2, 1, 4, 2, 6].into_iter().enumerate() {
            let g = Constructor::kayles(size).build_one();
            assert_eq!(eval.get_nimber(&g), Some(nimber));
        }
    }
    #[test]
    fn test_many() {
        let eval = Evaluator::new();
        let g = Constructor::hyper_cube(2, 4).build_one();
        assert_eq!(eval.get_nimber(&g), Some(0));
        let g = Constructor::rect(1, 2).build_one();
        assert_eq!(eval.get_nimber(&g), Some(2));
        let g = Constructor::hyper_cube(2, 3).build_one();
        assert_eq!(eval.get_nimber(&g), Some(0));
        let g = Constructor::kayles(40).build_one();
        assert_eq!(eval.get_nimber(&g), Some(1));
        let g = Constructor::hyper_tetrahedron(2).build_one();
        assert_eq!(eval.get_nimber(&g), Some(0));
        let g = Constructor::hyper_cuboid(vec![2, 2, 3]).build_one();
        assert_eq!(eval.get_nimber(&g), Some(0));
    }
}
