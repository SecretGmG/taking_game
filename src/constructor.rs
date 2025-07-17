use super::TakingGame;
use rand::{rng, Rng};
use sorted_vec::SortedSet;
use std::vec;

/// A helper struct for constructing `TakingGame` instances from various configurations.
///
/// Provides utilities for building graphs from hyperedges, performing transformations
/// like extrusion and connection, and generating standard structures (e.g., grids, cubes).
pub struct Constructor {
    g: TakingGame,
}
impl Constructor {
    /// Creates a `Constructor` from a given list of sets of nodes (hyperedges).
    pub fn from_sets_of_nodes(sets_of_nodes: Vec<SortedSet<usize>>) -> Constructor{
        Constructor { g: TakingGame::from_sets_of_nodes(sets_of_nodes) }
    }
    /// Creates a `Constructor` from a list of node vectors by converting them to sorted sets.
    pub fn from_vecs_of_nodes(vecs_of_nodes: Vec<Vec<usize>>) -> Constructor{
        Self::from_sets_of_nodes(
            vecs_of_nodes
            .into_iter()
            .map(SortedSet::from_unsorted)
            .collect()
        )
    }
    /// Returns a graph with one empty set (no nodes).
    pub fn empty() -> Constructor{
        Constructor::from_vecs_of_nodes(vec![vec![]])
    }
    /// Returns a graph with one set containing a single node.
    pub fn unit() -> Constructor {
        Constructor::from_vecs_of_nodes(vec![vec![0]])
    }
    /// Constructs a Kayles game of the given size.
    ///
    /// Each set connects two adjacent nodes. Returns `empty()` if size == 0,
    /// and `unit()` if size == 1.
    pub fn kayles(size: usize) -> Constructor{
        if size == 0 {
            return Constructor::empty();
        }
        if size == 1 {
            return Constructor::unit();
        }
        let mut sets_of_nodes = vec![];
        for i in 1..size{
            sets_of_nodes.push(SortedSet::from_unsorted(vec![i-1, i]));
        }
        Constructor::from_sets_of_nodes(sets_of_nodes)
    }
    /// Generates a random hypergraph with the given number of nodes and sets.
    ///
    /// Each node is connected to a random number of sets, within the given bounds.
    pub fn rand(
        node_count: usize,
        set_count: usize,
        min_sets_per_node: usize,
        max_sets_per_node: usize,
    ) -> Constructor {
        let mut sets_of_nodes = vec![SortedSet::new(); set_count ];
        for node in 0..node_count {
            for _ in 0..(rng().random_range(min_sets_per_node..max_sets_per_node)) {
                sets_of_nodes[rng().random_range(..set_count) ].push(node);
            }
        }
        Constructor::from_sets_of_nodes(sets_of_nodes)
    }
    
    /// Constructs a triangular grid of side length `l` using 3-directional diagonals.
    ///
    /// Each set runs in one of the three directions across the grid.
    pub fn triangle(l: usize) -> Constructor {
        let mut sets_of_nodes = vec![];
        for i in 0..l {
            let mut new_set_of_nodes1 = SortedSet::new();
            let mut new_set_of_nodes2 = SortedSet::new();
            let mut new_set_of_nodes3 = SortedSet::new();
            for j in 0..(l - i) {
                /*
                12# # #
                8 9 # #
                4 5 6 #
                0 1 2 3
                */
                new_set_of_nodes1.push(i + j * l);
                new_set_of_nodes2.push(j + i * l);
                new_set_of_nodes3.push(l - 1 - i + j * (l - 1));
            }
            sets_of_nodes.push(new_set_of_nodes1);
            sets_of_nodes.push(new_set_of_nodes2);
            sets_of_nodes.push(new_set_of_nodes3);
        }
        Constructor::from_sets_of_nodes(sets_of_nodes)
    }
    /// Constructs a 2D rectangular grid of size x by y.
    pub fn rect(x: usize, y: usize) -> Constructor {
        Self::hyper_cuboid(vec![x, y])
    }
    /// Constructs a hypercube of dimension `dim` and side length `l` in each dimension.
    ///
    /// Uses `hyper_cuboid` internally.
    pub fn hyper_cube(dim: usize, l: usize) -> Constructor {
        Self::hyper_cuboid(vec![l; dim ])
    }
    /// Constructs a hypercuboid with the given lengths along each axis.
    ///
    /// Built by repeatedly extruding a unit graph.
    pub fn hyper_cuboid(lengths: Vec<usize>) -> Constructor {
        let mut g = Self::unit();
        for length in lengths {
            g = g.extrude(length);
        }
        g
    }
    /// Constructs a hyper-tetrahedron of the given dimension.
    ///
    /// Iteratively connects a new unit node to all existing nodes at each step.
    pub fn hyper_tetrahedron(dim: usize) -> Constructor {
        let mut g = Self::unit();
        for _ in 0..dim {
            g = g.connect_unit_to_all();
        }
        g
    }
    /// Finalizes the graph and returns the underlying `TakingGame`.
    pub fn build(self) -> TakingGame{
        self.g
    }
    /// Connects a single-node unit graph to all existing nodes in the current graph.
    ///
    /// Returns the combined structure.
    pub fn connect_unit_to_all(self) -> Constructor {
        self.fully_connect(&Self::unit().build())
    }
    /// Fully connects the current graph to another `TakingGame`.
    ///
    /// Adds pairwise sets between all nodes of `self` and the other game,
    /// and appends all sets from the other game (offset appropriately).
    pub fn fully_connect(mut self, g:&TakingGame) -> Constructor {
        let node_count = self.g.get_node_count();
        let mut new_sets_of_nodes = self.g.get_sets_of_nodes().clone();
        for set in g.get_sets_of_nodes() {
            new_sets_of_nodes.push(SortedSet::from_unsorted(set.iter().map(|n| n + node_count).collect()));
        }
        for i in 0..node_count {
            for j in node_count..(node_count + g.get_node_count()) {
                new_sets_of_nodes.push(SortedSet::from_unsorted(vec![i, j]));
            }
        }
        self.g = TakingGame::from_sets_of_nodes(new_sets_of_nodes);
        self
    }
    /// Appends a new `TakingGame` to the current one without adding any connecting sets.
    ///
    /// The node indices of the appended game are offset to avoid collisions.
    pub fn combine(self, g: TakingGame) -> Constructor{
        let mut new_sets_of_nodes = self.g.get_sets_of_nodes().clone();
        let node_count = self.g.get_node_count();
        for set_of_nodes in g.get_sets_of_nodes() {
            new_sets_of_nodes.push(SortedSet::from_unsorted(set_of_nodes.iter().map(|n| n + node_count).collect()));
        }
        Self::from_sets_of_nodes(new_sets_of_nodes)
    }
    /// Extrudes the current graph `l` times along a new dimension.
    ///
    /// Duplicates all sets `l` times with increasing node offsets,
    /// and adds alignment sets connecting corresponding nodes across layers.
    pub fn extrude(mut self, l: usize) -> Constructor {
        let mut new_sets_of_nodes = self.g.get_sets_of_nodes().clone();
        let node_count = self.g.get_node_count();

        for set in self.g.get_sets_of_nodes() {
            for offset in 0..l {
                let mut new_set_of_nodes = SortedSet::new();
                for node in set {
                    new_set_of_nodes.push(node + offset * node_count);
                }
                new_sets_of_nodes.push(new_set_of_nodes);
            }
        }
        for node in 0..node_count {
            let mut new_set = SortedSet::new();
            for offset in 0..l {
                new_set.push(node + offset * node_count);
            }
            new_sets_of_nodes.push(new_set);
        }
        self.g = TakingGame::from_sets_of_nodes(new_sets_of_nodes);
        self
    }
}
