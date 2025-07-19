use std::hash::{Hasher, Hash};

use sorted_vec::SortedSet;

pub mod constructor;
pub mod util;

mod impartial;
mod new;
mod symmetries;

pub use constructor::Constructor;


/// A generalized representation of an impartial "taking game".
/// 
/// This struct implements tools to efficiently compute the nimber 
/// (Grundy number) for complex taking games by modeling them as 
/// collections of node sets.
/// 
/// Only `sets_of_nodes` is hashed and compared to ensure semantic identity across games.
/// `set_indices` and `nodes` are derived/cached for efficient queries.
#[derive(Clone, Debug)]
pub struct TakingGame {
    /// Each element is a SortedSet of node indices representing nodes that can be 'taken' in a move.
    ///
    /// These sets are canonicalized for efficiency
    sets_of_nodes: Vec<SortedSet<usize>>,
    /// For each node (by its canonical index), stores a vector of indices of
    /// all sets in `sets_of_nodes` that contain this node.
    ///
    /// This is used to quickly determine which moves involve a given node,
    /// speeding up game logic like move validation and adjacency queries.
    set_indices: Vec<Vec<usize>>,
    /// Maps canonical node indices back to original node labels.
    ///
    /// Used to relate moves and game states to the original problem context,
    /// especially important when working with subgames derived from a parent game.
    nodes: Vec<usize>,
}
impl TakingGame {
    /// Each element is a SortedSet of node indices representing a node that can be 'taken' in a move.
    ///
    /// These sets are canonicalized for efficiency.
    pub fn get_sets_of_nodes(&self) -> &[SortedSet<usize>] {
        &self.sets_of_nodes
    }
    pub fn get_node_count(&self) -> usize {
        self.nodes.len()
    }
    /// Maps canonical node indices back to original node labels.
    ///
    /// Used to relate moves and game states to the original problem context,
    /// especially important when working with subgames derived from a parent game.
    pub fn get_nodes(&self) -> &[usize] {
        &self.nodes
    }
    /// For each node (by its canonical index), stores a vector of indices of
    /// all sets in `sets_of_nodes` that contain this node.
    ///
    /// This is used to quickly determine which moves involve a given node,
    /// speeding up game logic like move validation and adjacency queries.
    pub fn get_set_indices(&self) -> &[Vec<usize>] {
        &self.set_indices
    }
}

impl Hash for TakingGame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sets_of_nodes.hash(state)
    }
}
impl PartialEq for TakingGame{
    fn eq(&self, other: &Self) -> bool {
        self.sets_of_nodes == other.sets_of_nodes
    }
}
impl Eq for TakingGame{}
