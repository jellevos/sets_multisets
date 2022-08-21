use bytevec::ByteEncodable;
use rand::rngs::OsRng;
use rand::seq::index::sample;
use std::collections::HashSet;
use std::iter::FromIterator;
use xxh3::hash64_with_seed;

#[derive(Eq, PartialEq, Debug)]
pub struct Set {
    pub elements: HashSet<usize>,
}

impl Set {
    pub fn new(elements: &[usize]) -> Self {
        Set {
            elements: elements.iter().copied().collect(),
        }
    }

    pub fn random(element_count: usize, universe: usize) -> Self {
        Set {
            elements: sample(&mut OsRng, universe, element_count)
                .into_iter()
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn intersect(&self, other: &Set) -> Set {
        Set {
            elements: self
                .elements
                .intersection(&other.elements)
                .copied()
                .collect(),
        }
    }

    pub fn intersection(sets: &[Set]) -> Set {
        let mut result = sets[0].intersect(&sets[1]);

        for set in &sets[2..] {
            result = result.intersect(set);
        }

        result
    }

    pub fn to_bitset(&self, universe: usize) -> Vec<bool> {
        let mut bitset = vec![false; universe];

        for element in &self.elements {
            bitset[*element] = true;
        }

        bitset
    }

    pub fn from_bitset(bitset: &[bool]) -> Set {
        Set {
            elements: bitset
                .iter()
                .enumerate()
                .filter(|(_, b)| **b)
                .map(|(i, _)| i)
                .collect(),
        }
    }

    pub fn to_bloom_filter(&self, bin_count: usize, hash_count: usize) -> Vec<bool> {
        let mut bins = vec![false; bin_count];

        for element in &self.elements {
            let element_bytes = (*element as u64).encode::<u64>().unwrap();

            for seed in 0..hash_count {
                bins[hash64_with_seed(&element_bytes, seed as u64) as usize % bin_count] = true;
            }
        }

        bins
    }
}

pub fn bloom_filter_indices(
    element: &usize,
    bin_count: usize,
    hash_count: usize,
) -> impl Iterator<Item = usize> {
    let element_bytes = (*element as u64).encode::<u64>().unwrap();

    (0..hash_count)
        .map(move |seed| (hash64_with_seed(&element_bytes, seed as u64) as usize % bin_count))
}

pub fn bloom_filter_contains(bins: &[bool], element: &usize, hash_count: usize) -> bool {
    let bin_count = bins.len();

    for index in bloom_filter_indices(element, bin_count, hash_count) {
        if !bins[index] {
            return false;
        }
    }

    true
}

impl FromIterator<usize> for Set {
    fn from_iter<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        Set {
            elements: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sets::{bloom_filter_contains, Set};

    #[test]
    fn test_random() {
        let set1 = Set::random(5, 100);
        let set2 = Set::random(5, 100);

        assert_eq!(set1.len(), 5);
        assert_eq!(set2.len(), 5);

        assert_ne!(set1, set2);
    }

    #[test]
    fn test_intersect() {
        let set1 = Set::new(&vec![1, 3, 4]);
        let set2 = Set::new(&vec![1, 2, 4, 5]);

        let expected = Set::new(&vec![1, 4]);

        assert_eq!(set1.intersect(&set2), expected);
    }

    #[test]
    fn test_intersection() {
        let set1 = Set::new(&vec![1, 3, 4]);
        let set2 = Set::new(&vec![1, 2, 4, 5]);
        let set3 = Set::new(&vec![4, 3, 2]);

        let expected = Set::new(&vec![4]);

        assert_eq!(Set::intersection(&vec![set1, set2, set3]), expected);
    }

    #[test]
    fn test_to_bitset() {
        let set = Set::new(&vec![1, 3, 4]);
        assert_eq!(set.to_bitset(5), vec![false, true, false, true, true]);
    }

    #[test]
    fn test_from_bitset() {
        let bitset = vec![false, false, true, true, false, true];
        assert_eq!(Set::from_bitset(&bitset), Set::new(&vec![2, 3, 5]));
    }

    #[test]
    fn test_to_bloom_filter() {
        let set = Set::new(&vec![1, 3, 4]);
        let bloom_filter = set.to_bloom_filter(20, 2);

        assert!(bloom_filter_contains(&bloom_filter, &1, 2));
        assert!(!bloom_filter_contains(&bloom_filter, &2, 2));
        assert!(bloom_filter_contains(&bloom_filter, &3, 2));
        assert!(bloom_filter_contains(&bloom_filter, &4, 2));
        assert!(!bloom_filter_contains(&bloom_filter, &5, 2));
    }

    #[test]
    fn test_set_from_iter() {
        let elements = vec![1usize, 3, 4];
        let set: Set = elements.iter().map(|e| *e).collect();
        assert_eq!(Set::new(&elements), set);
    }
}
