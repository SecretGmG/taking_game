use std::hash::{Hash, Hasher};

pub mod constructor;
pub mod util;

mod impartial;
mod new;
mod symmetries;

pub use constructor::Constructor;
/// A generalized representation of an impartial "taking game".
#[derive(Clone, Debug)]
pub struct TakingGame {
    hyperedges: Vec<Vec<usize>>,
    edge_structure_partitions: Vec<usize>,
    node_structure_partitions: Vec<usize>,
    nodes: Vec<usize>, //used to relate the now node indices with the original values
                       // unconnected_nodes: Vec<Vec<usize>>, //used to relate the edge indices with the original values
}
impl TakingGame {
    pub fn get_unconnected_node_counts(&self) -> Vec<usize> {
        // self.unconnected_nodes
        //     .iter()
        //     .map(|nodes| nodes.len())
        //     .collect::<Vec<usize>>()
        todo!()
    }
}

impl Hash for TakingGame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hyperedges.hash(state);
        //self.get_unconnected_node_counts().hash(state);
    }
}
impl PartialEq for TakingGame {
    fn eq(&self, other: &Self) -> bool {
        self.hyperedges == other.hyperedges
        //&& self.get_unconnected_node_counts() == other.get_unconnected_node_counts()
    }
}
impl Eq for TakingGame {}
