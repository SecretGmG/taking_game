use evaluator::Impartial;
use itertools::Itertools;

use crate::{
    hypergraph::{Bitset128, Set},
    taking_game::TakingGame,
};

impl Impartial for TakingGame {
    /// Return the maximum possible nimber for this game.
    ///
    /// If the game has a symmetry, the nimber is 0. Otherwise, it is
    /// bounded above by the number of nodes.
    fn get_max_nimber(&self) -> Option<usize> {
        match self.find_symmetry() {
            Some(_) => Some(0),
            None => Some(self.graph.nr_nodes()),
        }
    }

    /// Generate move splits by considering one representative
    /// from each structural equivalence class of edges.
    fn get_split_moves(&self) -> Vec<Vec<TakingGame>> {
        if self.graph.is_empty() {
            return vec![];
        }
        self.graph
            .get_edge_partitions()
            .iter()
            .flat_map(|e| self.get_moves_of_edge(e.start))
            .collect()
    }
}

impl TakingGame {
    /// Generate all moves resulting from removing nodes belonging
    /// to a given hyperedge, partitioned by structural equivalence.
    fn get_moves_of_edge(&self, hyperedge: usize) -> impl Iterator<Item = Vec<TakingGame>> + '_ {
        let partitioned_hyperedge =
            self.graph.hyperedges()[hyperedge].partition(&self.graph.get_node_partitions());

        let nodes_to_remove_per_part = partitioned_hyperedge.into_iter().map(|mut part| {
            let mut nodes_to_remove_in_part = Vec::with_capacity(part.len() + 1);
            nodes_to_remove_in_part.push(part.clone());
            while part.pop().is_some() {
                nodes_to_remove_in_part.push(part.clone());
            }
            //make sure the do nothing move comes first
            let last = nodes_to_remove_in_part.len() - 1;
            nodes_to_remove_in_part.swap(0, last);
            nodes_to_remove_in_part
        });

        let nodes_to_remove = nodes_to_remove_per_part
            .multi_cartesian_product()
            .map(|nodes_to_remove_in_parts| {
                let mut nodes_to_remove = Bitset128::default();
                nodes_to_remove_in_parts
                    .iter()
                    .for_each(|n| nodes_to_remove.union(n));
                nodes_to_remove
            })
            .skip(1);
        nodes_to_remove.map(|mask| self.with_nodes_removed(mask))
    }

    /// Return new game states with the given nodes removed.
    ///
    /// Each hyperedge is filtered to exclude the removed nodes.
    pub fn with_nodes_removed(&self, mask: Bitset128) -> Vec<Self> {
        self.graph
            .minus(mask)
            .into_iter()
            .map(|graph| Self { graph })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::Builder;
    use crate::hypergraph::Bitset128;

    #[test]
    fn test_simple_move_generation() {
        let g = Builder::from_hyperedges(vec![(0..5).collect()])
            .build_one()
            .unwrap();
        assert_eq!(g.get_split_moves().len(), 5);
    }

    #[test]
    fn test_max_nimber_empty_and_unit() {
        assert!(Builder::empty().build_one().is_none());

        let unit = Builder::unit().build_one().unwrap();
        assert_eq!(unit.get_max_nimber(), Some(1));
    }

    #[test]
    fn test_with_nodes_removed_basic() {
        let game = Builder::heap(3).build_one().unwrap();
        let mut mask = Bitset128::default();
        mask.union(&Bitset128::from_slice(&[0])); // remove node 0
        let with_one_removed = game.with_nodes_removed(mask);
        assert_eq!(with_one_removed.len(), 1);
        assert_eq!(with_one_removed[0].nr_nodes(), 2);
    }

    #[test]
    fn test_split_moves_single_edge() {
        // Graph with a single hyperedge of 5 nodes
        let g = Builder::from_hyperedges(vec![(0..5).collect()])
            .build_one()
            .unwrap();
        let moves = g.get_split_moves();

        // There should be 5 possible moves (removing 1..5 nodes),
        // each yielding exactly 1 connected component
        assert_eq!(moves.len(), 5);
    }

    #[test]
    fn test_split_moves_two_edges() {
        // Graph: two disjoint edges of size 2
        let g = Builder::from_hyperedges(vec![(0..=4).collect(), (4..8).collect()])
            .build_one()
            .unwrap();
        let moves = g.get_split_moves();

        // Moves exist on each hyperedge independently
        assert!(!moves.is_empty());
        // Each move yields components, either 1 or 2
        assert!(moves.iter().all(|comp| !comp.is_empty() && comp.len() <= 2));
    }

    #[test]
    fn test_split_moves_edge_overlap() {
        // Graph: edges overlap on one node
        let g = Builder::from_hyperedges(vec![vec![0, 1], vec![1, 2]])
            .build_one()
            .unwrap();
        let moves = g.get_split_moves();

        // Moves should still be non-empty
        assert!(!moves.is_empty());

        // At least some moves should result in multiple components
        assert!(moves.iter().any(|comp| comp.len() > 1));
    }
}
