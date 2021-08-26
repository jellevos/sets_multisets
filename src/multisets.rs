use crate::sets::bloom_filter_contains;
use bytevec::ByteEncodable;
use fasthash::xx;
use std::collections::HashMap;
use std::iter::FromIterator;

impl Multiset {
    pub fn new(elements: &[usize], counts: &[usize]) -> Self {
        assert_eq!(elements.len(), counts.len());

        Multiset {
            element_counts: elements
                .iter()
                .copied()
                .zip(counts.iter().copied())
                .collect(),
        }
    }

    pub fn to_bitset(&self, universe: usize, max_multiplicity: usize) -> Vec<bool> {
        let mut bitset = vec![false; universe * max_multiplicity];

        for (element, count) in &self.element_counts {
            for i in 0..*count {
                bitset[*element * max_multiplicity + i] = true;
            }
        }

        bitset
    }

    pub fn to_bloom_filter(
        &self,
        bin_count: usize,
        hash_count: usize,
        max_multiplicity: usize,
    ) -> Vec<bool> {
        let mut bins = vec![false; bin_count];

        for (element, count) in &self.element_counts {
            for i in 0..*count {
                let element_bytes = ((*element * max_multiplicity + i) as u64)
                    .encode::<u64>()
                    .unwrap();

                for seed in 0..hash_count {
                    bins[xx::hash32_with_seed(&element_bytes, seed as u32) as usize % bin_count] =
                        true;
                }
            }
        }

        bins
    }
}

pub fn bloom_filter_retrieve_count(
    bins: &[bool],
    element: &usize,
    hash_count: usize,
    max_multiplicity: usize,
) -> usize {
    for i in 0..max_multiplicity {
        if !bloom_filter_contains(bins, &(element * max_multiplicity + i), hash_count) {
            return i;
        }
    }

    max_multiplicity
}

#[derive(Eq, PartialEq, Debug)]
pub struct Multiset {
    pub element_counts: HashMap<usize, usize>,
}

impl FromIterator<(usize, usize)> for Multiset {
    fn from_iter<T: IntoIterator<Item = (usize, usize)>>(iter: T) -> Self {
        Multiset {
            element_counts: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::multisets::{bloom_filter_retrieve_count, Multiset};

    #[test]
    fn test_multiset_from_iter() {
        let elements = vec![1usize, 3, 4];
        let counts = vec![2usize, 2usize, 5usize];

        let multiset_a = Multiset::new(&elements, &counts);
        let multiset_b: Multiset = elements.into_iter().zip(counts).collect();

        assert_eq!(multiset_a, multiset_b);
    }

    #[test]
    fn test_to_bitset() {
        let multiset = Multiset::new(&vec![1, 3, 4], &vec![1, 2, 1]);
        assert_eq!(
            multiset.to_bitset(5, 2),
            vec![false, false, true, false, false, false, true, true, true, false]
        );
    }

    #[test]
    fn test_to_bloom_filter() {
        let multiset = Multiset::new(&vec![1, 3, 4], &vec![1, 2, 1]);
        let bloom_filter = multiset.to_bloom_filter(50, 2, 2);

        assert_eq!(bloom_filter_retrieve_count(&bloom_filter, &0, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count(&bloom_filter, &1, 2, 2), 1);
        assert_eq!(bloom_filter_retrieve_count(&bloom_filter, &2, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count(&bloom_filter, &3, 2, 2), 2);
        assert_eq!(bloom_filter_retrieve_count(&bloom_filter, &4, 2, 2), 1);
    }
}
