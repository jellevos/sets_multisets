use bytevec::ByteEncodable;
use fasthash::xx;
use rand::rngs::OsRng;
use rand::seq::index::sample;
use std::collections::{HashMap, HashSet};

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
            elements: sample(&mut OsRng, universe, element_count).into_iter().collect(),
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
            elements: self.elements.intersection(&other.elements).copied().collect(),
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

    pub fn to_bloom_filter(&self, bin_count: usize, hash_count: usize) -> Vec<bool> {
        let mut bins = vec![false; bin_count];

        for element in &self.elements {
            let element_bytes = (*element as u64).encode::<u64>().unwrap();

            for seed in 0..hash_count {
                bins[xx::hash32_with_seed(&element_bytes, seed as u32) as usize % bin_count] = true;
            }
        }

        bins
    }
}

pub struct Multiset {
    pub element_counts: HashMap<usize, usize>,
}

pub fn bloom_filter_contains(element: &usize, bins: &[bool], hash_count: usize) -> bool {
    let bin_count = bins.len();

    let element_bytes = (*element as u64).encode::<u64>().unwrap();

    for seed in 0..hash_count {
        if !bins[xx::hash32_with_seed(&element_bytes, seed as u32) as usize % bin_count] {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use crate::{bloom_filter_contains, Set};

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
    fn test_to_bloom_filter() {
        let set = Set::new(&vec![1, 3, 4]);
        let bloom_filter = set.to_bloom_filter(20, 2);

        assert!(bloom_filter_contains(&1, &bloom_filter, 2));
        assert!(!bloom_filter_contains(&2, &bloom_filter, 2));
        assert!(bloom_filter_contains(&3, &bloom_filter, 2));
        assert!(bloom_filter_contains(&4, &bloom_filter, 2));
        assert!(!bloom_filter_contains(&5, &bloom_filter, 2));
    }
}
