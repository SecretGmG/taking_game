use std::hash::{Hash, Hasher};

pub mod constructor;
pub mod util;

mod impartial;
mod new;
mod symmetries;

pub use constructor::DenseConstructor;
/// A generalized representation of an impartial "taking game".
#[derive(Clone, Debug, Eq)]
pub struct DenseTakingGame {
    hyperedges: Vec<u128>, // hyperedges as bitmasks
    edge_structure_partitions: Vec<usize>,
    node_structure_partitions: Vec<usize>,
    nodes: Vec<usize>, // original labels
}
impl Hash for DenseTakingGame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hyperedges.hash(state);
    }
}
impl PartialEq for DenseTakingGame {
    fn eq(&self, other: &Self) -> bool {
        self.hyperedges == other.hyperedges
    }
}
