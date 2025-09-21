use std::collections::HashMap;

use super::TakingGame;
use evaluator::Impartial;
use sorted_vec::SortedSet;
use union_find::{QuickUnionUf, UnionByRank, UnionFind};

impl Impartial for TakingGame {
    /// Provides an upper bound on the nimber of the game.
    ///
    /// This returns 0 if a symmetry is found (indicating potential simplification),
    /// otherwise returns the node count as a worst-case upper bound.
    fn get_max_nimber(&self) -> Option<usize> {
        //return self.get_node_count();

        match self.find_symmetry() {
            Some(_) => Some(0),
            None => Some(self.get_node_count()),
        }
    }

    /// Returns all legal child positions by simulating every possible move.
    ///
    /// Each legal move is generated from a set of nodes and all non-empty subsets
    /// of those nodes. Lone nodes (that only occur in one move set) are treated specially.
    ///
    /// # Efficiency
    /// This is exponential in the number of nodes per move (O(2ⁿ)). A panic is triggered
    /// for sets with more than 128 nodes.
    fn get_split_moves(&self) -> Vec<Vec<TakingGame>> {
        self.sets_of_nodes
            .iter()
            .flat_map(|set_of_nodes| {
                let (lone_nodes, other_nodes) =
                    self.collect_lone_nodes_and_other_nodes(set_of_nodes);
                self.get_moves_of_set(lone_nodes, other_nodes)
            })
            .map(|_move| {
                split_to_independent_sets_of_nodes(&_move)
                    .into_iter()
                    .map(TakingGame::from_sets_of_nodes)
                    .collect()
            })
            .collect()
    }
}
fn split_to_independent_sets_of_nodes(g: &TakingGame) -> Vec<Vec<SortedSet<usize>>> {
    // First, determine the maximum node index
    let mut uf: QuickUnionUf<UnionByRank> = QuickUnionUf::new(g.get_node_count());

    // Union all nodes in each set
    for set in g.get_sets_of_nodes() {
        let mut iter = set.iter();
        if let Some(&first) = iter.next() {
            for &node in iter {
                uf.union(first, node);
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
    /// Generates all legal moves from a single move set by trying
    /// every subset of the nodes involved (excluding the empty set).
    ///
    /// Nodes that occur in no other move sets are treated separately.
    ///
    /// # Panics
    /// Panics if the number of non-lone nodes exceeds 128 to avoid overflow
    /// in bitmask-based enumeration.
    fn get_moves_of_set(&self, lone_nodes: Vec<usize>, other_nodes: Vec<usize>) -> Vec<TakingGame> {
        let other_len = other_nodes.len();
        if other_len > 128 {
            panic!("This game is too complex!")
        }
        let mask_bound = 1u128 << other_len;
        (0..(lone_nodes.len() + 1))
            .flat_map(|lone_nodes_to_remove| {
                // mask == 0 and lone_nodes_to_remove == 0 => empty move, which is illegal
                let start = if lone_nodes_to_remove == 0 { 1 } else { 0 };
                (start..mask_bound).map(move |mask| (lone_nodes_to_remove, mask))
            })
            .map(|(lone_nodes_to_remove, mask)| {
                self.get_child(&lone_nodes, &other_nodes, lone_nodes_to_remove, mask)
            })
            .collect()
    }
    /// Splits a node set into:
    /// - Lone nodes: nodes that appear in only one set.
    /// - Other nodes: nodes shared across multiple sets.
    ///
    /// Used to optimize move enumeration.
    fn collect_lone_nodes_and_other_nodes(
        &self,
        set_of_nodes: &[usize],
    ) -> (Vec<usize>, Vec<usize>) {
        set_of_nodes
            .iter()
            .copied()
            .partition(|&node| self.get_set_indices()[node].len() == 1)
    }
    /// Computes a child game resulting from removing a subset of nodes.
    ///
    /// The `mask` determines which of the `other_nodes` to include.
    /// A prefix of `lone_nodes` is also removed.
    fn get_child(
        &self,
        lone_nodes: &[usize],
        other_nodes: &[usize],
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
    /// Removes all nodes specified in the argument and returns the resulting game.
    ///
    /// Preserves the original node mapping.
    pub fn make_move_unchecked(&self, nodes_to_remove: &mut SortedSet<usize>) -> TakingGame {
        TakingGame::from_sets_of_nodes_with_node_map(
            self.sets_of_nodes
                .iter()
                .map(|set| util::remove_subset(set, nodes_to_remove))
                .collect(),
            self.nodes.clone(),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::Constructor;
    use evaluator::Evaluator;

    #[test]
    fn test_many() {
        let eval = Evaluator::new();
        let g = Constructor::hyper_cube(2, 4);
        assert_eq!(eval.get_nimber(&g.build()), Some(0));
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
