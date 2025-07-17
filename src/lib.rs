use std::hash::{Hasher, Hash};

use sorted_vec::SortedSet;

pub mod impls;
pub mod constructor;
pub mod new;
pub mod symmetries;
pub mod util;

pub use constructor::Constructor;


/// A generalized representation of an impartial "taking game".
/// 
/// This struct implements tools to efficiently compute the nimber 
/// (Grundy number) for complex taking games by modeling them as 
/// collections of node sets.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TakingGame {
    sets_of_nodes: Vec<SortedSet<usize>>,
    set_indices: Vec<Vec<usize>>,
    node_count: usize,
}
impl TakingGame {
    
    pub fn get_sets_of_nodes(&self) -> &Vec<SortedSet<usize>> {
        &self.sets_of_nodes
    }
    pub fn get_node_count(&self) -> usize {
        self.node_count
    }
    pub fn get_set_indices(&self) -> &Vec<Vec<usize>> {
        &self.set_indices
    }
}

impl Hash for TakingGame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sets_of_nodes.hash(state)
    }
}

