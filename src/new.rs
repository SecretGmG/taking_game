use sorted_vec::SortedSet;
use super::{util, TakingGame};
use std::cmp::Ordering;

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
            set_indices: Vec::new(),
            nodes: Vec::new(),
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
    ///
    /// The `nodes` vector here stores the original node labels after normalization,
    /// preserving the mapping from canonical indices to original labels.
    pub fn from_sets_of_nodes(sets_of_nodes: Vec<SortedSet<usize>>) -> TakingGame {
        let (flattened_sets_of_nodes, nodes) = Self::flatten_and_get_nodes(sets_of_nodes);
        let cleaned_sets_of_nodes = Self::remove_redundant_sets(flattened_sets_of_nodes);
        let (sorted_sets_of_nodes, nodes, set_indices) = Self::sort(cleaned_sets_of_nodes, nodes);

        TakingGame {
            set_indices,
            sets_of_nodes: sorted_sets_of_nodes,
            nodes,
        }
    }
    /// Reconstructs a `TakingGame` with node labels preserved from a parent game.
    ///
    /// This is useful when generating subgames or children in a search tree:
    /// - `sets_of_nodes` is the input set form the parent with some parts removed
    /// - `nodes` maps compact indices in the subgame to original game labels
    ///
    /// This function updates `game.nodes[i]` to be `nodes[game.nodes[i]]`,
    /// ensuring the node labeling remains consistent with the parent game.
    pub fn from_sets_of_nodes_with_node_map(
        sets_of_nodes: Vec<SortedSet<usize>>,
        nodes: Vec<usize>,
    ) -> TakingGame {
        let mut game = TakingGame::from_sets_of_nodes(sets_of_nodes);
        for i in 0..game.get_node_count() {
            let node_mapping = game.nodes[i];
            game.nodes[i] = nodes[node_mapping];
        }
        game
    }
    ///flattens the indecies and then returns the nr of nodes
    /// Normalize node indices by flattening and mapping original node labels
    /// to a compact range 0..N-1, returning the updated sets and original node labels.
    fn flatten_and_get_nodes(
        mut sets_of_nodes: Vec<SortedSet<usize>>,
    ) -> (Vec<SortedSet<usize>>, Vec<usize>) {
        let mut all_nodes: Vec<usize> = sets_of_nodes
            .iter()
            .flat_map(|s| s.iter())
            .copied()
            .collect();
        all_nodes.sort_unstable();
        all_nodes.dedup();

        for set in &mut sets_of_nodes {
            *set = SortedSet::from_unsorted(
                set.iter()
                    .map(|x| all_nodes.binary_search(x).unwrap())
                    .collect(),
            );
        }

        (sets_of_nodes, all_nodes)
    }
    /// Remove sets that are subsets of other sets.
    ///
    /// This avoids redundant moves in the game representation.
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
    /// Sort sets and nodes into canonical order.
    ///
    /// This performs up to MAX_SORT_STEPS reordering iterations until node order
    /// stabilizes. Each iteration:
    /// - Generates a permutation that orders nodes by their set membership pattern
    /// - Applies the permutation to nodes and sets
    /// - Recomputes set indices per node
    pub fn sort(
        mut sets_of_nodes: Vec<SortedSet<usize>>,
        mut nodes: Vec<usize>,
    ) -> (Vec<SortedSet<usize>>, Vec<usize>, Vec<Vec<usize>>) {
        Self::sort_sets_of_nodes_by_indices(&mut sets_of_nodes);
        let mut set_indices = Self::generate_set_indices(&sets_of_nodes, nodes.len());

        for _ in 0..Self::MAX_SORT_STEPS {
            let permutation = Self::generate_index_mapping(&set_indices, nodes.len());
            if permutation.iter().enumerate().all(|(a, b)| a == *b) {
                return (sets_of_nodes, nodes, set_indices);
            }
            Self::apply_permutation(&mut sets_of_nodes, &mut nodes, &permutation);
            Self::sort_sets_of_nodes_by_indices(&mut sets_of_nodes);
            set_indices = Self::generate_set_indices(&sets_of_nodes, nodes.len());
        }
        (sets_of_nodes, nodes, set_indices)
    }
    /// Apply a node permutation to the sets and nodes.
    ///
    /// Re-indexes each node in sets according to permutation.
    fn apply_permutation(
        sets: &mut Vec<SortedSet<usize>>,
        nodes: &mut Vec<usize>,
        perm: &Vec<usize>,
    ) {
        for set in sets.iter_mut() {
            let mut new_set: Vec<usize> = set.iter().map(|&x| perm[x]).collect();
            new_set.sort_unstable();
            *set = unsafe { SortedSet::from_sorted(new_set) };
        }
        let mut new_nodes = vec![0; nodes.len()];
        for i in 0..nodes.len() {
            new_nodes[perm[i]] = nodes[i];
        }
        *nodes = new_nodes;
    }
    ///sorts each set of nodes and sorts the sets of nodes
    fn sort_sets_of_nodes_by_indices(sets_of_nodes: &mut Vec<SortedSet<usize>>) {
        sets_of_nodes.sort_by(|set1, set2| util::compare_sorted(set1, set2));
    }
    /// Generate the permutation mapping that orders nodes by their set membership.
    ///
    /// This orders nodes by lex order of their set indices, then inverts to a permutation.
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
    fn test_canonization() {
        let game1 = TakingGame::from_sets_of_nodes(vec![
            SortedSet::from_unsorted(vec![2, 4]),
            SortedSet::from_unsorted(vec![0, 4]),
            SortedSet::from_unsorted(vec![0, 2]),
        ]);
        let game2 = TakingGame::from_sets_of_nodes(vec![
            SortedSet::from_unsorted(vec![1, 3]),
            SortedSet::from_unsorted(vec![3, 5]),
            SortedSet::from_unsorted(vec![1, 5]),
        ]);
        assert_eq!(game1, game2); // should be true due to cannonization
    }

    use super::*;
    use sorted_vec::SortedSet;

    #[test]
    fn test_empty_game() {
        let empty_game = TakingGame::empty();
        assert_eq!(empty_game.get_node_count(), 0);
        assert!(empty_game.sets_of_nodes.is_empty());
        assert!(empty_game.set_indices.is_empty());
    }
    #[test]
    fn test_node_label_preservation_structured_example() {
        use sorted_vec::SortedSet;

        // Construct a game with uniquely identifiable nodes by membership profile
        let original_sets = vec![
            SortedSet::from_unsorted(vec![10, 50]),    // Set 0
            SortedSet::from_unsorted(vec![50, 20, 3]), // Set 1
            SortedSet::from_unsorted(vec![20, 3, 4]),  // Set 2
        ];

        // Create the canonicalized parent game
        let game = TakingGame::from_sets_of_nodes(original_sets.clone());

        let mut new_node_10: usize = 0;
        let mut new_node_20: usize = 0;
        let mut new_node_50: usize = 0;
        for i in 0..game.get_node_count() {
            let sets = &game.get_set_indices()[i];
            // 10 is the only node that is in one set  and a set that has size 2
            if sets.len() == 1 && game.get_sets_of_nodes()[sets[0]].len() == 2 {
                new_node_10 = i;
            }
            // 20 is the only one in two sets of size 3
            if sets.len() == 2
                && game.get_sets_of_nodes()[sets[0]].len() == 3
                && game.get_sets_of_nodes()[sets[1]].len() == 3
            {
                new_node_20 = i;
            }
            // 50 is the only node that is on two sets and a set that has size 2
            if sets.len() == 2
                && (game.get_sets_of_nodes()[sets[0]].len() == 2
                    || game.get_sets_of_nodes()[sets[1]].len() == 2)
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
        use sorted_vec::SortedSet;

        // Construct a game with uniquely identifiable nodes by membership profile
        let original_sets = vec![
            SortedSet::from_unsorted(vec![99, 10, 50]), // Set 0
            SortedSet::from_unsorted(vec![50, 100, 20, 3, 99]), // Set 1
            SortedSet::from_unsorted(vec![20, 3, 4, 100]), // Set 2
        ];

        // Create the canonicalized parent game
        let parent_game = TakingGame::from_sets_of_nodes(original_sets.clone());

        let mut new_sets = parent_game.get_sets_of_nodes().clone();

        let new_node_99: usize = parent_game.nodes.iter().position(|n| *n == 99).unwrap();
        let new_node_100: usize = parent_game.nodes.iter().position(|n| *n == 100).unwrap();
        for set in &mut new_sets.iter_mut() {
            set.remove_item(&new_node_99);
            set.remove_item(&new_node_100);
        }
        let game = TakingGame::from_sets_of_nodes_with_node_map(new_sets, parent_game.nodes);

        let mut new_node_10: usize = 0;
        let mut new_node_20: usize = 0;
        let mut new_node_50: usize = 0;
        for i in 0..game.get_node_count() {
            let sets = &game.get_set_indices()[i];
            // 10 is the only node that is in one set  and a set that has size 2
            if sets.len() == 1 && game.get_sets_of_nodes()[sets[0]].len() == 2 {
                new_node_10 = i;
            }
            // 20 is the only one in two sets of size 3
            if sets.len() == 2
                && game.get_sets_of_nodes()[sets[0]].len() == 3
                && game.get_sets_of_nodes()[sets[1]].len() == 3
            {
                new_node_20 = i;
            }
            // 50 is the only node that is on two sets and a set that has size 2
            if sets.len() == 2
                && (game.get_sets_of_nodes()[sets[0]].len() == 2
                    || game.get_sets_of_nodes()[sets[1]].len() == 2)
            {
                new_node_50 = i;
            }
        }

        assert_eq!(game.nodes[new_node_10], 10);
        assert_eq!(game.nodes[new_node_20], 20);
        assert_eq!(game.nodes[new_node_50], 50);
    }
}
