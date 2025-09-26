use super::TakingGame;
use std::collections::HashSet;

impl TakingGame {
    /// Attempts to find a node-to-node symmetry of the game.
    ///
    /// A symmetry is a bijection on nodes that preserves the hypergraph structure.
    /// where no node is mapped to a node in the same hyperedge
    /// Returns `Some(vec)` if a valid mapping is found, where `vec[i]` is the node
    /// symmetric to `i`. Returns `None` if no symmetry exists.
    pub fn find_symmetry(&self) -> Option<Vec<usize>> {
        if self.nodes.len().is_multiple_of(2)
            && self.hyperedges.len().is_multiple_of(2)
            && self
                .node_structure_partitions
                .windows(2)
                .all(|w| (w[1] - w[0]).is_multiple_of(2))
            && self
                .edge_structure_partitions
                .windows(2)
                .all(|w| (w[1] - w[0]).is_multiple_of(2))
        {
            let neighbourhoods = self.get_neighbourhoods();
            let mut symmetries = vec![None; self.nodes.len()];
            self.generate_symmetry_from_sets_of_candidates(&mut symmetries, &neighbourhoods)
        } else {
            None
        }
    }

    /// Recursively pairs nodes into symmetric matches from candidate groups.
    ///
    /// Builds a full involutive mapping (`symmetries[node] = cand` and `symmetries[cand] = node`)
    /// by backtracking. Returns a completed mapping if successful, or `None` if no valid
    /// assignment exists.
    fn generate_symmetry_from_sets_of_candidates(
        &self,
        symmetries: &mut Vec<Option<usize>>,
        neighbourhoods: &Vec<HashSet<usize>>,
    ) -> Option<Vec<usize>> {
        if let Some(node) = symmetries.iter().position(|v| v.is_none()) {
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

    /// Finds all valid candidate matches for a node.
    ///
    /// Restricts candidates to the same structural partition and filters
    /// them with [`is_valid_match`].
    fn find_valid_candidates(
        &self,
        node: usize,
        symmetries: &[Option<usize>],
        neighbourhoods: &[HashSet<usize>],
    ) -> Vec<usize> {
        let partition = match self.node_structure_partitions.binary_search(&node) {
            Ok(v) => v,
            Err(v) => v - 1,
        };
        (self.node_structure_partitions[partition]..self.node_structure_partitions[partition + 1])
            .filter(|&cand| self.is_valid_match(node, cand, symmetries, neighbourhoods))
            .collect()
    }
    /// Checks whether two nodes can be symmetric partners.
    ///
    /// Conditions:
    /// - Nodes must be distinct and unmapped.
    /// - They must not share a hyperedge directly.
    /// - Already-mapped neighbors of `node` must map into neighbors of `candidate`.
    fn is_valid_match(
        &self,
        node: usize,
        candidate: usize,
        symmetries: &[Option<usize>],
        neighbourhoods: &[HashSet<usize>],
    ) -> bool {
        if node == candidate || symmetries[candidate].is_some() {
            return false;
        }

        if neighbourhoods[node].contains(&candidate) {
            return false;
        }

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

    /// Builds neighborhood lists for all nodes.
    ///
    /// Each entry contains the union of nodes sharing a hyperedge with the given node.
    fn get_neighbourhoods(&self) -> Vec<HashSet<usize>> {
        let mut neighbourhoods: Vec<HashSet<usize>> = vec![HashSet::new(); self.nodes.len()];
        let dual = self.hypergraph_dual();
        for node in 0..self.nodes.len() {
            for &e in &dual[node] {
                for neighbour in &self.hyperedges[e] {
                    neighbourhoods[node].insert(*neighbour);
                }
            }
        }
        neighbourhoods
    }
}

#[cfg(test)]
mod tests {
    use crate::Constructor;

    #[test]
    fn test_rect_4_8() {
        let g = Constructor::rect(4, 8).build_one();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_2_2() {
        let g = Constructor::hyper_cube(2, 2).build_one();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_2_4() {
        let g = Constructor::hyper_cube(2, 4).build_one();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_4_4() {
        let g = Constructor::hyper_cube(4, 4).build_one();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_2_32() {
        let g = Constructor::hyper_cube(2, 32).build_one();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_3_3() {
        let g = Constructor::hyper_cube(3, 3).build_one();
        assert!(g.find_symmetry().is_none());
    }
    #[test]
    fn test_hypertetrahedron_16() {
        let g = Constructor::hyper_tetrahedron(15).build_one();
        assert!(g.find_symmetry().is_none());
    }
}
