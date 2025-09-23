use super::TakingGame;
use evaluator::Impartial;
use itertools::Itertools;

impl Impartial for TakingGame {
    fn get_max_nimber(&self) -> Option<usize> {
        match self.find_symmetry() {
            Some(_) => Some(0),
            None => Some(self.nodes.len()),
        }
    }
    fn get_split_moves(&self) -> Vec<Vec<TakingGame>> {
        //get moves for one representative for each structural equivalence class
        self.edge_structure_partitions
            .iter()
            .rev()
            .skip(1) // the last index in the partition is always = hyperedges.len()
            .flat_map(|e| self.get_moves_of_set(&self.hyperedges[*e]))
            .collect()
    }
}

//implements the generation of moves;
impl TakingGame {
    fn get_moves_of_set(&self, hyperedge: &[usize]) -> Vec<Vec<TakingGame>> {
        let mut partitioned_edge: Vec<&[usize]> = vec![];

        let mut start = 0;
        let mut p = 1;

        for i in 0..hyperedge.len() {
            if hyperedge[i] >= self.node_structure_partitions[p] {
                partitioned_edge.push(&hyperedge[start..i]);
                start = i;
                p += 1;
            }
        }
        partitioned_edge.push(&hyperedge[start..]);

        partitioned_edge
            .iter()
            .map(|part| (0..=part.len()).rev().map(|i| part[0..i].to_vec())) //remove 0 to all from each structural equivalnve class of nodes in this edge
            .multi_cartesian_product()
            .map(|nodes_to_remove| nodes_to_remove.into_iter().flatten().collect())
            .skip(1)
            .map(|nodes_to_remove| self.with_nodes_removed(nodes_to_remove))
            .collect()
    }

    pub fn with_nodes_removed(&self, nodes: Vec<usize>) -> Vec<Self> {
        TakingGame::from_hyperedges_with_nodes(
            self.hyperedges
                .iter()
                .map(|e| e.iter().filter(|n| nodes.contains(n)).copied().collect())
                .collect(),
            self.nodes.clone(),
        )
    }
}

#[cfg(test)]
mod test {
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
        let g = Constructor::unit().build();
        assert_eq!(g.get_split_moves(), vec![Vec::<TakingGame>::new()])
    }

    #[test]
    fn test_many() {
        let eval = Evaluator::new();
        let g = Constructor::hyper_cube(2, 4);
        assert_eq!(eval.get_nimber(&g.build()), Some(0));
        let g = Constructor::rect(1, 10);
        assert_eq!(eval.get_nimber(&g.build()), Some(10));
        let g = Constructor::hyper_cube(2, 3);
        assert_eq!(eval.get_nimber(&g.build()), Some(0));
        let g = Constructor::kayles(40);
        assert_eq!(eval.get_nimber(&g.build()), Some(1));
        let g = Constructor::hyper_tetrahedron(10);
        assert_eq!(eval.get_nimber(&g.build()), Some(2));
        let g = Constructor::triangle(4);
        assert_eq!(eval.get_nimber(&g.build()), Some(0));
        let g = Constructor::hyper_cuboid(vec![2, 2, 3]);
        assert_eq!(eval.get_nimber(&g.build()), Some(0));
    }
}
