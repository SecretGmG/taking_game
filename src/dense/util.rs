use crate::{Constructor, TakingGame};

///checks if a is a subset of b, assums both vectors are sorted
pub(crate) fn is_subset(a: &[usize], b: &[usize]) -> bool {
    let mut i = 0;
    let mut j = 0;

    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => return false, // element in vec1 not found in vec2
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Greater => j += 1, // skip element in vec2
        }
    }

    i == a.len()
}
pub(crate) fn refine_partitions_by_key<T: Ord>(
    partitions: &mut Vec<usize>,
    permutation: &[usize],
    keys: &[T],
) {
    for i in 1..keys.len() {
        if keys[permutation[i - 1]] != keys[permutation[i]] {
            if let Err(partition_index) = partitions.binary_search(&i) {
                partitions.insert(partition_index, i);
            }
        }
    }
}

pub(crate) fn sort_partitions_by_key<T: Ord>(
    partitions: &[usize],
    permutation: &mut [usize],
    keys: &[T],
) {
    for i in 0..partitions.len() - 1 {
        let part = &mut permutation[partitions[i]..partitions[i + 1]];
        part.sort_by_key(|e| &keys[*e]);
    }
}
pub(crate) fn sort_refine_partitions_by_key<T: Ord>(
    partitions: &mut Vec<usize>,
    permutation: &mut [usize],
    keys: &[T],
) {
    sort_partitions_by_key(partitions, permutation, keys);
    refine_partitions_by_key(partitions, permutation, keys);
}

/// Returns a partition map assigning each element to a partition index.
/// Partition indices are 1-based: elements in the first block map to 1,
/// next block to 2, etc.
pub(crate) fn fill_partition_map(buff: &mut [usize], partitions: &[usize]) {
    let mut p = 1;
    (0..buff.len()).for_each(|i| {
        if partitions[p] == i {
            p += 1;
        }
        buff[i] = p;
    });
}
pub(crate) fn fill_inverse_permutation(buff: &mut [usize], permutation: &[usize]) {
    for i in 0..permutation.len() {
        buff[permutation[i]] = i
    }
}

pub(crate) fn union_append(buff: &mut Vec<usize>, other: &[usize]) {
    let mut iter1 = buff.clone().into_iter();
    let mut iter2 = other.iter().copied();

    let mut a = iter1.next();
    let mut b = iter2.next();

    while let (Some(x), Some(y)) = (a, b) {
        match x.cmp(&y) {
            std::cmp::Ordering::Less => {
                buff.push(x);
                a = iter1.next();
            }
            std::cmp::Ordering::Greater => {
                buff.push(y);
                b = iter2.next();
            }
            std::cmp::Ordering::Equal => {
                buff.push(x);
                a = iter1.next();
                b = iter2.next();
            }
        }
    }

    buff.extend(a);
    buff.extend(iter1);
    buff.extend(b);
    buff.extend(iter2);
}

pub fn get_test_games() -> Vec<(TakingGame, Option<usize>, Option<bool>)> {
    vec![
        (Constructor::rect(1, 3).build_one(), Some(3), Some(false)),
        (Constructor::rect(4, 1).build_one(), Some(4), Some(false)),
        (
            Constructor::rect(100, 1).build_one(),
            Some(100),
            Some(false),
        ),
        (
            Constructor::rect(1, 101).build_one(),
            Some(101),
            Some(false),
        ),
        (Constructor::rect(2, 2).build_one(), Some(0), Some(true)),
        (Constructor::rect(3, 3).build_one(), Some(0), Some(false)),
        (Constructor::rect(3, 4).build_one(), None, Some(false)),
        (Constructor::rect(4, 4).build_one(), Some(0), Some(true)),
        (Constructor::rect(5, 4).build_one(), None, Some(false)),
        (
            Constructor::hyper_cube(3, 2).build_one(),
            Some(0),
            Some(true),
        ),
    ]
}
