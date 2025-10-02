use std::fmt::Display;
use std::hash::{Hash, Hasher};
mod impartial;
mod symmetries;

use crate::hypergraph::Bitset128;
use crate::hypergraph::Set;
use crate::hypergraph::StructuredHypergraph;

/// A generalized representation of an impartial "taking game".
#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct TakingGame {
    graph: StructuredHypergraph<Bitset128>,
}
impl TakingGame {
    pub fn from_hyperesges(edges: Vec<Vec<usize>>) -> Vec<Self> {
        StructuredHypergraph::from_hyperedges(
            edges.iter().map(|s| Bitset128::from_slice(s)).collect(),
        )
        .into_iter()
        .map(|graph| Self { graph })
        .collect()
    }
    pub fn nr_nodes(&self) -> usize {
        self.graph.nr_nodes()
    }
    pub fn nodes(&self) -> &[usize] {
        self.graph.nodes()
    }
}
impl Display for TakingGame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Taking Game:")?;
        self.graph.fmt(f)
    }
}
impl Hash for TakingGame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.graph.hash(state);
    }
}
