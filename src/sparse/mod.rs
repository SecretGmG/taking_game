use std::hash::{Hash, Hasher};

pub mod constructor;
pub mod util;

mod impartial;
mod new;
mod symmetries;

pub use constructor::Constructor;
/// A generalized representation of an impartial "taking game".
#[derive(Clone, Debug, Eq)]
pub struct TakingGame {
    hyperedges: Vec<Vec<usize>>,
    edge_structure_partitions: Vec<usize>,
    node_structure_partitions: Vec<usize>,
    nodes: Vec<usize>, //used to relate the new node indices with the original labels
}

impl Hash for TakingGame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hyperedges.hash(state);
    }
}
impl PartialEq for TakingGame {
    fn eq(&self, other: &Self) -> bool {
        self.hyperedges == other.hyperedges
    }
}
