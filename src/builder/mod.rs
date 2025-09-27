use rand::{rng, Rng};
use std::vec;

use crate::taking_game::TakingGame;

/// A helper struct for constructing `TakingGame` instances from various configurations.
///
/// Provides utilities for buid_one()ing graphs from hyperedges, performing transformations
/// like extrusion and connection, and generating standard structures (e.g., grids, cubes).
#[derive(PartialEq, Eq, Debug)]
pub struct Builder {
    hyperedges: Vec<Vec<usize>>,
}
impl Builder {
    pub fn get_nodes(&self) -> Vec<usize> {
        let mut nodes: Vec<usize> = self.hyperedges.iter().flatten().copied().collect();
        nodes.sort();
        nodes.dedup();
        nodes
    }
    pub fn get_max_node(&self) -> usize {
        self.get_nodes().pop().unwrap_or(0)
    }
    /// Creates a `Builder` from a given list of sets of nodes (hyperedges).
    pub fn from_hyperedges(hyperedges: Vec<Vec<usize>>) -> Builder {
        Builder { hyperedges }
    }
    /// Returns a graph with one empty set (no nodes).
    pub fn empty() -> Builder {
        Builder::from_hyperedges(vec![vec![]])
    }
    /// Returns a graph with one set containing a single node.
    pub fn unit() -> Builder {
        Builder::from_hyperedges(vec![vec![0]])
    }
    pub fn heap(size: usize) -> Builder {
        Builder::from_hyperedges(vec![(0..size).collect()])
    }
    /// Constructs a Kayles game of the given size.
    ///
    /// Each set connects two adjacent nodes. Returns `empty()` if size == 0,
    /// and `unit()` if size == 1.
    pub fn kayles(size: usize) -> Builder {
        if size == 0 {
            return Builder::empty();
        }
        if size == 1 {
            return Builder::unit();
        }
        let mut hyperedges = vec![];
        for i in 1..size {
            hyperedges.push(vec![i - 1, i]);
        }
        Builder::from_hyperedges(hyperedges)
    }
    /// Generates a random hypergraph with the given number of nodes and sets.
    ///
    /// Each node is connected to a random number of sets, within the given bounds.
    pub fn rand(
        node_count: usize,
        set_count: usize,
        min_sets_per_node: usize,
        max_sets_per_node: usize,
    ) -> Builder {
        let mut hyperedges = vec![Vec::new(); set_count];
        for node in 0..node_count {
            for _ in 0..(rng().random_range(min_sets_per_node..max_sets_per_node)) {
                hyperedges[rng().random_range(..set_count)].push(node);
            }
        }
        Builder::from_hyperedges(hyperedges)
    }

    /// Constructs a triangular grid of side length `l` using 3-directional diagonals.
    ///
    /// Each set runs in one of the three directions across the grid.
    pub fn triangle(l: usize) -> Builder {
        let mut hyperedges = vec![];
        for i in 0..l {
            let mut h1 = Vec::new();
            let mut h2 = Vec::new();
            let mut h3 = Vec::new();
            for j in 0..(l - i) {
                /*
                12# # #
                8 9 # #
                4 5 6 #
                0 1 2 3
                */
                h1.push(i + j * l);
                h2.push(j + i * l);
                h3.push(l - 1 - i + j * (l - 1));
            }
            hyperedges.push(h1);
            hyperedges.push(h2);
            hyperedges.push(h3);
        }
        Builder::from_hyperedges(hyperedges)
    }
    /// Constructs a 2D rectangular grid of size x by y.
    pub fn rect(x: usize, y: usize) -> Builder {
        Self::hyper_cuboid(vec![x, y])
    }
    /// Constructs a hypercube of dimension `dim` and side length `l` in each dimension.
    ///
    /// Uses `hyper_cuboid` internally.
    pub fn hyper_cube(dim: usize, l: usize) -> Builder {
        Self::hyper_cuboid(vec![l; dim])
    }
    /// Constructs a hypercuboid with the given lengths along each axis.
    ///
    /// Built by repeatedly extruding a unit graph.
    pub fn hyper_cuboid(lengths: Vec<usize>) -> Builder {
        if lengths.contains(&0) {
            return Builder::empty();
        }
        let mut g = Self::unit();
        for length in lengths {
            g = g.extrude(length);
        }
        g
    }
    /// Constructs a hyper-tetrahedron of the given dimension.
    ///
    /// Iteratively connects a new unit node to all existing nodes at each step.
    pub fn hyper_tetrahedron(dim: usize) -> Builder {
        let mut g = Self::unit();
        for _ in 0..dim {
            g = g.connect_unit_to_all();
        }
        g
    }
    pub fn build(self) -> Vec<TakingGame> {
        TakingGame::from_hyperesges(self.hyperedges)
    }
    pub fn build_one(self) -> Option<TakingGame> {
        let mut games = self.build();
        games.sort_by_key(|g| g.nr_nodes());
        games.pop()
    }
    /// Connects a single-node unit graph to all existing nodes in the current graph.
    ///
    /// Returns the combined structure.
    pub fn connect_unit_to_all(self) -> Builder {
        self.fully_connect(&Self::unit())
    }
    /// Fully connects the current graph to another `TakingGame`.
    ///
    /// Adds pairwise sets between all nodes of `self` and the other game,
    /// and appends all sets from the other game (offset appropriately).
    pub fn fully_connect(mut self, other: &Self) -> Builder {
        let self_nodes = self.get_nodes();
        let other_nodes = other.get_nodes();
        let shift = self.get_max_node() + 1;
        for e in &other.hyperedges {
            self.hyperedges.push(e.iter().map(|n| n + shift).collect());
        }
        for i in &self_nodes {
            for j in &other_nodes {
                self.hyperedges.push(vec![*i, *j + shift]);
            }
        }
        self
    }
    /// Extrudes the current graph `l` times along a new dimension.
    ///
    /// Duplicates all sets `l` times with increasing node offsets,
    /// and adds alignment sets connecting corresponding nodes across layers.
    pub fn extrude(mut self, l: usize) -> Builder {
        let old_hyperedges = self.hyperedges.clone();
        let shift = self.get_max_node() + 1;

        for edge in &old_hyperedges {
            for offset in 0..l {
                let mut new_edge = Vec::new();
                for node in edge {
                    new_edge.push(node + offset * shift);
                }
                self.hyperedges.push(new_edge);
            }
        }
        for node in 0..shift {
            let mut new_set = Vec::new();
            for offset in 0..l {
                new_set.push(node + offset * shift);
            }
            self.hyperedges.push(new_set);
        }
        self
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_unit_heap() {
        let empty = Builder::empty();
        assert_eq!(empty.hyperedges, vec![vec![]]);
        assert_eq!(empty.get_nodes(), vec![]);

        let unit = Builder::unit();
        assert_eq!(unit.hyperedges, vec![vec![0]]);
        assert_eq!(unit.get_nodes(), vec![0]);

        let heap = Builder::heap(3);
        assert_eq!(heap.hyperedges, vec![vec![0, 1, 2]]);
        assert_eq!(heap.get_nodes(), vec![0, 1, 2]);
    }

    #[test]
    fn test_kayles() {
        let g0 = Builder::kayles(0);
        assert_eq!(g0.hyperedges, vec![vec![]]);

        let g1 = Builder::kayles(1);
        assert_eq!(g1.hyperedges, vec![vec![0]]);

        let g3 = Builder::kayles(3);
        assert_eq!(g3.hyperedges, vec![vec![0, 1], vec![1, 2]]);
        let nodes: Vec<usize> = g3.get_nodes();
        assert_eq!(nodes, vec![0, 1, 2]);
    }

    #[test]
    fn test_connect_unit_to_all() {
        let base = Builder::unit();
        let g = base.connect_unit_to_all();
        let nodes = g.get_nodes();
        assert!(nodes.len() >= 2);
        for e in &g.hyperedges {
            assert!(!e.is_empty());
        }
    }

    #[test]
    fn test_fully_connect() {
        let a = Builder::unit();
        let b = Builder::heap(2);
        let max_node_a = a.get_max_node();
        let c = a.fully_connect(&b);
        let nodes = c.get_nodes();
        assert!(nodes.len() >= 3); // 1 from a, 2 from b
                                   // Each node from a should be connected to each node from b
        let max_node_b = b.get_max_node();
        for e in &c.hyperedges {
            for &n in e {
                assert!(n <= max_node_a + max_node_b + 1);
            }
        }
    }

    #[test]
    fn test_extrude() {
        let base = Builder::unit();
        let extruded = base.extrude(3);
        let nodes = extruded.get_nodes();
        assert!(nodes.len() > 1);
        // All layers contain at least the original node
        for i in 0..3 {
            assert!(nodes.contains(&(i)));
        }
    }

    #[test]
    fn test_triangle_rect_hypercube() {
        let tri = Builder::triangle(3);
        assert!(!tri.hyperedges.is_empty());

        let rect = Builder::rect(2, 3);
        assert!(!rect.hyperedges.is_empty());

        let cube = Builder::hyper_cube(2, 2);
        assert!(!cube.hyperedges.is_empty());
    }

    #[test]
    fn test_hyper_tetrahedron() {
        let tet = Builder::hyper_tetrahedron(2);
        assert!(!tet.hyperedges.is_empty());
        assert!(!tet.get_nodes().is_empty());
    }

    #[test]
    fn test_get_max_node() {
        let c = Builder::from_hyperedges(vec![vec![1, 2], vec![3]]);
        assert_eq!(c.get_max_node(), 3);

        let empty = Builder::empty();
        assert_eq!(empty.get_max_node(), 0);
    }

    #[test]
    fn test_build_and_build_one() {
        let c = Builder::unit();
        let games = c.build();
        assert!(!games.is_empty());
        let c = Builder::unit();
        let one_game = c.build_one();
        assert!(one_game.is_some());
        assert_eq!(one_game.unwrap().nr_nodes(), 1);
    }

    #[test]
    fn test_rand() {
        let r = Builder::rand(5, 3, 1, 3);
        let nodes = r.get_nodes();
        assert!(nodes.len() <= 5);
        assert!(r.hyperedges.len() == 3);
    }
}
