use super::TakingGame;
use crate::util;

impl TakingGame {
    /// Attempts to find a node-to-node symmetry of the game graph.
    ///
    /// If the graph has an even number of nodes and passes a series of parity-based hash
    /// checks, this function searches for a bijection that maps nodes to their symmetric
    /// counterparts such that the local structure is preserved.
    ///
    /// Returns `Some(vec)` if a valid symmetry mapping exists, where `vec[i]` gives the
    /// node symmetric to `i`. Returns `None` if no such symmetry is found
    pub fn find_symmetry(&self) -> Option<Vec<usize>> {
        let neighbourhoods = self.get_neighbourhoods();

        let mut symmetries = vec![None; self.nodes.len()];
        self.generate_symmetry_from_sets_of_candidates(&mut symmetries, &neighbourhoods)
    }

    /// Recursively attempts to pair nodes into symmetric matches from candidate groups.
    ///
    /// Fills in the `symmetries` vector with a valid involutive mapping
    /// (symmetries[node] = node' <=> symmetries[node'] = node), if possible.
    /// Returns the completed symmetry if successful, or `None` on backtracking failure.
    fn generate_symmetry_from_sets_of_candidates(
        &self,
        symmetries: &mut Vec<Option<usize>>,
        neighbourhoods: &Vec<Vec<usize>>,
    ) -> Option<Vec<usize>> {
        if let Some(node) = Self::find_unmatched_node(symmetries) {
            let candidates = self.find_valid_candidates(node, symmetries, neighbourhoods);
            for cand in candidates {
                symmetries[node] = Some(cand);
                symmetries[cand] = Some(node);

                if let Some(result) =
                    self.generate_symmetry_from_sets_of_candidates(symmetries, neighbourhoods)
                {
                    return Some(result);
                }

                symmetries[node] = None;
                symmetries[cand] = None;
            }
            return None;
        }

        // All nodes are matched
        Some(symmetries.iter().map(|x| x.unwrap()).collect())
    }
    /// Finds the first node that hasn't been matched in the current symmetry assignment.
    ///
    /// Returns the index of the unmatched node or `None` if all nodes are matched.
    fn find_unmatched_node(symmetries: &[Option<usize>]) -> Option<usize> {
        symmetries.iter().position(|v| v.is_none())
    }
    /// Returns all valid symmetry candidates for a given node.
    ///
    /// Filters candidates from the same hash group that are unmatched and compatible
    /// with the current symmetry mapping and local neighbourhood structure.
    fn find_valid_candidates(
        &self,
        node: usize,
        symmetries: &[Option<usize>],
        neighbourhoods: &[Vec<usize>],
    ) -> Vec<usize> {
        let partition = match self.node_structure_partitions.binary_search(&node) {
            Ok(v) => v,
            Err(v) => v - 1,
        };
        (self.node_structure_partitions[partition]..self.node_structure_partitions[partition + 1])
            .filter(|&cand| self.is_valid_match(node, cand, symmetries, neighbourhoods))
            .collect()
    }
    /// Checks whether two nodes can be valid symmetry partners.
    ///
    /// Ensures they are not directly connected, and that mapped neighbours
    /// of `node` also exist in the neighbourhood of `candidate`.
    fn is_valid_match(
        &self,
        node: usize,
        candidate: usize,
        symmetries: &[Option<usize>],
        neighbourhoods: &[Vec<usize>],
    ) -> bool {
        if node == candidate || symmetries[candidate].is_some() {
            return false;
        }

        // Check if they share a set -> immediate disqualification
        if neighbourhoods[node].binary_search(&candidate).is_ok() {
            return false;
        }

        // All neighbors of node (that are already mapped) must have their images in candidate's neighborhood
        let candidate_neighbours = &neighbourhoods[candidate];

        for &neighbour in &neighbourhoods[node] {
            if let Some(mapped) = symmetries[neighbour] {
                if !candidate_neighbours.contains(&mapped) {
                    return false;
                }
            }
        }
        true
    }
    /// Constructs a neighbourhood list for all nodes in the graph.
    ///
    /// Each entry contains a set of all nodes directly connected to the given node
    /// via shared hyperedges.
    fn get_neighbourhoods(&self) -> Vec<Vec<usize>> {
        let mut neighbourhoods: Vec<Vec<usize>> = vec![Vec::new(); self.nodes.len()];
        let dual = self.hypergraph_dual();
        for node in 0..self.nodes.len() {
            for &e in &dual[node] {
                util::union_append(&mut neighbourhoods[node], &self.hyperedges[e]);
            }
        }
        neighbourhoods
    }
}

#[cfg(test)]
mod tests {

    use crate::Constructor;

    #[test]
    fn test_hypercube_2_2() {
        let g = Constructor::hyper_cube(2, 2).build();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_2_4() {
        let g = Constructor::hyper_cube(2, 4).build();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_4_4() {
        let g = Constructor::hyper_cube(4, 4).build();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_2_32() {
        let g = Constructor::hyper_cube(2, 32).build();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_3_3() {
        let g = Constructor::hyper_cube(3, 3).build();
        assert!(g.find_symmetry().is_none());
    }
    #[test]
    fn test_hypertetrahedron_16() {
        let g = Constructor::hyper_tetrahedron(15).build();
        assert!(g.find_symmetry().is_none());
    }
}
