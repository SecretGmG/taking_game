use std::{hash::Hash, ops::Range};

pub trait Set: Default + Sized {
    type Iter<'a>: Iterator<Item = usize> + 'a
    where
        Self: 'a;

    fn from_slice(vec: &[usize]) -> Self;
    fn insert(&mut self, value: usize);

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn iter(&self) -> Self::Iter<'_>;

    fn contains(&self, element: &usize) -> bool;
    fn union(&mut self, other: &Self);
    fn minus(&self, other: &Self) -> Self;
    fn is_subset(&self, other: &Self) -> bool;
    fn intersects(&self, other: &Self) -> bool;

    fn apply_node_map(&mut self, permutation: &[usize]);
    fn is_flattened(&self) -> bool;
    fn partition(&self, partitions: &[Range<usize>]) -> Vec<Self>;
    fn pop(&mut self) -> Option<usize>;
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub struct Bitset128(u128);
impl Bitset128 {
    pub fn new(bits: u128) -> Self {
        Bitset128(bits)
    }
}

pub struct Bitset128Iter {
    bits: u128,
}

impl Iterator for Bitset128Iter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }
        let tz = self.bits.trailing_zeros() as usize;
        self.bits &= !(1 << tz);
        Some(tz)
    }
}

impl Set for Bitset128 {
    type Iter<'a>
        = Bitset128Iter
    where
        Self: 'a;

    fn from_slice(vec: &[usize]) -> Self {
        let mut set = Bitset128::default();
        vec.iter().copied().for_each(|e| set.insert(e));
        set
    }

    fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    fn is_empty(&self) -> bool {
        self.0 == 0
    }

    fn iter(&self) -> Self::Iter<'_> {
        Bitset128Iter { bits: self.0 }
    }

    fn union(&mut self, other: &Self) {
        self.0 |= other.0;
    }

    fn is_subset(&self, other: &Self) -> bool {
        (self.0 & !other.0) == 0
    }

    fn intersects(&self, other: &Self) -> bool {
        (self.0 & other.0) != 0
    }

    fn apply_node_map(&mut self, permutation: &[usize]) {
        let mut new_bits = 0u128;
        for (new_idx, old_idx) in permutation.iter().enumerate() {
            if (self.0 >> old_idx) & 1 != 0 {
                new_bits |= 1 << new_idx;
            }
        }
        self.0 = new_bits;
    }

    fn is_flattened(&self) -> bool {
        self.0 & (self.0.wrapping_add(1)) == 0
    }

    fn partition(&self, partitions: &[Range<usize>]) -> Vec<Self> {
        let mut p = Vec::with_capacity(partitions.len());
        for part in partitions {
            let mask = if part.len() == 128 {
                u128::MAX
            } else {
                ((1u128 << part.len()) - 1) << part.start
            };
            p.push(Bitset128(self.0 & mask));
        }
        p
    }

    fn pop(&mut self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        let val = 127 - self.0.leading_zeros() as usize;
        self.0 &= !(1 << val);
        Some(val)
    }

    fn insert(&mut self, value: usize) {
        self.0 |= 1 << value;
    }

    fn minus(&self, other: &Self) -> Self {
        Self(self.0 & !other.0)
    }

    fn contains(&self, element: &usize) -> bool {
        (self.0 >> element) & 1 == 1
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len_and_is_empty() {
        let mut b = Bitset128(0);
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);

        b.0 = 0b1011;
        assert!(!b.is_empty());
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn test_iter() {
        let b = Bitset128(0b10110);
        let collected: Vec<usize> = b.iter().collect();
        assert_eq!(collected, vec![1, 2, 4]);
    }

    #[test]
    fn test_union() {
        let mut a = Bitset128(0b1010);
        let b = Bitset128(0b0110);
        a.union(&b);
        assert_eq!(a.0, 0b1110);
    }

    #[test]
    fn test_is_subset_and_intersects() {
        let a = Bitset128(0b1010);
        let b = Bitset128(0b1110);
        assert!(a.is_subset(&b));
        assert!(!b.is_subset(&a));
        assert!(a.intersects(&b));

        let c = Bitset128(0b0001);
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_apply_node_map() {
        let mut b = Bitset128(0b1011); // bits 0,1,3
        b.apply_node_map(&[3, 2, 1, 0]); // reverse order
        assert_eq!(b.0, 0b1101); // bits 0,2,3
    }

    #[test]
    fn test_is_flattened() {
        let a = Bitset128(0b111); // 0,1,2
        let b = Bitset128(0b101); // 0,2
        assert!(a.is_flattened());
        assert!(!b.is_flattened());
    }

    #[test]
    fn test_partition() {
        let b = Bitset128(0b1111_1010); // bits 1,3,4,5,7
        let partitions = [0..4, 4..8];
        let parts = b.partition(&partitions);
        assert_eq!(parts[0].0, 0b1010); // bits 1,3
        assert_eq!(parts[1].0, 0b11110000 & b.0); // bits 4,5,7
    }

    #[test]
    fn test_pop() {
        let mut b = Bitset128(0b10110);
        assert_eq!(b.pop(), Some(4));
        assert_eq!(b.pop(), Some(2));
        assert_eq!(b.pop(), Some(1));
        assert_eq!(b.pop(), None);
        assert!(b.is_empty());
    }
    #[test]
    fn test_contains() {
        let b = Bitset128::from_slice(&[1, 2, 3, 5, 8, 13, 21, 34]);
        assert!(b.contains(&1));
        assert!(b.contains(&34));
        assert!(!b.contains(&17));
    }
}
