use sorted_vec::SortedSet;

use super::{util, TakingGame};
use std::{cmp::Ordering, collections::HashMap};

impl TakingGame {
    #[cfg(feature = "no_sort")]
    const MAX_SORT_STEPS: usize = 0;

    #[cfg(not(feature = "no_sort"))]
    const MAX_SORT_STEPS: usize = 256;

    #[allow(dead_code)]
    ///creates an empty GeneralizedNimGame
    pub fn empty() -> TakingGame {
        TakingGame {
            sets_of_nodes: Vec::new(),
            node_count: 0,
            set_indices: Vec::new(),
        }
    }

    /// Constructs a canonicalized TakingGame from a collection of node sets.
    ///
    /// This function:
    /// - Normalizes node indices to a compact range starting from 0
    /// - Deduplicates and sorts each set
    /// - Removes redundant (subset) sets
    /// - Computes and stores set indices per node
    /// - Reorders node indices to canonical form
    pub fn new(sets_of_nodes: Vec<SortedSet<usize>>) -> TakingGame {
        let (flattened_sets_of_nodes, node_count) = Self::flatten_and_get_node_count(sets_of_nodes);
        let cleaned_sets_of_nodes = Self::remove_redundant_sets(flattened_sets_of_nodes);
        let (sorted_sets_of_nodes, set_indices) = Self::sort(cleaned_sets_of_nodes, node_count);

        TakingGame {
            set_indices: set_indices,
            sets_of_nodes: sorted_sets_of_nodes,
            node_count,
        }
    }
    ///flattens the indecies and then returns the nr of nodes
    fn flatten_and_get_node_count(
        mut sets_of_nodes: Vec<SortedSet<usize>>,
    ) -> (Vec<SortedSet<usize>>, usize) {
        let mut all_nodes: Vec<usize> = sets_of_nodes
            .iter()
            .flat_map(|s| s.iter())
            .copied()
            .collect();
        all_nodes.sort_unstable();
        all_nodes.dedup();

        let mut map = HashMap::with_capacity(all_nodes.len());
        for (i, val) in all_nodes.iter().enumerate() {
            map.insert(*val, i);
        }

        for set in &mut sets_of_nodes {
            *set = SortedSet::from_unsorted(set.iter().map(|x| map[x]).collect());
        }

        (sets_of_nodes, all_nodes.len())
    }
    ///removes sets that are totally contained in other sets
    fn remove_redundant_sets(mut sets_of_nodes: Vec<SortedSet<usize>>) -> Vec<SortedSet<usize>> {
        sets_of_nodes.sort_by_key(|set| set.len());

        let mut retained = Vec::new();
        'outer: for i in 0..sets_of_nodes.len() {
            for j in (i + 1)..sets_of_nodes.len() {
                if util::is_subset(&sets_of_nodes[i], &sets_of_nodes[j]) {
                    continue 'outer;
                }
            }
            if !sets_of_nodes[i].is_empty() {
                retained.push(sets_of_nodes[i].clone());
            }
        }

        retained
    }
    pub fn sort(
        mut sets_of_nodes: Vec<SortedSet<usize>>,
        node_count: usize,
    ) -> (Vec<SortedSet<usize>>, Vec<Vec<usize>>) {
        Self::sort_sets_of_nodes_by_indices(&mut sets_of_nodes);
        let mut set_indices = Self::generate_set_indices(&sets_of_nodes, node_count);

        for _ in 0..Self::MAX_SORT_STEPS {
            let permutation = Self::generate_index_mapping(&set_indices, node_count);
            if permutation.iter().enumerate().all(|(a, b)| a == *b) {
                return (sets_of_nodes, set_indices);
            }
            Self::apply_permutation(&mut sets_of_nodes, &permutation);
            Self::sort_sets_of_nodes_by_indices(&mut sets_of_nodes);
            set_indices = Self::generate_set_indices(&sets_of_nodes, node_count);
        }
        (sets_of_nodes, set_indices)
    }
    fn apply_permutation(sets: &mut Vec<SortedSet<usize>>, perm: &Vec<usize>) {
        for set in sets.iter_mut() {
            let mut new_set: Vec<usize> = set.iter().map(|&x| perm[x]).collect();
            new_set.sort_unstable();
            *set = unsafe {SortedSet::from_sorted(new_set)};
        }
    }
    ///sorts each set of nodes and sorts the sets of nodes
    fn sort_sets_of_nodes_by_indices(sets_of_nodes: &mut Vec<SortedSet<usize>>) {
        sets_of_nodes.sort_by(|set1, set2| util::compare_sorted(set1, set2));
    }
    fn generate_index_mapping(set_indices: &Vec<Vec<usize>>, node_count: usize) -> Vec<usize> {
        let mut inverse_maping: Vec<usize> = (0..node_count).collect();
        inverse_maping.sort_by(|a, b| Self::node_comparer(*a, *b, &set_indices));
        util::inverse_permutation(inverse_maping)
    }
    fn node_comparer(a: usize, b: usize, set_indices: &Vec<Vec<usize>>) -> Ordering {
        util::compare_sorted(&set_indices[a], &set_indices[b])
    }
    fn generate_set_indices(
        sets_of_nodes: &Vec<SortedSet<usize>>,
        node_count: usize,
    ) -> Vec<Vec<usize>> {
        let mut node_to_sets: Vec<Vec<usize>> = vec![vec![]; node_count];

        for (set_index, set) in sets_of_nodes.iter().enumerate() {
            for &node in set.iter() {
                node_to_sets[node].push(set_index);
            }
        }
        node_to_sets
    }

    
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_canonicalization() {
        let game1 = TakingGame::new(vec![
            SortedSet::from_unsorted(vec![2, 4]),
            SortedSet::from_unsorted(vec![0, 4]),
            SortedSet::from_unsorted(vec![0, 2]),
        ]);
        let game2 = TakingGame::new(vec![
            SortedSet::from_unsorted(vec![1, 3]),
            SortedSet::from_unsorted(vec![3, 5]),
            SortedSet::from_unsorted(vec![1, 5]),
        ]);
        assert_eq!(game1, game2); // should be true due to canonicalization
    }

    use super::*;
    use sorted_vec::SortedSet;

    #[test]
    fn test_empty_game() {
        let empty_game = TakingGame::empty();
        assert_eq!(empty_game.node_count, 0);
        assert!(empty_game.sets_of_nodes.is_empty());
        assert!(empty_game.set_indices.is_empty());
    }
}
