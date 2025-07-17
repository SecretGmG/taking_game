use std::collections::HashMap;

use super::TakingGame;
use evaluator::Impartial;
use sorted_vec::SortedSet;
use union_find::{QuickFindUf, UnionByRank, UnionFind};

impl Impartial<TakingGame> for TakingGame {
    #[cfg(feature = "no_split")]
    fn get_parts(&self) -> Option<Vec<TakingGame>> {
        None
    }

    #[cfg(not(feature = "no_split"))]
    fn get_parts(&self) -> Option<Vec<TakingGame>> {
        let independent_sets_of_nodes = split_to_independent_sets_of_nodes(&self);
        if independent_sets_of_nodes.len() <= 1 {
            None
        } else {
            Some(
                independent_sets_of_nodes
                    .into_iter()
                    .map(|sets| TakingGame::new(sets))
                    .collect(),
            )
        }
    }
    fn get_max_nimber(&self) -> Option<usize> {
        //return self.get_node_count();

        match self.find_symmetry() {
            Some(_) => Some(0),
            None => Some(self.node_count),
        }
    }

    fn get_moves(&self) -> Vec<TakingGame> {
        let mut moves = vec![];

        for set_of_nodes in &self.sets_of_nodes {
            let (lone_nodes, other_nodes) = self.collect_lone_nodes_and_other_nodes(set_of_nodes);
            self.append_moves_of_set(lone_nodes, other_nodes, &mut moves);
        }
        return moves;
    }
}
fn split_to_independent_sets_of_nodes(g: &TakingGame) -> Vec<Vec<SortedSet<usize>>> {
    // First, determine the maximum node index
    let mut uf: QuickFindUf<UnionByRank> = QuickFindUf::new(g.get_node_count() + 1);

    // Union all nodes in each set
    for set in g.get_sets_of_nodes() {
        let mut iter = set.iter();
        if let Some(&first) = iter.next() {
            for &node in iter {
                uf.union(first.into(), node.into());
            }
        }
    }

    // Group sets by their component root
    let mut group_map: HashMap<usize, Vec<SortedSet<usize>>> = HashMap::new();
    for set in g.get_sets_of_nodes() {
        if let Some(&representative) = set.iter().next() {
            let root = uf.find(representative);
            group_map.entry(root).or_default().push(set.clone());
        }
    }

    group_map.into_values().collect()
}

use super::util;

//implements the generation of moves;
impl TakingGame {
    fn append_moves_of_set(
        &self,
        lone_nodes: Vec<usize>,
        other_nodes: Vec<usize>,
        child_games: &mut Vec<TakingGame>,
    ) {
        let other_len = other_nodes.len();
        if other_len > 128 {
            panic!("This game is too complex!")
        }

        let mask_bound = 1u128 << other_len;
        for lone_nodes_to_remove in 0..(lone_nodes.len() + 1) {
            let start = if lone_nodes_to_remove == 0 { 1 } else { 0 };
            for mask in start..mask_bound {
                child_games.push(self.get_child(
                    &lone_nodes,
                    &other_nodes,
                    lone_nodes_to_remove,
                    mask,
                ));
            }
        }
    }
    fn collect_lone_nodes_and_other_nodes(
        &self,
        set_of_nodes: &Vec<usize>,
    ) -> (Vec<usize>, Vec<usize>) {
        set_of_nodes
            .iter()
            .copied()
            .partition(|&node| self.get_set_indices()[node].len() == 1)
    }

    fn get_child(
        &self,
        lone_nodes: &Vec<usize>,
        other_nodes: &Vec<usize>,
        lone_nodes_to_remove: usize,
        mask: u128,
    ) -> TakingGame {
        let mut nodes_to_remove =
            SortedSet::with_capacity(lone_nodes_to_remove + other_nodes.len());

        for &node in lone_nodes.iter().take(lone_nodes_to_remove) {
            nodes_to_remove.push(node);
        }
        for (i, &node) in other_nodes.iter().enumerate() {
            if (mask >> i) & 1 == 1 {
                nodes_to_remove.push(node);
            }
        }
        self.make_move_unchecked(&mut nodes_to_remove)
    }
    ///removes all nodes specified in the argument
    pub fn make_move_unchecked(&self, nodes_to_remove: &mut SortedSet<usize>) -> TakingGame {
        TakingGame::new(
            self.sets_of_nodes
                .iter()
                .map(|set| util::remove_subset(set, nodes_to_remove))
                .collect(),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::Constructor;
    use evaluator::Evaluator;

    #[test]
    fn test_many() {
        let mut eval = Evaluator::new();
        let g = Constructor::hyper_cube(2, 4);
        assert_eq!(eval.get_nimber(g.build()), Some(0));
        let g = Constructor::hyper_cube(2, 3);
        assert_eq!(eval.get_nimber(g.build()), Some(0));
        let g = Constructor::kayles(40);
        assert_eq!(eval.get_nimber(g.build()), Some(1));
        let g = Constructor::hyper_tetrahedron(10);
        assert_eq!(eval.get_nimber(g.build()), Some(2));
        let g = Constructor::triangle(4);
        assert_eq!(eval.get_nimber(g.build()), Some(0));
        let g = Constructor::hyper_cuboid(vec![2, 2, 3]);
        assert_eq!(eval.get_nimber(g.build()), Some(0));
    }
}
