use sorted_vec::SortedSet;
use std::cmp::Ordering;

use crate::{Constructor, TakingGame};

pub fn compare_sorted<T: Ord>(vec1: &[T], vec2: &[T]) -> Ordering {
    match vec1.len().cmp(&vec2.len()) {
        Ordering::Less => return Ordering::Less,
        Ordering::Greater => return Ordering::Greater,
        Ordering::Equal => (),
    }
    for i in 0..vec1.len() {
        match vec1[i].cmp(&vec2[i]) {
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
            Ordering::Equal => (),
        }
    }
    Ordering::Equal
}
///retures true if a and b share any elements
pub fn have_common_element(a: &SortedSet<usize>, b: &SortedSet<usize>) -> bool {
    let mut i = 0;
    let mut j = 0;

    while i < a.len() && j < b.len() {
        if a[i] < b[j] {
            i += 1;
        } else if a[i] > b[j] {
            j += 1;
        } else {
            return true;
        }
    }
    false
}
///calculates the inverse permutation of a given input permutation
///undefined behaviour if the input is not a permutation
pub fn inverse_permutation(refrences: Vec<usize>) -> Vec<usize> {
    let mut perm = vec![0; refrences.len()];
    for i in 0..refrences.len() {
        perm[refrences[i]] = i;
    }
    perm
}

///checks if a is a subset of b, assums both vectors are sorted
pub fn is_subset(a: &[usize], b: &[usize]) -> bool {
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

pub fn sort_together_by_key<U, V, F, K>(vec: &mut Vec<U>, other: &mut Vec<V>, mut key: F)
where
    F: FnMut(&U) -> K,
    K: Ord,
{
    if vec.is_sorted_by_key(&mut key) {
        return;
    }

    let mut pairs: Vec<(U, V)> = vec.drain(..).zip(other.drain(..)).collect();

    pairs.sort_by_key(|(a, _)| key(a));
    // Unzip back
    let (sorted_vec, sorted_other): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
    *vec = sorted_vec;
    *other = sorted_other;
}

pub fn union_append(buff: &mut Vec<usize>, other: &[usize]) {
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
        (Constructor::rect(1, 3).build(), Some(3), Some(false)),
        (Constructor::rect(4, 1).build(), Some(4), Some(false)),
        (Constructor::rect(100, 1).build(), Some(100), Some(false)),
        (Constructor::rect(1, 101).build(), Some(101), Some(false)),
        (Constructor::rect(2, 2).build(), Some(0), Some(true)),
        (Constructor::rect(3, 3).build(), Some(0), Some(false)),
        (Constructor::rect(3, 4).build(), None, Some(false)),
        (Constructor::rect(4, 4).build(), Some(0), Some(true)),
        (Constructor::rect(5, 4).build(), None, Some(false)),
        // (
        //     Constructor::rect(3, 6)
        //         .combine(Constructor::rect(6, 3).build())
        //         .build(),
        //     Some(0),
        //     Some(true),
        // ),
        // (
        //     Constructor::rect(1, 50)
        //         .combine(Constructor::rect(2, 9).build())
        //         .build(),
        //     None,
        //     Some(false),
        // ),
        // (
        //     Constructor::rect(1, 10)
        //         .combine(Constructor::rect(2, 5).build())
        //         .connect_unit_to_all()
        //         .build(),
        //     None,
        //     Some(false),
        // ),
        // (
        //     Constructor::rect(1, 50)
        //         .combine(Constructor::rect(2, 9).build())
        //         .combine(Constructor::triangle(3).build())
        //         .build(),
        //     None,
        //     Some(false),
        // ),
        // (
        //     Constructor::rect(2, 11)
        //         .combine(Constructor::rect(2, 11).build())
        //         .combine(Constructor::rect(2, 10).build())
        //         .build(),
        //     Some(0),
        //     Some(true),
        // ),
        (Constructor::hyper_cube(3, 2).build(), Some(0), Some(true)),
    ]
}
