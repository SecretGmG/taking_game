use sorted_vec::SortedSet;

use crate::util;

use super::TakingGame;
use std::collections::HashMap;

impl TakingGame {

    #[cfg(not(feature = "symmetry_finder"))]
    pub fn find_symmetry(&self) -> Option<Vec<usize>> {
        // disabled version: always return None
        None
    }
    #[cfg(feature = "symmetry_finder")]
    pub fn find_symmetry(&self) -> Option<Vec<usize>> {
        if self.node_count % 2 != 0 {
            return None;
        }

        let symmetry_hash = self.generate_symmety_hash()?;

        let sets_of_candidates = Self::generate_sets_of_candidates(symmetry_hash)?;

        let neighbourhoods = self.get_neighbourhoods();

        let mut symmetries = vec![None; self.node_count];
        self.generate_symmetry_from_sets_of_candidates(&mut symmetries, &sets_of_candidates, &neighbourhoods)
    }


    fn update_node_parities(&self, node_parities : &mut Vec<usize>, set_parities : & Vec<usize>){
        for ni in 0..self.node_count {
            let mut hash: usize = 0;
            for &si in &self.get_set_indices()[ni] {
                hash = hash.wrapping_mul(31) ^ set_parities[si];
            }
            node_parities[ni] = node_parities[ni].wrapping_mul(31) ^ hash;
        }
    }
    fn update_set_parities(&self, node_parities : &Vec<usize>, set_parities : &mut Vec<usize>){
        for sis in self.get_set_indices() {
            let mut hash: usize = 0;
            for &si in sis{
                hash = hash.wrapping_mul(31) ^ self.sets_of_nodes[si].iter().fold(0, |a, b| a ^ node_parities[*b]);
            }
            for &si in sis{
                set_parities[si] = set_parities[si].wrapping_mul(31) ^ hash;
            }
        }
    }

    fn generate_symmety_hash(&self) -> Option<Vec<usize>> {
        let mut node_parities = vec![0; self.node_count];
        let mut set_parities: Vec<usize> = self.sets_of_nodes.iter().map(|s| s.len()).collect();

        self.update_node_parities(&mut node_parities, &set_parities);
        for _ in 0..3{
            self.update_set_parities(&node_parities, &mut set_parities);
            self.update_node_parities(&mut node_parities, &set_parities);
            if node_parities.iter().fold(0, |a, b| a ^ *b) != 0{
                return None
            }

        }
        Some(node_parities)
    }

    fn generate_sets_of_candidates(symmetry_hash : Vec<usize>) -> Option<Vec<SortedSet<usize>>>{
        let mut sets_of_candidates: HashMap<usize, SortedSet<usize>> = HashMap::with_capacity(symmetry_hash.len() / 2);

        for (i, hash) in symmetry_hash.iter().enumerate(){
            match sets_of_candidates.get_mut(hash){
                Some(set) => { set.push(i); },
                None => { sets_of_candidates.insert(*hash,  SortedSet::from_unsorted(vec![i])); },
            }
        }
        if sets_of_candidates.values().any(|set| set.len() % 2 != 0){
            return None
        }
        Some(sets_of_candidates.into_values().collect())
    }

    fn generate_symmetry_from_sets_of_candidates(
        &self,
        symmetries: &mut Vec<Option<usize>>,
        sets_of_candidates: &Vec<SortedSet<usize>>,
        neighbourhoods : &Vec<SortedSet<usize>>
    ) -> Option<Vec<usize>> {
        if let Some(node) = Self::find_unmatched_node(symmetries) {
            let candidates = self.find_valid_candidates(node, symmetries, sets_of_candidates, neighbourhoods);
            for cand in candidates {
                symmetries[node] = Some(cand);
                symmetries[cand] = Some(node);

                if let Some(result) = self.generate_symmetry_from_sets_of_candidates(symmetries, sets_of_candidates, neighbourhoods) {
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

    fn find_unmatched_node(symmetries: &[Option<usize>]) -> Option<usize> {
        symmetries.iter().position(|v| v.is_none())
    }

    fn find_valid_candidates(
        &self,
        node: usize,
        symmetries: &[Option<usize>],
        candidate_groups: &Vec<SortedSet<usize>>,
        neighbourhoods : &Vec<SortedSet<usize>>
    ) -> Vec<usize> {
        for group in candidate_groups {
            if group.contains(&node) {
                return group
                    .iter()
                    .copied()
                    .filter(|&cand| self.is_valid_match(node, cand, symmetries, neighbourhoods))
                    .collect();
            }
        }
        unreachable!();
    }

    fn is_valid_match(
        &self,
        node: usize,
        candidate: usize,
        symmetries: &[Option<usize>],
        neighbourhoods : &Vec<SortedSet<usize>>
    ) -> bool {
        if node == candidate || symmetries[candidate].is_some() {
            return false;
        }

        // Check if they share a set -> immediate disqualification
        if self.get_set_indices()[node]
            .iter()
            .any(|&si| self.sets_of_nodes[si].contains(&candidate))
        {
            return false;
        }

        // All neighbors of node (that are already mapped) must have their images in candidate's neighborhood
        let candidate_neighbours = &neighbourhoods[candidate];

        for &set_index in &self.get_set_indices()[node] {
            for &neighbour in &self.sets_of_nodes[set_index] {
                if let Some(mapped) = symmetries[neighbour] {
                    if !candidate_neighbours.contains(&mapped) {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn get_neighbourhoods(&self) -> Vec<SortedSet<usize>> {
        let mut neighbourhoods: Vec<SortedSet<usize>> = vec![SortedSet::new(); self.get_node_count()];
        for node in 0..self.get_node_count(){
            for &si in &self.get_set_indices()[node] {
                neighbourhoods[node] = util::merge(&neighbourhoods[node], &self.get_sets_of_nodes()[si])
            }
        }
        neighbourhoods
    }
    //fn get_neighbourhood(&self, node: usize) -> SortedSet<usize> {
    //    let mut neighbours = SortedSet::new();
    //    for &si in &self.get_set_indices()[node] {
    //        neighbours = util::merge(&neighbours, &self.sets_of_nodes[si])
    //    }
    //    neighbours
    //}
}


#[cfg(test)]
mod tests {

    use crate::Constructor;

    #[test]
    fn test_hypercube_2_2() {
        let g = Constructor::hyper_cube(2, 2).build();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_4_4() {
        let g = Constructor::hyper_cube(4, 4).build();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_2_32() {
        let g = Constructor::hyper_cube(2, 32).build();
        assert!(g.find_symmetry().is_some());
    }
    #[test]
    fn test_hypercube_3_3() {
        let g = Constructor::hyper_cube(3, 3).build();
        assert!(g.find_symmetry().is_none());
    }
    #[test]
    fn test_hypertetrahedron_16() {
        let g = Constructor::hyper_tetrahedron(15).build();
        assert!(g.find_symmetry().is_none());
    }
}
