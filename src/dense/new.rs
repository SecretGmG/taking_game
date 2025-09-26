use super::{util, DenseTakingGame};

impl Default for DenseTakingGame {
    /// Returns an empty `TakingGame`.
    fn default() -> Self {
        Self::empty()
    }
}

impl DenseTakingGame {
    /// Returns an empty `TakingGame`.
    pub fn empty() -> DenseTakingGame {
        DenseTakingGame {
            hyperedges: Vec::new(),
            edge_structure_partitions: Vec::new(),
            node_structure_partitions: Vec::new(),
            nodes: Vec::new(),
        }
    }
    /// Constructs one or more `TakingGame`s from hyperedges only.
    /// May return multiple components if the hypergraph is disconnected.
    pub fn from_hyperedges(hyperedges: Vec<Vec<usize>>) -> Vec<DenseTakingGame> {
        let nodes = (0..=hyperedges.iter().flatten().max().copied().unwrap_or(0usize)).collect();
        Self::from_hyperedges_with_nodes(hyperedges, nodes)
        //, Vec::new());
    }
    /// Constructs one or more `TakingGame`s from hyperedges and optional node labels.
    /// - Removes redundant hyperedges (subsets).
    /// - Splits disconnected parts into separate games.
    pub fn from_hyperedges_with_nodes(
        hyperedges: Vec<Vec<usize>>,
        nodes: Vec<usize>,
    ) -> Vec<DenseTakingGame> {
        Self::from_dense_hyperedges_with_nodes(
            hyperedges
                .into_iter()
                .map(|edge| {
                    let mut mask = 0u128;
                    for n in edge {
                        mask |= 1 << n;
                    }
                    mask
                })
                .collect(),
            nodes,
        )
    }

    /// Constructs one or more `TakingGame`s from hyperedges and optional node labels.
    /// - Removes redundant hyperedges (subsets).
    /// - Splits disconnected parts into separate games.
    pub fn from_dense_hyperedges_with_nodes(
        hyperedges: Vec<u128>,
        nodes: Vec<usize>,
    ) -> Vec<DenseTakingGame> {
        let mut g = DenseTakingGame {
            hyperedges,
            edge_structure_partitions: Vec::new(),
            node_structure_partitions: Vec::new(),
            nodes,
        };
        g.remove_redundant_hyperedges();
        g.get_parts()
    }

    /// Normalize node indices:
    /// - Maps arbitrary node labels to a compact range [0..N).
    /// - Updates both `hyperedges` and `nodes`.
    ///
    /// Assumes `nodes` is consistent with hyperedges.
    fn flatten_nodes(&mut self) {
        let all_nodes = self.hyperedges.iter().fold(0, |a, b| a | b);

        if all_nodes.trailing_ones() == all_nodes.count_ones() {
            self.nodes.truncate(all_nodes.count_ones() as usize);
            return;
        }

        let mut nodemap = vec![];
        let mut mask = all_nodes;
        let mut idx = 0;
        while mask != 0 {
            if mask & 1 != 0 {
                nodemap.push(idx);
            }
            mask >>= 1;
            idx += 1;
        }

        debug_assert_eq!(nodemap.len(), all_nodes.count_ones() as usize);

        for edge in self.hyperedges.iter_mut() {
            let mut new_edge = 0u128;
            for (new_idx, &old_idx) in nodemap.iter().enumerate() {
                if (*edge & (1 << old_idx)) != 0 {
                    new_edge |= 1 << new_idx;
                }
            }
            debug_assert_eq!(edge.count_ones(), new_edge.count_ones());
            *edge = new_edge;
        }
        let old_labels = std::mem::take(&mut self.nodes);
        self.nodes = nodemap.iter().map(|&new_idx| old_labels[new_idx]).collect();
    }

    /// Removes redundant hyperedges:
    /// - Deletes hyperedges that are subsets of others.
    /// - Re-flattens node indices if modifications occur.
    fn remove_redundant_hyperedges(&mut self) {
        self.flatten_nodes();
        //biggest hyperedges first
        self.hyperedges.sort_by_key(|e| e.count_zeros());

        let mut new_edges = Vec::new();

        for &e in &self.hyperedges {
            if new_edges.iter().all(|&ue| (e | ue) != ue) {
                new_edges.push(e);
            }
        }
        debug_assert_eq!(
            self.hyperedges.iter().fold(0, |a, b| a | b),
            new_edges.iter().fold(0, |a, b| a | b)
        );
        if self.hyperedges.len() == new_edges.len() {
            return;
        }
        self.hyperedges = new_edges;
        self.flatten_nodes();
    }
    /// Splits the game into connected components.
    /// Returns one `TakingGame` per component.
    pub fn get_parts(mut self) -> Vec<DenseTakingGame> {
        // Union all nodes in each hyperedge
        let mut masks: Vec<u128> = Vec::new();

        for &e in &self.hyperedges {
            let mut merged = e;
            let mut i = 0;
            while i < masks.len() {
                if masks[i] & merged != 0 {
                    merged |= masks[i];
                    masks.swap_remove(i);
                    // don’t increment i, check the swapped-in element too
                } else {
                    i += 1;
                }
            }
            masks.push(merged);
        }

        debug_assert_eq!(
            self.nodes.len(),
            masks.iter().map(|m| m.count_ones() as usize).sum()
        );

        if masks.len() > 1 {
            let mut parts: Vec<DenseTakingGame> = vec![self.clone(); masks.len()];
            for (part, mask) in parts.iter_mut().zip(masks) {
                for e in part.hyperedges.iter_mut() {
                    *e &= mask
                }
                part.remove_redundant_hyperedges();
                part.partition_sort();
            }
            dbg!(parts
                .iter()
                .map(|part| part.nodes.clone())
                .collect::<Vec<Vec<usize>>>());
            parts
        } else {
            self.partition_sort();
            vec![self]
        }
    }
    /// Computes the hypergraph dual:
    /// - For each node, returns the list of hyperedges it belongs to.
    pub fn hypergraph_dual(&self) -> Vec<Vec<usize>> {
        let mut dual = vec![Vec::new(); self.nodes.len()];
        for (i, &edge) in self.hyperedges.iter().enumerate() {
            (0..self.nodes.len()).for_each(|n| {
                if edge & (1 << n) != 0 {
                    dual[n].push(i);
                }
            });
        }
        dual
    }

    /// Applies a permutation to reorder hyperedges.
    /// Assumes `permutation` is a valid reordering of [0..edges).
    fn apply_edge_permutation(&mut self, permutation: &[usize]) {
        let l = self.hyperedges.len();
        let old_hyperedges = self.hyperedges.clone();

        for i in 0..l {
            self.hyperedges[i] = old_hyperedges[permutation[i]];
        }
    }

    /// Applies a permutation to reorder nodes.
    /// Also updates hyperedges to reflect new node indices.
    /// Assumes `permutation` is a valid reordering of [0..nodes).
    fn apply_node_permutation(&mut self, permutation: &[usize]) {
        let l = self.nodes.len();
        let old_nodes = self.nodes.clone();

        for i in 0..l {
            self.nodes[i] = old_nodes[permutation[i]];
        }

        // Build inverse mapping for remapping hyperedges
        for edge in self.hyperedges.iter_mut() {
            let mut new_edge = 0u128;
            for node in 0..self.nodes.len() {
                if (*edge & (1 << permutation[node])) != 0 {
                    new_edge |= 1 << node;
                }
            }
            *edge = new_edge;
        }
    }

    /// Sorts nodes and hyperedges into a canonical order.
    /// - Builds structural equivalence classes.
    /// - Refines partitions until stable.
    /// - Applies canonical permutations to nodes and edges.
    fn partition_sort(&mut self) {
        let mut edge_permutation: Vec<usize> = (0..self.hyperedges.len()).collect();
        let mut node_permutation: Vec<usize> = (0..self.nodes.len()).collect();

        let dual = self.hypergraph_dual();
        let initial_node_keys: Vec<usize> = dual.iter().map(|edges| edges.len()).collect();
        let initial_edge_keys: Vec<u32> = self
            .hyperedges
            .iter()
            .map(|nodes| nodes.count_zeros())
            .collect();

        self.edge_structure_partitions = vec![0, self.hyperedges.len()];
        self.node_structure_partitions = vec![0, self.nodes.len()];

        util::sort_refine_partitions_by_key(
            &mut self.edge_structure_partitions,
            &mut edge_permutation,
            &initial_edge_keys,
        );
        util::sort_refine_partitions_by_key(
            &mut self.node_structure_partitions,
            &mut node_permutation,
            &initial_node_keys,
        );

        self.build_structural_eq_classes(&mut edge_permutation, &mut node_permutation, &dual);
        self.sort_canonically(&mut edge_permutation, &mut node_permutation, &dual);

        self.apply_edge_permutation(&edge_permutation);
        self.apply_node_permutation(&node_permutation);
    }

    /// Refines structural equivalence classes of nodes and edges
    /// until partition count stops increasing.
    fn build_structural_eq_classes(
        &mut self,
        edge_permutation: &mut [usize],
        node_permutation: &mut [usize],
        dual: &[Vec<usize>],
    ) {
        let mut nr_partitions =
            self.edge_structure_partitions.len() + self.node_structure_partitions.len();

        let mut node_keys = vec![Vec::new(); self.nodes.len()];
        let mut edge_keys = vec![Vec::new(); self.hyperedges.len()];

        let mut inv_node_permutation = vec![0; self.nodes.len()];
        let mut inv_edge_permutation = vec![0; self.hyperedges.len()];

        let mut node_partition_map = vec![0; self.nodes.len()];
        let mut edge_partition_map = vec![0; self.hyperedges.len()];

        loop {
            util::fill_partition_map(&mut edge_partition_map, &self.edge_structure_partitions);
            util::fill_inverse_permutation(&mut inv_edge_permutation, edge_permutation);
            for (i, n) in dual.iter().enumerate() {
                node_keys[i].clear();
                node_keys[i].extend(
                    n.iter()
                        .map(|edge| edge_partition_map[inv_edge_permutation[*edge]]),
                );
                node_keys[i].sort_unstable();
            }

            util::sort_refine_partitions_by_key(
                &mut self.node_structure_partitions,
                node_permutation,
                &node_keys,
            );

            util::fill_partition_map(&mut node_partition_map, &self.node_structure_partitions);
            util::fill_inverse_permutation(&mut inv_node_permutation, node_permutation);
            for (i, e) in self.hyperedges.iter().enumerate() {
                edge_keys[i].clear();
                for node in 0..self.nodes.iter().len() {
                    if e & (1 << node) != 0 {
                        edge_keys[i].push(node_partition_map[inv_node_permutation[node]]);
                    }
                }
                edge_keys[i].sort_unstable();
            }

            util::sort_refine_partitions_by_key(
                &mut self.edge_structure_partitions,
                edge_permutation,
                &edge_keys,
            );
            let new_nr_partitions =
                self.edge_structure_partitions.len() + self.node_structure_partitions.len();
            if new_nr_partitions == nr_partitions {
                break;
            }
            nr_partitions = new_nr_partitions;
        }
    }

    /// Canonicalizes order of nodes and edges within partitions.
    /// Runs at most MAX_ITER iterations or until stable.
    fn sort_canonically(
        &mut self,
        edge_permutation: &mut Vec<usize>,
        node_permutation: &mut Vec<usize>,
        dual: &[Vec<usize>],
    ) {
        const MAX_ITER: usize = 8;

        let mut old_node = vec![0; node_permutation.len()];
        let mut old_edge = vec![0; edge_permutation.len()];

        let mut node_keys = vec![Vec::new(); self.nodes.len()];
        let mut edge_keys = vec![Vec::new(); self.hyperedges.len()];

        let mut inv_node_permutation = vec![0; self.nodes.len()];
        let mut inv_edge_permutation = vec![0; self.hyperedges.len()];

        for _ in 0..MAX_ITER {
            old_edge.copy_from_slice(edge_permutation);
            old_node.copy_from_slice(node_permutation);

            util::fill_inverse_permutation(&mut inv_edge_permutation, edge_permutation);
            for (i, n) in dual.iter().enumerate() {
                node_keys[i].clear();
                node_keys[i].extend(n.iter().map(|edge| inv_edge_permutation[*edge]));
                node_keys[i].sort_unstable();
            }
            util::sort_partitions_by_key(
                &self.node_structure_partitions,
                node_permutation,
                &node_keys,
            );

            util::fill_inverse_permutation(&mut inv_node_permutation, node_permutation);
            for (i, e) in self.hyperedges.iter().enumerate() {
                edge_keys[i].clear();
                for node in 0..self.nodes.iter().len() {
                    if e & (1 << node) != 0 {
                        edge_keys[i].push(inv_node_permutation[node]);
                    }
                }
                edge_keys[i].sort_unstable();
            }
            util::sort_partitions_by_key(
                &self.edge_structure_partitions,
                edge_permutation,
                &edge_keys,
            );

            if &old_edge == edge_permutation && &old_node == node_permutation {
                break;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_flatten() {
        let g = DenseTakingGame::from_hyperedges(vec![vec![3, 5, 28]])
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(g.hyperedges, vec![7]);
        assert_eq!(g.nodes, vec![3, 5, 28]);
    }
    #[test]
    fn test_remove_redundant() {
        let g = DenseConstructor::from_hyperedges(vec![
            vec![1, 2, 3],
            vec![1, 2, 3],
            vec![1],
            vec![2, 3],
            vec![],
        ])
        .build_one();
        assert_eq!(g.hyperedges, vec![7]);
    }
    #[test]
    fn test_basic_label_preservation() {
        let g = DenseTakingGame::from_hyperedges_with_nodes(vec![vec![1, 3]], vec![0, 10, 0, 30])
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(g.hyperedges, vec![3]);
        assert_eq!(g.nodes, vec![10, 30]);
    }
    #[test]
    fn test_split() {
        let splits = DenseTakingGame::from_hyperedges(vec![vec![0], vec![1], Vec::new()]);
        assert_eq!(splits.len(), 2);
        assert_eq!(splits[0].nodes.len(), 1);
        assert_eq!(splits[1].nodes.len(), 1);
    }
    #[test]
    fn test_canonization() {
        let game1 = DenseTakingGame::from_hyperedges(vec![vec![5, 2, 4], vec![0, 4], vec![0, 2]]);
        let game2 = DenseTakingGame::from_hyperedges(vec![vec![8, 1, 3], vec![3, 5], vec![1, 5]]);
        assert_eq!(game1, game2); // should be true due to canonization
    }

    use crate::dense::DenseConstructor;

    use super::*;

    #[test]
    fn test_empty_game() {
        let empty_game = DenseTakingGame::empty();
        assert_eq!(empty_game.nodes.len(), 0);
        assert!(empty_game.hyperedges.is_empty());
        assert!(empty_game.hyperedges.is_empty());
    }
    #[test]
    fn test_node_label_preservation_structured_example() {
        // Construct a game with uniquely identifiable nodes by membership profile
        let original_sets = vec![
            vec![10, 50],    // Set 0
            vec![50, 20, 3], // Set 1
            vec![20, 3, 4],  // Set 2
        ];

        // Create the canonicalized parent game
        let game = DenseTakingGame::from_hyperedges(original_sets.clone())
            .into_iter()
            .next()
            .unwrap();

        let mut new_node_10: usize = 0;
        let mut new_node_20: usize = 0;
        let mut new_node_50: usize = 0;

        let dual = game.hypergraph_dual();

        for (i, edges) in dual.iter().enumerate() {
            if edges.len() == 1 && game.hyperedges[edges[0]].count_ones() == 2 {
                new_node_10 = i;
            }
            // 20 is the only one in two sets of size 3
            if edges.len() == 2
                && game.hyperedges[edges[0]].count_ones() == 3
                && game.hyperedges[edges[1]].count_ones() == 3
            {
                new_node_20 = i;
            }
            // 50 is the only node that is on two sets and a set that has size 2
            if edges.len() == 2
                && (game.hyperedges[edges[0]].count_ones() == 2
                    || game.hyperedges[edges[1]].count_ones() == 2)
            {
                new_node_50 = i;
            }
        }

        assert_eq!(game.nodes[new_node_10], 10);
        assert_eq!(game.nodes[new_node_20], 20);
        assert_eq!(game.nodes[new_node_50], 50);
    }
    #[test]
    fn test_node_label_preservation_from_parent_structured_example() {
        // Construct a game with uniquely identifiable nodes by membership profile
        let original_sets = vec![
            vec![99, 10, 50],         // Set 0
            vec![50, 100, 20, 3, 99], // Set 1
            vec![20, 3, 4, 100],      // Set 2
        ];

        // Create the canonicalized parent game
        let parent_game = DenseTakingGame::from_hyperedges(original_sets.clone())
            .into_iter()
            .next()
            .unwrap();

        let mut new_hyperedges = parent_game.hyperedges;

        let new_node_99: u128 = 1 << parent_game.nodes.iter().position(|n| *n == 99).unwrap();
        let new_node_100: u128 = 1 << parent_game.nodes.iter().position(|n| *n == 100).unwrap();
        for e in new_hyperedges.iter_mut() {
            if new_node_99 & *e != 0 {
                *e &= !new_node_99;
            }
            if new_node_100 & *e != 0 {
                *e &= !new_node_100;
            }
        }
        let game =
            DenseTakingGame::from_dense_hyperedges_with_nodes(new_hyperedges, parent_game.nodes)
                .into_iter()
                .next()
                .unwrap();

        let mut new_node_10: usize = 0;
        let mut new_node_20: usize = 0;
        let mut new_node_50: usize = 0;

        let dual = game.hypergraph_dual();

        for (i, edges) in dual.iter().enumerate() {
            // 10 is the only node that is in one set  and a set that has size 2
            if edges.len() == 1 && game.hyperedges[edges[0]].count_ones() == 2 {
                new_node_10 = i;
            }
            // 20 is the only one in two sets of size 3
            if edges.len() == 2
                && game.hyperedges[edges[0]].count_ones() == 3
                && game.hyperedges[edges[1]].count_ones() == 3
            {
                new_node_20 = i;
            }
            // 50 is the only node that is on two sets and a set that has size 2
            if edges.len() == 2
                && (game.hyperedges[edges[0]].count_ones() == 2
                    || game.hyperedges[edges[1]].count_ones() == 2)
            {
                new_node_50 = i;
            }
        }

        assert_eq!(game.nodes[new_node_10], 10);
        assert_eq!(game.nodes[new_node_20], 20);
        assert_eq!(game.nodes[new_node_50], 50);
    }
}
