use super::{util, TakingGame};
use rayon::vec;
use std::{collections::HashMap, mem};
use union_find::{QuickUnionUf, UnionByRank, UnionFind};

impl Default for TakingGame {
    fn default() -> Self {
        Self::empty()
    }
}

impl TakingGame {
    ///creates an empty GeneralizedNimGame
    pub fn empty() -> TakingGame {
        TakingGame {
            hyperedges: Vec::new(),
            edge_structure_partitions: Vec::new(),
            node_structure_partitions: Vec::new(),
            nodes: Vec::new(),
            unconnected_nodes: Vec::new(),
        }
    }
    pub fn from_hyperedges(hyperedges: Vec<Vec<usize>>) -> Vec<TakingGame> {
        return Self::from_hyperedges_with_nodes(hyperedges, Vec::new(), Vec::new());
    }
    pub fn from_hyperedges_with_nodes(
        hyperedges: Vec<Vec<usize>>,
        nodes: Vec<usize>,
        unconnected_nodes: Vec<Vec<usize>>,
    ) -> Vec<TakingGame> {
        let mut g = TakingGame {
            hyperedges,
            edge_structure_partitions: Vec::new(),
            node_structure_partitions: Vec::new(),
            nodes,
            unconnected_nodes,
        };

        // start by removing everything that is not necessary
        g.remove_redundant_hyperedges();
        g.absorb_unconnected_nodes();

        //now split into independent parts
        g.get_parts()
    }
    /// Normalize node indices by flattening and mapping original node labels
    /// to a compact range 0..N-1, updates hyperedges and original node labels.
    fn flatten_nodes(&mut self) {
        let mut all_nodes: Vec<usize> = self
            .hyperedges
            .iter()
            .flat_map(|s| s.iter())
            .copied()
            .collect();
        all_nodes.sort_unstable();
        all_nodes.dedup();

        if *all_nodes.last().unwrap_or(&0) == all_nodes.len() - 1 {
            return;
        }

        let mut node_map = HashMap::new();
        for (i, n) in all_nodes.iter().enumerate() {
            node_map.insert(n, i);
        }

        for e in self.hyperedges.iter_mut() {
            for n in e.iter_mut() {
                *n = *node_map.get(n).expect("all nodes should be in node map");
            }
            e.sort_unstable();
            e.dedup();
        }

        if self.nodes.is_empty() {
            self.nodes = all_nodes;
        } else {
            let enumerator = std::mem::replace(&mut self.nodes, vec![0; all_nodes.len()])
                .into_iter()
                .enumerate();
            for (i, label) in enumerator {
                self.nodes[*node_map.get(&i).expect("all nodes should be in node map")] = label;
            }
        }
    }
    /// Remove sets that are subsets of other sets
    /// Sorts and dedups hyperedges
    fn remove_redundant_hyperedges(&mut self) {
        self.flatten_nodes();

        util::sort_together_by_key(&mut self.hyperedges, &mut self.unconnected_nodes, |e| {
            e.len()
        });

        let mut retained_hyperedges = Vec::new();
        let mut retained_unconnected_nodes = Vec::new();

        if self.unconnected_nodes.is_empty() {
            self.unconnected_nodes = vec![Vec::new(); self.hyperedges.len()]
        }

        'outer: for i in 0..self.hyperedges.len() {
            let node_count = self.unconnected_nodes[i].len();
            for j in (i + 1)..self.hyperedges.len() {
                if node_count == 0 && util::is_subset(&self.hyperedges[i], &self.hyperedges[j]) {
                    continue 'outer;
                }
            }
            if !self.hyperedges[i].is_empty() || node_count != 0 {
                retained_hyperedges.push(std::mem::take(&mut self.hyperedges[i]));
                retained_unconnected_nodes.push(std::mem::take(&mut self.unconnected_nodes[i]));
            }
        }
        if self.hyperedges.len() != retained_hyperedges.len() {
            self.hyperedges = retained_hyperedges;
            self.unconnected_nodes = retained_unconnected_nodes;
            self.flatten_nodes();
        }
    }
    // identifies nodes that are only contained in 1 hyperedge, these are then
    // removed and noted in unconnected_node_counts
    // needs initilized unconnected_nodes vector!
    fn absorb_unconnected_nodes(&mut self) {
        let mut edges_of_lone_nodes: Vec<Option<Option<usize>>> = vec![None; self.nodes.len()];
        for (e, nodes) in self.hyperedges.iter().enumerate() {
            for n in nodes {
                match edges_of_lone_nodes[*n] {
                    Some(_) => edges_of_lone_nodes[*n] = Some(None),
                    None => edges_of_lone_nodes[*n] = Some(Some(e)),
                }
            }
        }
        let mut removed_nodes: usize = 0;
        for (n, maybe_e) in edges_of_lone_nodes.iter().enumerate() {
            if let Some(e) = maybe_e.expect("all nodes should have been looked at") {
                let node_index = self.hyperedges[e]
                    .binary_search(&n)
                    .expect("hyperedge should contain this node, hyperedge should be sorted");

                self.hyperedges[e].remove(node_index);
                self.unconnected_nodes[e].push(self.nodes[n]);
                removed_nodes += 1;
            }
        }
        if removed_nodes > 0 {
            self.flatten_nodes();
        }
    }
    pub fn get_parts(self) -> Vec<TakingGame> {
        let mut uf: QuickUnionUf<UnionByRank> = QuickUnionUf::new(self.nodes.len());

        // Union all nodes in each set
        for e in &self.hyperedges {
            let mut iter = e.iter();
            if let Some(&first) = iter.next() {
                for &node in iter {
                    uf.union(first, node);
                }
            }
        }

        // No new redundancies will be created!
        let mut group_map: HashMap<usize, TakingGame> = HashMap::new();
        for (e, unconnected_nodes) in self.hyperedges.into_iter().zip(self.unconnected_nodes) {
            if let Some(&representative) = e.iter().next() {
                let root = uf.find(representative);
                let g = group_map.entry(root).or_insert_with(|| {
                    let mut default = TakingGame::empty();
                    default.nodes = self.nodes.clone();
                    default
                });
                g.hyperedges.push(e);
                g.unconnected_nodes.push(unconnected_nodes);
            }
        }

        let mut parts: Vec<TakingGame> = group_map.into_values().collect();
        if parts.len() > 1 {
            parts.iter_mut().for_each(|part| part.flatten_nodes());
        }
        parts.iter_mut().for_each(|part| part.partition_sort());
        parts
    }
    pub fn hypergraph_dual(&self) -> Vec<Vec<usize>> {
        // initialize one empty vec per node
        let mut dual: Vec<Vec<usize>> = vec![Vec::new(); self.nodes.len()];

        for (edge_index, edge) in self.hyperedges.iter().enumerate() {
            for &node in edge {
                dual[node].push(edge_index);
            }
        }
        dual
    }

    fn refine_partitions_by_key<T: Ord>(
        partitions: &mut Vec<usize>,
        permutation: &[usize],
        keys: &[T],
    ) {
        for i in 0..keys.len().saturating_sub(1) {
            if keys[permutation[i]] != keys[permutation[i + 1]] {
                if let Err(partition_index) = partitions.binary_search(&i) {
                    partitions.insert(partition_index, i);
                }
            }
        }
    }

    fn sort_partitions_by_key<T: Ord>(partitions: &[usize], permutation: &mut [usize], keys: &[T]) {
        for i in 0..partitions.len() - 1 {
            let part = &mut permutation[partitions[i]..partitions[i + 1]];
            part.sort_by_key(|e| &keys[*e]);
        }
    }
    fn sort_refine_partitions_by_key<T: Ord>(
        partitions: &mut Vec<usize>,
        permutation: &mut [usize],
        keys: &[T],
    ) {
        Self::sort_partitions_by_key(partitions, permutation, keys);
        Self::refine_partitions_by_key(partitions, permutation, keys);
    }

    /// Returns a partition map assigning each element to a partition index.
    /// Partition indices are 1-based: elements in the first block map to 1,
    /// next block to 2, etc.
    fn fill_partition_map(buff: &mut [usize], partitions: &[usize]) {
        let mut p = 1;
        for i in 0..buff.len() {
            if partitions[p] == i {
                p += 1;
            }
            buff[i] = p;
        }
    }
    fn fill_inverse_permutation(buff: &mut [usize], permutation: &[usize]) {
        for i in 0..permutation.len() {
            buff[permutation[i]] = i
        }
    }
    fn apply_edge_permutation(&mut self, permutation: &[usize]) {
        let l = self.hyperedges.len();
        let mut old_hyperedges = mem::replace(&mut self.hyperedges, vec![Vec::new(); l]);

        for i in 0..l {
            self.hyperedges[i] = mem::take(&mut old_hyperedges[permutation[i]]);
        }
    }
    fn apply_node_permutation(&mut self, permutation: &[usize]) {
        let l = self.nodes.len();
        let old_nodes = mem::replace(&mut self.nodes, vec![0; l]);

        for i in 0..l {
            self.nodes[i] = old_nodes[permutation[i]];
        }

        let mut inv = vec![0; permutation.len()];
        Self::fill_inverse_permutation(&mut inv, permutation);
        for e in self.hyperedges.iter_mut() {
            for n in e.iter_mut() {
                *n = inv[*n];
            }
            e.sort();
        }
    }

    /// Sort sets and nodes into pseudo-canonical order.
    /// partitions sets and nodes into structural equivalents classes
    fn partition_sort(&mut self) {
        let mut edge_permutation: Vec<usize> = (0..self.hyperedges.len()).collect();
        let mut node_permutation: Vec<usize> = (0..self.nodes.len()).collect();

        let dual = self.hypergraph_dual();
        let initial_node_keys: Vec<usize> = dual.iter().map(|edges| edges.len()).collect();
        let initial_edge_keys: Vec<(usize, usize)> = self
            .hyperedges
            .iter()
            .zip(self.unconnected_nodes.iter())
            .map(|(e, unconnected)| (unconnected.len(), e.len()))
            .collect();

        self.edge_structure_partitions = vec![0, self.hyperedges.len()];
        self.node_structure_partitions = vec![0, self.nodes.len()];

        Self::sort_refine_partitions_by_key(
            &mut self.edge_structure_partitions,
            &mut edge_permutation,
            &initial_edge_keys,
        );
        Self::sort_refine_partitions_by_key(
            &mut self.node_structure_partitions,
            &mut node_permutation,
            &initial_node_keys,
        );

        self.build_structural_eq_classes(&mut edge_permutation, &mut node_permutation, &dual);
        self.sort_canonically(&mut edge_permutation, &mut node_permutation, &dual);

        self.apply_edge_permutation(&mut edge_permutation);
        self.apply_node_permutation(&mut node_permutation);
    }

    fn build_structural_eq_classes(
        &mut self,
        edge_permutation: &mut Vec<usize>,
        node_permutation: &mut Vec<usize>,
        dual: &Vec<Vec<usize>>,
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
            Self::fill_partition_map(&mut node_partition_map, &self.node_structure_partitions);
            Self::fill_partition_map(&mut edge_partition_map, &self.edge_structure_partitions);

            Self::fill_inverse_permutation(&mut inv_node_permutation, &node_permutation);
            Self::fill_inverse_permutation(&mut inv_edge_permutation, &edge_permutation);

            for (i, n) in dual.iter().enumerate() {
                node_keys[i].clear();
                node_keys[i].extend(
                    n.iter()
                        .map(|edge| edge_partition_map[inv_edge_permutation[*edge]]),
                );
                node_keys[i].sort_unstable();
            }
            for (i, e) in self.hyperedges.iter().enumerate() {
                edge_keys[i].clear();
                edge_keys[i].extend(
                    e.iter()
                        .map(|node| node_partition_map[inv_node_permutation[*node]]),
                );
                edge_keys[i].sort_unstable();
            }

            Self::sort_refine_partitions_by_key(
                &mut self.node_structure_partitions,
                node_permutation,
                &node_keys,
            );
            Self::sort_refine_partitions_by_key(
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
    fn sort_canonically(
        &mut self,
        edge_permutation: &mut Vec<usize>,
        node_permutation: &mut Vec<usize>,
        dual: &Vec<Vec<usize>>,
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

            Self::fill_inverse_permutation(&mut inv_node_permutation, &node_permutation);
            Self::fill_inverse_permutation(&mut inv_edge_permutation, &edge_permutation);

            for (i, n) in dual.iter().enumerate() {
                node_keys[i].clear();
                node_keys[i].extend(n.iter().map(|edge| inv_edge_permutation[*edge]));
                node_keys[i].sort_unstable();
            }
            for (i, e) in self.hyperedges.iter().enumerate() {
                edge_keys[i].clear();
                edge_keys[i].extend(e.iter().map(|node| inv_node_permutation[*node]));
                edge_keys[i].sort_unstable();
            }

            Self::sort_partitions_by_key(
                &self.edge_structure_partitions,
                edge_permutation,
                &edge_keys,
            );
            Self::sort_partitions_by_key(
                &self.node_structure_partitions,
                node_permutation,
                &node_keys,
            );

            if &old_edge == edge_permutation && &old_node == node_permutation {
                break;
            }
        }
    }
    // Apply a node permutation to the sets and nodes.
    //
    // Re-indexes each node in sets according to permutation.
    // fn apply_permutation(sets: &mut [SortedSet<usize>], nodes: &mut Vec<usize>, perm: &[usize]) {
    //     for set in sets.iter_mut() {
    //         let mut new_set: Vec<usize> = set.iter().map(|&x| perm[x]).collect();
    //         new_set.sort_unstable();
    //         *set = unsafe { SortedSet::from_sorted(new_set) };
    //     }
    //     let mut new_nodes = vec![0; nodes.len()];
    //     for i in 0..nodes.len() {
    //         new_nodes[perm[i]] = nodes[i];
    //     }
    //     *nodes = new_nodes;
    // }
    // /// Lexicographically sorts the list of sets, assuming each set is already sorted
    // fn sort_sets_of_nodes_by_indices(sets_of_nodes: &mut [SortedSet<usize>]) {
    //     sets_of_nodes.sort_by(|set1, set2| util::compare_sorted(set1, set2));
    // }
    // /// Generate the permutation mapping that orders nodes by their set membership.
    // ///
    // /// This orders nodes by lex order of their set indices, then inverts to a permutation.
    // fn generate_index_mapping(set_indices: &[Vec<usize>], node_count: usize) -> Vec<usize> {
    //     let mut inverse_mapping: Vec<usize> = (0..node_count).collect();
    //     inverse_mapping.sort_by(|a, b| Self::node_comparer(*a, *b, set_indices));
    //     util::inverse_permutation(inverse_mapping)
    // }
    // fn node_comparer(a: usize, b: usize, set_indices: &[Vec<usize>]) -> Ordering {
    //     util::compare_sorted(&set_indices[a], &set_indices[b])
    // }
    // fn generate_set_indices(
    //     sets_of_nodes: &[SortedSet<usize>],
    //     node_count: usize,
    // ) -> Vec<Vec<usize>> {
    //     let mut node_to_sets: Vec<Vec<usize>> = vec![vec![]; node_count];
    //
    //     for (set_index, set) in sets_of_nodes.iter().enumerate() {
    //         for &node in set.iter() {
    //             node_to_sets[node].push(set_index);
    //         }
    //     }
    //     node_to_sets
    //}
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_canonization() {
        let game1 = TakingGame::from_hyperedges(vec![vec![2, 4], vec![0, 4], vec![0, 2]]);
        let game2 = TakingGame::from_hyperedges(vec![vec![1, 3], vec![3, 5], vec![1, 5]]);
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
        for i in 0..game.nodes.len() {
            let sets: Vec<usize> = vec![]; //e.get_set_indices()[i];
                                           // 10 is the only node that is in one set  and a set that has size 2
            if sets.len() == 1 && game.hyperedges[sets[0]].len() == 2 {
                new_node_10 = i;
            }
            // 20 is the only one in two sets of size 3
            if sets.len() == 2
                && game.hyperedges[sets[0]].len() == 3
                && game.hyperedges[sets[1]].len() == 3
            {
                new_node_20 = i;
            }
            // 50 is the only node that is on two sets and a set that has size 2
            if sets.len() == 2
                && (game.hyperedges[sets[0]].len() == 2 || game.hyperedges[sets[1]].len() == 2)
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
        let game = TakingGame::from_hyperedges_with_nodes(
            new_hyperedges,
            parent_game.nodes,
            parent_game.unconnected_nodes,
        )
        .into_iter()
        .next()
        .unwrap();

        let mut new_node_10: usize = 0;
        let mut new_node_20: usize = 0;
        let mut new_node_50: usize = 0;
        for i in 0..game.nodes.len() {
            let edges: Vec<usize> = vec![]; //&game.get_set_indices()[i];

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
