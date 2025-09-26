use super::{util, TakingGame};
use std::{collections::HashMap, mem};
use union_find::{QuickUnionUf, UnionByRank, UnionFind};

impl Default for TakingGame {
    /// Returns an empty `TakingGame`.
    fn default() -> Self {
        Self::empty()
    }
}

impl TakingGame {
    /// Returns an empty `TakingGame`.
    pub fn empty() -> TakingGame {
        TakingGame {
            hyperedges: Vec::new(),
            edge_structure_partitions: Vec::new(),
            node_structure_partitions: Vec::new(),
            nodes: Vec::new(),
            //unconnected_nodes: Vec::new(),
        }
    }
    /// Constructs one or more `TakingGame`s from hyperedges only.
    /// May return multiple components if the hypergraph is disconnected.
    pub fn from_hyperedges(hyperedges: Vec<Vec<usize>>) -> Vec<TakingGame> {
        Self::from_hyperedges_with_nodes(hyperedges, Vec::new()) //, Vec::new());
    }
    /// Constructs one or more `TakingGame`s from hyperedges and optional node labels.
    /// - Removes redundant hyperedges (subsets).
    /// - Splits disconnected parts into separate games.
    pub fn from_hyperedges_with_nodes(
        hyperedges: Vec<Vec<usize>>,
        nodes: Vec<usize>,
    ) -> Vec<TakingGame> {
        let mut g = TakingGame {
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
    /// Assumes `nodes` is either empty (auto-fill) or consistent with hyperedges.
    fn flatten_nodes(&mut self) {
        let mut all_nodes: Vec<usize> = self
            .hyperedges
            .iter()
            .flat_map(|s| s.iter())
            .copied()
            .collect();
        all_nodes.sort_unstable();
        all_nodes.dedup();

        let edges_are_compact = all_nodes
            .last()
            .is_some_and(|last| last + 1 == all_nodes.len());

        let mut node_map = HashMap::new();
        if !edges_are_compact {
            for (i, n) in all_nodes.iter().enumerate() {
                node_map.insert(n, i);
            }

            // Remap hyperedge indices
            for e in self.hyperedges.iter_mut() {
                (0..e.len()).for_each(|i| {
                    e[i] = *node_map
                        .get(&e[i])
                        .expect("all nodes should be in node map");
                });
                e.sort_unstable();
                e.dedup();
            }
        }
        // Update `nodes` accordingly
        if self.nodes.is_empty() || all_nodes.is_empty() {
            self.nodes = all_nodes;
        } else if edges_are_compact {
            self.nodes.truncate(all_nodes.len());
        } else {
            let old_nodes = mem::take(&mut self.nodes);
            self.nodes = vec![0; all_nodes.len()];
            for (old_index, new_index) in node_map.iter() {
                self.nodes[*new_index] = old_nodes[**old_index];
            }
        }
    }

    /// Removes redundant hyperedges:
    /// - Deletes hyperedges that are subsets of others.
    /// - Re-flattens node indices if modifications occur.
    fn remove_redundant_hyperedges(&mut self) {
        self.flatten_nodes();
        self.hyperedges.sort_by_key(|e| e.len());

        let mut hyperedges_to_remove = Vec::new();

        for i in 0..self.hyperedges.len() {
            'inner: for j in (i + 1)..self.hyperedges.len() {
                if util::is_subset(&self.hyperedges[i], &self.hyperedges[j]) {
                    hyperedges_to_remove.push(i);
                    break 'inner;
                }
            }
        }
        if hyperedges_to_remove.is_empty() {
            return;
        }
        for e in hyperedges_to_remove.iter().rev() {
            self.hyperedges.remove(*e);
        }
        if !hyperedges_to_remove.is_empty() {
            self.flatten_nodes();
        }
    }

    /// Splits the game into connected components.
    /// Returns one `TakingGame` per component.
    pub fn get_parts(self) -> Vec<TakingGame> {
        let mut uf: QuickUnionUf<UnionByRank> = QuickUnionUf::new(self.nodes.len());

        // Union all nodes in each hyperedge
        for e in &self.hyperedges {
            let mut iter = e.iter();
            if let Some(&first) = iter.next() {
                for &node in iter {
                    uf.union(first, node);
                }
            }
        }

        let mut group_map: HashMap<usize, TakingGame> = HashMap::new();
        for e in self.hyperedges.into_iter() {
            if let Some(&representative) = e.first() {
                let root = uf.find(representative);

                match group_map.get_mut(&root) {
                    Some(g) => g.hyperedges.push(e),
                    None => {
                        let mut default = TakingGame::empty();
                        default.hyperedges.push(e);
                        default.nodes = self.nodes.clone();
                        group_map.insert(root, default);
                    }
                }
            }
        }

        let mut parts: Vec<TakingGame> = group_map.into_values().collect();
        if parts.len() > 1 {
            parts.iter_mut().for_each(|part| part.flatten_nodes());
        }
        parts.iter_mut().for_each(|part| part.partition_sort());
        parts
    }
    /// Computes the hypergraph dual:
    /// - For each node, returns the list of hyperedges it belongs to.
    pub fn hypergraph_dual(&self) -> Vec<Vec<usize>> {
        let mut dual: Vec<Vec<usize>> = vec![Vec::new(); self.nodes.len()];

        for (edge_index, edge) in self.hyperedges.iter().enumerate() {
            for &node in edge {
                dual[node].push(edge_index);
            }
        }
        dual
    }

    /// Applies a permutation to reorder hyperedges.
    /// Assumes `permutation` is a valid reordering of [0..edges).
    fn apply_edge_permutation(&mut self, permutation: &[usize]) {
        let l = self.hyperedges.len();
        let mut old_hyperedges = mem::replace(&mut self.hyperedges, vec![Vec::new(); l]);

        for i in 0..l {
            self.hyperedges[i] = mem::take(&mut old_hyperedges[permutation[i]]);
        }
    }

    /// Applies a permutation to reorder nodes.
    /// Also updates hyperedges to reflect new node indices.
    /// Assumes `permutation` is a valid reordering of [0..nodes).
    fn apply_node_permutation(&mut self, permutation: &[usize]) {
        let l = self.nodes.len();
        let old_nodes = mem::replace(&mut self.nodes, vec![0; l]);

        for i in 0..l {
            self.nodes[i] = old_nodes[permutation[i]];
        }

        // Build inverse mapping for remapping hyperedges
        let mut inv = vec![0; permutation.len()];
        util::fill_inverse_permutation(&mut inv, permutation);
        for e in self.hyperedges.iter_mut() {
            for i in 0..e.len() {
                e[i] = inv[e[i]];
            }
            e.sort();
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
        let initial_edge_keys: Vec<usize> =
            self.hyperedges.iter().map(|nodes| nodes.len()).collect();

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
                edge_keys[i].extend(
                    e.iter()
                        .map(|node| node_partition_map[inv_node_permutation[*node]]),
                );
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
                edge_keys[i].extend(e.iter().map(|node| inv_node_permutation[*node]));
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
    fn test_split() {
        let splits = TakingGame::from_hyperedges(vec![vec![0], vec![1], Vec::new()]);
        assert_eq!(splits.len(), 2);
        assert_eq!(splits[0].nodes.len(), 1);
        assert_eq!(splits[1].nodes.len(), 1);
    }
    #[test]
    fn test_canonization() {
        let game1 = TakingGame::from_hyperedges(vec![vec![5, 2, 4], vec![0, 4], vec![0, 2]]);
        let game2 = TakingGame::from_hyperedges(vec![vec![8, 1, 3], vec![3, 5], vec![1, 5]]);
        assert_eq!(game1, game2); // should be true due to canonization
    }

    use super::*;

    #[test]
    fn test_empty_game() {
        let empty_game = TakingGame::empty();
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
        let game = TakingGame::from_hyperedges(original_sets.clone())
            .into_iter()
            .next()
            .unwrap();

        let mut new_node_10: usize = 0;
        let mut new_node_20: usize = 0;
        let mut new_node_50: usize = 0;

        let dual = game.hypergraph_dual();

        for (i, edges) in dual.iter().enumerate() {
            if edges.len() == 1 && game.hyperedges[edges[0]].len() == 2 {
                new_node_10 = i;
            }
            // 20 is the only one in two sets of size 3
            if edges.len() == 2
                && game.hyperedges[edges[0]].len() == 3
                && game.hyperedges[edges[1]].len() == 3
            {
                new_node_20 = i;
            }
            // 50 is the only node that is on two sets and a set that has size 2
            if edges.len() == 2
                && (game.hyperedges[edges[0]].len() == 2 || game.hyperedges[edges[1]].len() == 2)
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
        let parent_game = TakingGame::from_hyperedges(original_sets.clone())
            .into_iter()
            .next()
            .unwrap();

        let mut new_hyperedges = parent_game.hyperedges;

        let new_node_99: usize = parent_game.nodes.iter().position(|n| *n == 99).unwrap();
        let new_node_100: usize = parent_game.nodes.iter().position(|n| *n == 100).unwrap();
        for e in &mut new_hyperedges.iter_mut() {
            if let Ok(index) = e.binary_search(&new_node_99) {
                e.remove(index);
            }
            if let Ok(index) = e.binary_search(&new_node_100) {
                e.remove(index);
            }
        }
        let game = TakingGame::from_hyperedges_with_nodes(new_hyperedges, parent_game.nodes)
            .into_iter()
            .next()
            .unwrap();

        let mut new_node_10: usize = 0;
        let mut new_node_20: usize = 0;
        let mut new_node_50: usize = 0;

        let dual = game.hypergraph_dual();

        for (i, edges) in dual.iter().enumerate() {
            // 10 is the only node that is in one set  and a set that has size 2
            if edges.len() == 1 && game.hyperedges[edges[0]].len() == 2 {
                new_node_10 = i;
            }
            // 20 is the only one in two sets of size 3
            if edges.len() == 2
                && game.hyperedges[edges[0]].len() == 3
                && game.hyperedges[edges[1]].len() == 3
            {
                new_node_20 = i;
            }
            // 50 is the only node that is on two sets and a set that has size 2
            if edges.len() == 2
                && (game.hyperedges[edges[0]].len() == 2 || game.hyperedges[edges[1]].len() == 2)
            {
                new_node_50 = i;
            }
        }

        assert_eq!(game.nodes[new_node_10], 10);
        assert_eq!(game.nodes[new_node_20], 20);
        assert_eq!(game.nodes[new_node_50], 50);
    }
}
