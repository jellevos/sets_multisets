use crate::sets::Set;
use bytevec::ByteEncodable;
use xxh3::hash64_with_seed;

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

impl Set {
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

#[cfg(test)]
mod tests {
    use crate::{bloom_filters::bloom_filter_contains, sets::Set};

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
}
