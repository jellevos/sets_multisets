use crate::{multisets::Multiset, sets::Set};
use bytevec::ByteEncodable;

// TODO: Consider supporting all hash functions at the same time
#[cfg(all(feature = "xxh3", feature = "shake128"))]
compile_error!("Please choose one hash function to use: xxh3 or shake128.");
#[cfg(all(feature = "xxh3", feature = "blake3"))]
compile_error!("Please choose one hash function to use: xxh3 or blake3.");
#[cfg(all(feature = "shake128", feature = "blake3"))]
compile_error!("Please choose one hash function to use: shake128 or blake3.");
#[cfg(all(feature = "xxh3", feature = "shake128", feature = "blake3"))]
compile_error!("Please choose one hash function to use: xxh3, shake128 or blake3.");

#[cfg(feature = "xxh3")]
use xxh3::hash64_with_seed;
#[cfg(feature = "xxh3")]
pub fn hash_element(element: &usize, seed: u64) -> usize {
    let element_bytes = (*element as u64).encode::<u64>().unwrap();
    hash64_with_seed(&element_bytes, seed) as usize
}

#[cfg(feature = "shake128")]
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake128,
};
#[cfg(feature = "shake128")]
pub fn hash_element(element: &usize, seed: u64) -> usize {
    let element_bytes = (*element as u64).encode::<u64>().unwrap();
    let seed_bytes = seed.encode::<u64>().unwrap();

    let mut hasher = Shake128::default();
    hasher.update(&element_bytes);
    hasher.update(&seed_bytes);
    let mut reader = hasher.finalize_xof();
    let mut res = [0u8; 8];
    reader.read(&mut res);

    usize::from_ne_bytes(res)
}

#[cfg(feature = "blake3")]
pub fn hash_element(element: &usize, seed: u64) -> usize {
    let element_bytes = (*element as u64).encode::<u64>().unwrap();
    let seed_bytes = seed.encode::<u64>().unwrap();

    let mut hasher = blake3::Hasher::new();
    hasher.update(&element_bytes);
    hasher.update(&seed_bytes);
    let mut reader = hasher.finalize_xof();
    let mut res = [0u8; 8];
    reader.fill(&mut res);

    usize::from_ne_bytes(res)
}

/// For a maximum error rate and maximum set size, returns a suitable minimum bin count and hash count. These parameters lead to the lowest possible bin_count that assures this maximum error rate.
pub fn gen_bloom_filter_params(max_error_rate: f64, max_set_size: usize) -> (usize, usize) {
    let mut h = 1;
    let mut previous = None;
    loop {
        let current = (-(h as f64) * (max_set_size as f64 + 0.5)
            / (1f64.ln() - max_error_rate.powf(1. / (h as f64)))
            + 1.)
            .ceil() as usize;

        if let Some(p) = previous {
            if p < current {
                return (p, h - 1);
            }
        }

        h += 1;
        previous = Some(current);
    }
}

pub fn bloom_filter_indices(
    element: &usize,
    bin_count: usize,
    hash_count: usize,
) -> impl Iterator<Item = usize> + '_ {
    (0..hash_count).map(move |seed| (hash_element(element, seed as u64) % bin_count))
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
            for seed in 0..hash_count {
                bins[hash_element(element, seed as u64) % bin_count] = true;
            }
        }

        bins
    }
}

impl Multiset {
    pub fn to_bloom_filter(
        &self,
        bin_count: usize,
        hash_count: usize,
        max_multiplicity: usize,
    ) -> Vec<bool> {
        let mut bins = vec![false; bin_count];

        for (element, count) in &self.element_counts {
            for i in 0..*count {
                for seed in 0..hash_count {
                    bins[hash_element(&(*element * max_multiplicity + i), seed as u64)
                        % bin_count] = true;
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

#[cfg(test)]
mod tests {
    use crate::{
        bloom_filters::{bloom_filter_contains, bloom_filter_retrieve_count},
        multisets::Multiset,
        sets::Set,
    };

    #[test]
    fn test_set_to_bloom_filter() {
        let set = Set::new(&vec![1, 3, 4]);
        let bloom_filter = set.to_bloom_filter(20, 2);

        assert!(bloom_filter_contains(&bloom_filter, &1, 2));
        assert!(!bloom_filter_contains(&bloom_filter, &2, 2));
        assert!(bloom_filter_contains(&bloom_filter, &3, 2));
        assert!(bloom_filter_contains(&bloom_filter, &4, 2));
        assert!(!bloom_filter_contains(&bloom_filter, &5, 2));
    }

    #[test]
    fn test_multiset_to_bloom_filter() {
        let multiset = Multiset::new(&vec![1, 3, 4], &vec![1, 2, 1]);
        let bloom_filter = multiset.to_bloom_filter(50, 2, 2);

        assert_eq!(bloom_filter_retrieve_count(&bloom_filter, &0, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count(&bloom_filter, &1, 2, 2), 1);
        assert_eq!(bloom_filter_retrieve_count(&bloom_filter, &2, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count(&bloom_filter, &3, 2, 2), 2);
        assert_eq!(bloom_filter_retrieve_count(&bloom_filter, &4, 2, 2), 1);
    }
}
