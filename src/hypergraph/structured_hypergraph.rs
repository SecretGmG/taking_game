use core::hash;
use std::{cmp::Reverse, collections::HashMap, mem, ops::Range};
use union_find::{QuickUnionUf, UnionByRank, UnionFind};

use crate::hypergraph::Set;

#[derive(Clone, Eq)]
pub struct StructuredHypergraph<E>
where
    E: Set,
{
    hyperedges: Vec<E>,
    nodes: Vec<usize>,
    node_structure_partitions: Vec<usize>,
    edge_structure_partitions: Vec<usize>,
}

impl<E> PartialEq for StructuredHypergraph<E>
where
    E: Set,
{
    fn eq(&self, other: &Self) -> bool {
        self.hyperedges == other.hyperedges
    }
}

impl<E> hash::Hash for StructuredHypergraph<E>
where
    E: Set,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hyperedges.hash(state);
    }
}

impl<E> StructuredHypergraph<E>
where
    E: Set,
{
    /// Returns the number of nodes in the hypergraph.
    pub fn nr_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Returns a slice of node indices.
    pub fn nodes(&self) -> &[usize] {
        &self.nodes
    }

    /// Returns true if the hypergraph has no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns a slice of hyperedges.
    pub fn hyperedges(&self) -> &[E] {
        &self.hyperedges
    }

    /// Returns a vector of ranges representing partitions of hyperedges.
    pub fn get_edge_partitions(&self) -> Vec<Range<usize>> {
        self.edge_structure_partitions
            .windows(2)
            .map(|w| w[0]..w[1])
            .collect()
    }

    /// Returns a vector of ranges representing partitions of nodes.
    pub fn get_node_partitions(&self) -> Vec<Range<usize>> {
        self.node_structure_partitions
            .windows(2)
            .map(|w| w[0]..w[1])
            .collect()
    }

    /// Removes the given nodes and returns resulting hypergraph components.
    pub fn minus(&self, nodes: E) -> Vec<Self> {
        Self::from_hyperedges_with_nodes(
            self.hyperedges.iter().map(|e| e.minus(&nodes)).collect(),
            self.nodes.clone(),
        )
    }

    /// Constructs hypergraphs from raw hyperedges.
    pub fn from_hyperedges(hyperedges: Vec<E>) -> Vec<StructuredHypergraph<E>> {
        let max_node = hyperedges
            .iter()
            .flat_map(|e| e.iter())
            .max()
            .map(|max| max + 1)
            .unwrap_or_default();
        let nodes: Vec<usize> = (0..max_node).collect();
        Self::from_hyperedges_with_nodes(hyperedges, nodes)
    }

    /// Constructs hypergraphs from hyperedges and explicit nodes.
    ///
    /// Assumptions:
    /// - `nodes` must label all nodes in `hyperedges`.
    pub fn from_hyperedges_with_nodes(
        hyperedges: Vec<E>,
        nodes: Vec<usize>,
    ) -> Vec<StructuredHypergraph<E>> {
        let mut g = Self {
            hyperedges,
            edge_structure_partitions: Vec::new(),
            node_structure_partitions: Vec::new(),
            nodes,
        };
        g.remove_redundant_hyperedges();
        g.get_parts()
    }

    /// Returns the dual hypergraph representation.
    ///
    /// Each node is mapped to the list of incident hyperedges.
    pub fn dual(&self) -> Vec<Vec<usize>> {
        let mut dual = vec![Vec::new(); self.nodes.len()];
        for (i, edge) in self.hyperedges.iter().enumerate() {
            for node in edge.iter() {
                dual[node].push(i);
            }
        }
        dual
    }

    /// Normalize node indices:
    /// - Maps arbitrary node labels to a compact range [0..N).
    /// - Updates both `hyperedges` and `nodes`.
    ///
    /// Assumes `nodes` is consistent with hyperedges.
    fn flatten_nodes(&mut self) {
        let mut all_nodes = E::default();

        // union of all nodes appearing in hyperedges
        self.hyperedges.iter().for_each(|e| all_nodes.union(e));

        // if already sequential 0..N-1, just truncate
        if all_nodes.is_flattened() {
            self.nodes.truncate(all_nodes.len());
            return;
        }

        let mut node_map: Vec<usize> = Vec::with_capacity(all_nodes.len());
        node_map.extend(all_nodes.iter());
        debug_assert_eq!(node_map.len(), all_nodes.len());
        self.apply_node_map(&node_map);
    }

    fn remove_redundant_hyperedges(&mut self) {
        self.flatten_nodes();
        // sort largest hyperedges first
        self.hyperedges.sort_by_cached_key(|e| Reverse(e.len()));

        let mut new_edges = Vec::new();
        let prev_hyperedges_len = self.hyperedges.len();
        for e in self.hyperedges.drain(..) {
            // only add hyperedge if it is not empty and not a subset of existing hyperedges
            if !e.is_empty() && new_edges.iter().all(|ue| !e.is_subset(ue)) {
                new_edges.push(e);
            }
        }
        self.hyperedges = new_edges;
        if prev_hyperedges_len != self.hyperedges.len() {
            // re-flatten after removal to ensure consistent node mapping
            self.flatten_nodes();
        }
    }

    /// Returns disconnected parts of the hypergraph as separate StructuredHypergraphs.
    fn get_parts(mut self) -> Vec<StructuredHypergraph<E>> {
        let mut uf: QuickUnionUf<UnionByRank> = QuickUnionUf::new(self.nodes.len());

        // Union all nodes in each hyperedge
        for e in &self.hyperedges {
            let mut iter = e.iter();
            if let Some(first) = iter.next() {
                for node in iter {
                    uf.union(first, node);
                }
            }
        }

        // Group hyperedges by root node of any member
        let mut buckets: HashMap<usize, Vec<usize>> = HashMap::with_capacity(2);
        for e in 0..self.hyperedges.len() {
            let representative = self.hyperedges[e]
                .iter()
                .next()
                .expect("every hyperedge should be non-empty");
            let root = uf.find(representative);
            match buckets.get_mut(&root) {
                Some(v) => v.push(e),
                None => {
                    buckets.insert(root, vec![e]);
                }
            };
        }

        if buckets.len() == 1 {
            return vec![StructuralHypergraphSorter::new(self).sort()];
        }

        let mut parts = Vec::with_capacity(buckets.len());
        for edges in buckets.values() {
            let mut part = StructuredHypergraph {
                hyperedges: edges
                    .iter()
                    .map(|e| mem::take(&mut self.hyperedges[*e]))
                    .collect(),
                nodes: self.nodes.clone(),
                node_structure_partitions: vec![],
                edge_structure_partitions: vec![],
            };
            part.flatten_nodes();
            parts.push(StructuralHypergraphSorter::new(part).sort());
        }
        parts
    }

    /// Applies a permutation to reorder edges.
    /// Assumes map contains a valid permutation of edge indices.
    fn apply_edge_map(&mut self, map: &[usize]) {
        let mut old_edges: Vec<Option<E>> =
            self.hyperedges.drain(..).map(|node| Some(node)).collect();
        self.hyperedges = map
            .iter()
            .map(|&new_idx| {
                old_edges[new_idx]
                    .take()
                    .expect("every label should only be accessed once")
            })
            .collect();
    }

    /// Applies a permutation to reorder nodes.
    /// Also updates hyperedges to reflect new node indices.
    /// Assumes `map` is a valid reordering of [0..nodes).
    fn apply_node_map(&mut self, map: &[usize]) {
        for edge in self.hyperedges.iter_mut() {
            edge.apply_node_map(map);
        }
        let old_nodes = mem::take(&mut self.nodes);
        self.nodes.resize(old_nodes.len(), 0);
        self.nodes = map.iter().map(|&old_idx| old_nodes[old_idx]).collect();
    }
}

use std::fmt;

impl<E> fmt::Display for StructuredHypergraph<E>
where
    E: Set,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.nodes.is_empty() {
            return writeln!(f, "Empty hypergraph");
        }

        // Determine column width dynamically based on largest node number
        let max_node = *self.nodes().iter().max().unwrap_or(&0);
        let col_width = max_node.to_string().len().max(3); // at least width 3

        // Print node headers
        write!(f, "Nodes:  ")?;
        for &n in self.nodes() {
            write!(f, "{:>width$} ", n, width = col_width)?;
        }
        writeln!(f)?;

        // Print edges
        writeln!(f, "Edges:")?;
        for e in &self.hyperedges {
            write!(f, "        ")?;
            for (i, &n) in self.nodes().iter().enumerate() {
                if e.contains(&i) {
                    write!(f, "{:>width$} ", n, width = col_width)?;
                } else {
                    write!(f, "{:width$} ", "", width = col_width)?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl<E> fmt::Debug for StructuredHypergraph<E>
where
    E: Set,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

struct StructuralHypergraphSorter<E>
where
    E: Set,
{
    node_map: Vec<usize>,
    edge_map: Vec<usize>,

    temp_buffer: Vec<usize>,    // temporary buffer for convergence check
    key_map_buffer: Vec<usize>, // used to map values to keys

    node_keys: Vec<Vec<usize>>,
    edge_keys: Vec<Vec<usize>>,

    dual: Vec<Vec<usize>>,
    hypergraph: StructuredHypergraph<E>,
}

impl<E> StructuralHypergraphSorter<E>
where
    E: Set,
{
    const MAX_ITER: usize = 128;

    /// Creates a new sorter for a hypergraph.
    ///
    /// Assumptions:
    /// - Initializes node_map and edge_map as identity permutations.
    /// - Builds initial node and edge keys based on sizes of incident edges/nodes.
    pub fn new(hypergraph: StructuredHypergraph<E>) -> Self {
        let buffsize = hypergraph.nodes.len().max(hypergraph.hyperedges.len());
        let dual = hypergraph.dual();
        Self {
            node_map: (0..hypergraph.nodes.len()).collect(),
            edge_map: (0..hypergraph.hyperedges.len()).collect(),

            temp_buffer: Vec::with_capacity(buffsize),
            key_map_buffer: Vec::with_capacity(buffsize),

            node_keys: Self::get_initial_keys(dual.iter().map(|n| n.len())),
            edge_keys: Self::get_initial_keys(hypergraph.hyperedges.iter().map(|n| n.len())),
            dual,
            hypergraph,
        }
    }

    fn get_initial_keys<T: Iterator<Item = usize>>(v: T) -> Vec<Vec<usize>> {
        v.map(|l| {
            let mut k = Vec::with_capacity(l);
            k.push(usize::MAX - l);
            k
        })
        .collect()
    }

    /// Sorts nodes and hyperedges into a canonical order.
    ///
    /// - Builds structural equivalence classes.
    /// - Refines partitions until stable.
    /// - Applies canonical permutations to nodes and edges.
    ///
    /// Assumptions:
    /// - Partitions will stabilize within MAX_ITER iterations.
    pub fn sort(mut self) -> StructuredHypergraph<E> {
        self.hypergraph.edge_structure_partitions = vec![0, self.hypergraph.hyperedges.len()];
        self.hypergraph.node_structure_partitions = vec![0, self.hypergraph.nodes.len()];

        self.sort_edges();
        self.sort_nodes();

        self.build_structural_eq_classes();
        self.sort_canonically();

        self.hypergraph.apply_edge_map(&self.edge_map);
        self.hypergraph.apply_node_map(&self.node_map);
        self.hypergraph
    }
    fn build_structural_eq_classes(&mut self) {
        loop {
            let old_partition_count = self.hypergraph.node_structure_partitions.len()
                + self.hypergraph.edge_structure_partitions.len();

            Self::fill_partition_map(
                &mut self.temp_buffer,
                &self.hypergraph.edge_structure_partitions,
            );
            Self::apply_inv_permutation(
                &self.temp_buffer,
                &mut self.key_map_buffer,
                &self.edge_map,
            );
            self.build_node_keys();
            self.sort_nodes();
            self.refine_nodes();

            Self::fill_partition_map(
                &mut self.temp_buffer,
                &self.hypergraph.node_structure_partitions,
            );
            Self::apply_inv_permutation(
                &self.temp_buffer,
                &mut self.key_map_buffer,
                &self.node_map,
            );
            self.build_edge_keys();
            self.sort_edges();
            self.refine_edges();

            let partition_count = self.hypergraph.node_structure_partitions.len()
                + self.hypergraph.edge_structure_partitions.len();
            if old_partition_count == partition_count {
                return;
            }
        }
    }
    fn sort_canonically(&mut self) {
        for _ in 0..Self::MAX_ITER {
            Self::fill_inv_permutation(&mut self.key_map_buffer, &self.edge_map);
            for k in self.key_map_buffer.iter_mut() {
                *k = self.edge_map.len() - 1 - *k;
            }
            self.temp_buffer.resize(self.node_map.len(), 0);
            self.temp_buffer.copy_from_slice(&self.node_map);
            self.build_node_keys();
            self.sort_nodes();
            let node_perm_unchanged = self.temp_buffer == self.node_map;

            Self::fill_inv_permutation(&mut self.key_map_buffer, &self.node_map);
            for k in self.key_map_buffer.iter_mut() {
                *k = self.node_map.len() - 1 - *k;
            }
            self.temp_buffer.resize(self.edge_map.len(), 0);
            self.temp_buffer.copy_from_slice(&self.edge_map);
            self.build_edge_keys();
            self.sort_edges();
            let edge_perm_unchanged = self.temp_buffer == self.edge_map;
            if edge_perm_unchanged && node_perm_unchanged {
                return;
            }
        }
    }

    fn build_edge_keys(&mut self) {
        for (i, e) in self.hypergraph.hyperedges.iter().enumerate() {
            self.edge_keys[i].clear();
            self.edge_keys[i].extend(e.iter().map(|n| self.key_map_buffer[n]));
            self.edge_keys[i].sort_unstable();
        }
    }
    fn build_node_keys(&mut self) {
        for (i, n) in self.dual.iter().enumerate() {
            self.node_keys[i].clear();
            self.node_keys[i].extend(n.iter().map(|e| self.key_map_buffer[*e]));
            self.node_keys[i].sort_unstable();
        }
    }

    fn sort_edges(&mut self) {
        Self::sort_partitions_by_key(
            &self.hypergraph.edge_structure_partitions,
            &mut self.edge_map,
            &self.edge_keys,
        );
    }
    fn refine_edges(&mut self) {
        Self::refine_partitions_by_key(
            &mut self.hypergraph.edge_structure_partitions,
            &self.edge_map,
            &self.edge_keys,
        );
    }
    fn sort_nodes(&mut self) {
        Self::sort_partitions_by_key(
            &self.hypergraph.node_structure_partitions,
            &mut self.node_map,
            &self.node_keys,
        );
    }
    fn refine_nodes(&mut self) {
        Self::refine_partitions_by_key(
            &mut self.hypergraph.node_structure_partitions,
            &self.node_map,
            &self.node_keys,
        );
    }

    fn refine_partitions_by_key<T: Ord>(
        partitions: &mut Vec<usize>,
        permutation: &[usize],
        keys: &[T],
    ) {
        partitions.clear();
        partitions.push(0);
        for i in 1..keys.len() {
            if keys[permutation[i - 1]] != keys[permutation[i]] {
                partitions.push(i);
            }
        }
        partitions.push(keys.len());
    }

    fn sort_partitions_by_key<T: Ord>(partitions: &[usize], permutation: &mut [usize], keys: &[T]) {
        for i in 0..partitions.len() - 1 {
            let part = &mut permutation[partitions[i]..partitions[i + 1]];

            part.sort_unstable_by(|a, b| keys[*a].cmp(&keys[*b]));
        }
    }
    /// Returns a partition map assigning each element to a partition index.
    fn fill_partition_map(buff: &mut Vec<usize>, partitions: &[usize]) {
        buff.resize(*partitions.last().unwrap(), 0);
        let mut p = 1;
        (0..buff.len()).for_each(|i| {
            if partitions[p] == i {
                p += 1;
            }
            buff[i] = p - 1;
        });
    }
    fn fill_inv_permutation(buff: &mut Vec<usize>, permutation: &[usize]) {
        buff.resize(permutation.len(), 0);
        for i in 0..permutation.len() {
            buff[permutation[i]] = i
        }
    }
    fn apply_inv_permutation(buff_in: &[usize], buff_out: &mut Vec<usize>, permutation: &[usize]) {
        buff_out.resize(buff_in.len(), 0);
        for i in 0..permutation.len() {
            buff_out[permutation[i]] = buff_in[i];
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::hypergraph::Bitset128;

    use super::*;

    #[test]
    fn test_from_hyperedges_basic() {
        let edges = vec![
            Bitset128::from_slice(&[0, 1]),
            Bitset128::from_slice(&[1, 2]),
        ];
        let graphs = StructuredHypergraph::from_hyperedges(edges);
        assert_eq!(graphs.len(), 1);
        let g = &graphs[0];
        assert_eq!(g.nr_nodes(), 3);
        assert_eq!(g.hyperedges().len(), 2);
    }

    #[test]
    fn test_remove_redundant_hyperedges() {
        let edges = vec![
            Bitset128::from_slice(&[0, 1]),
            Bitset128::from_slice(&[0, 1, 2]), // superset
            Bitset128::from_slice(&[2, 3]),
        ];
        let graphs = StructuredHypergraph::from_hyperedges(edges);
        assert_eq!(graphs.len(), 1); // two disconnected components
        for g in graphs {
            for e in g.hyperedges() {
                // no edge is subset of another inside a component
                for f in g.hyperedges() {
                    if e != f {
                        assert!(!e.is_subset(f));
                    }
                }
            }
        }
    }

    #[test]
    fn test_flatten_nodes() {
        let edges = vec![
            Bitset128::from_slice(&[2, 4]),
            Bitset128::from_slice(&[4, 6]),
        ];
        let mut g = StructuredHypergraph {
            hyperedges: edges.clone(),
            nodes: (0..=6).collect(),
            node_structure_partitions: vec![],
            edge_structure_partitions: vec![],
        };
        g.flatten_nodes();
        // node indices should now be 0,1,2
        for edge in &g.hyperedges {
            for node in edge.iter() {
                assert!(node < g.nr_nodes());
            }
        }
    }

    #[test]
    fn test_get_parts() {
        let edges = vec![
            Bitset128::from_slice(&[0, 1]),
            Bitset128::from_slice(&[1, 2]),
            Bitset128::from_slice(&[3, 4]),
        ];
        let g = StructuredHypergraph::from_hyperedges(edges);
        // Should split into 2 components
        assert_eq!(g.len(), 2);
        let sizes: Vec<usize> = g.iter().map(|h| h.nr_nodes()).collect();
        assert!(sizes.contains(&3)); // nodes 0,1,2
        assert!(sizes.contains(&2)); // nodes 3,4
    }
    #[test]
    fn test_canonization() {
        let edges = vec![
            Bitset128::from_slice(&[0, 1]),
            Bitset128::from_slice(&[1, 2]),
        ];
        let g1 = StructuredHypergraph::from_hyperedges(edges);
        let edges = vec![
            Bitset128::from_slice(&[0, 3]),
            Bitset128::from_slice(&[3, 2]),
        ];
        let g2 = StructuredHypergraph::from_hyperedges(edges);
        let edges = vec![
            Bitset128::from_slice(&[9, 4]),
            Bitset128::from_slice(&[4, 2]),
            Bitset128::from_slice(&[9]),
            Bitset128::from_slice(&[2, 4]),
        ];
        let g3 = StructuredHypergraph::from_hyperedges(edges);
        assert_eq!(g1, g2);
        assert_eq!(g1, g3);
        assert_eq!(g2, g3);
    }

    #[test]
    fn test_dual() {
        let edges = vec![
            Bitset128::from_slice(&[0, 1]),
            Bitset128::from_slice(&[1, 2]),
        ];
        let g = StructuredHypergraph::from_hyperedges(edges.clone());
        let dual = g[0].dual();
        // dual should map nodes to incident hyperedges
        assert_eq!(dual.len(), 3); // 3 nodes
        assert_eq!(dual[0], vec![1]);
        assert_eq!(dual[1], vec![0]);
        assert_eq!(dual[2], vec![0, 1]);
    }

    #[test]
    fn test_apply_node_map() {
        let edges = vec![
            Bitset128::from_slice(&[0, 2]),
            Bitset128::from_slice(&[1, 2]),
        ];
        let mut g = StructuredHypergraph::from_hyperedges(edges.clone())[0].clone();
        g.apply_node_map(&[2, 0, 1]); // permute nodes
        let all_nodes: Vec<usize> = g.hyperedges().iter().flat_map(|e| e.iter()).collect();
        assert_eq!(
            all_nodes.len(),
            g.hyperedges().iter().map(|e| e.len()).sum()
        );
        assert!(all_nodes.iter().all(|&n| n < g.nr_nodes()));
    }

    #[test]
    fn test_minus_splits_components() {
        let edges = vec![
            Bitset128::from_slice(&[0, 1]),
            Bitset128::from_slice(&[1, 2]),
        ];
        let g = StructuredHypergraph::from_hyperedges(edges.clone())[0].clone();

        // Remove node 1
        let comps = g.minus(Bitset128::from_slice(&[g
            .nodes
            .iter()
            .position(|&n| n == 1)
            .unwrap()]));

        // Should split into two components: {0} and {2}
        assert_eq!(comps.len(), 2);
        let node_sets: Vec<Vec<usize>> = comps.iter().map(|c| c.nodes.to_vec()).collect();

        assert!(node_sets.contains(&vec![0]));
        assert!(node_sets.contains(&vec![2]));
    }

    #[test]
    fn test_minus_remove_all_nodes() {
        let edges = vec![Bitset128::from_slice(&[0, 1, 2])];
        let g = StructuredHypergraph::from_hyperedges(edges.clone())[0].clone();

        let comps = g.minus(Bitset128::from_slice(&[0, 1, 2]));

        // Removing all nodes should yield no components
        assert!(comps.is_empty());
    }

    #[test]
    fn test_minus_noop() {
        let edges = vec![
            Bitset128::from_slice(&[0, 1]),
            Bitset128::from_slice(&[1, 2]),
        ];
        let g = StructuredHypergraph::from_hyperedges(edges.clone())[0].clone();

        let other = g.minus(Bitset128::default())[0].clone();
        assert_eq!(g, other);
    }
}
