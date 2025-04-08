use crate::{multisets::Multiset, sets::Set};
use bytevec::ByteEncodable;

use xxh3::hash64_with_seed;

use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake128,
};

use argon2::Argon2;

pub trait ElementHasher {
    fn hash_element(element: &usize, seed: u64) -> usize;
    fn hash_element_multiple_seeds(element: &usize, seeds: &[u64]) -> Vec<usize> {
        seeds
            .iter()
            .map(|seed| Self::hash_element(element, *seed))
            .collect()
    }
}

pub struct Xxh3Hasher;

pub struct Shake128Hasher;

pub struct Blake3Hasher;

pub struct Argon2Hasher;

impl ElementHasher for Xxh3Hasher {
    fn hash_element(element: &usize, seed: u64) -> usize {
        let element_bytes = (*element as u64).encode::<u64>().unwrap();
        hash64_with_seed(&element_bytes, seed) as usize
    }
}

impl ElementHasher for Shake128Hasher {
    fn hash_element(element: &usize, seed: u64) -> usize {
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
}

impl ElementHasher for Blake3Hasher {
    fn hash_element(element: &usize, seed: u64) -> usize {
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
}

impl ElementHasher for Argon2Hasher {
    fn hash_element(element: &usize, seed: u64) -> usize {
        let element_bytes = (*element as u64).encode::<u64>().unwrap();

        let mut res = [0u8; 32];
        Argon2::default()
            .hash_password_into(&element_bytes, b"bloom_filter", &mut res)
            .unwrap();

        hash64_with_seed(&res, seed) as usize
    }

    fn hash_element_multiple_seeds(element: &usize, seeds: &[u64]) -> Vec<usize> {
        let element_bytes = (*element as u64).encode::<u64>().unwrap();

        let mut res = [0u8; 32];
        Argon2::default()
            .hash_password_into(&element_bytes, b"bloom_filter", &mut res)
            .unwrap();

        seeds
            .iter()
            .map(|seed| hash64_with_seed(&res, *seed) as usize)
            .collect()
    }
}

/// For a maximum error rate and maximum set size, returns a suitable minimum bin count and hash count. These parameters lead to the lowest possible bin_count that assures this maximum error rate.
pub fn gen_bloom_filter_params(max_error_rate: f64, max_set_size: usize) -> (usize, usize) {
    let mut h = 1;
    let mut previous = None;
    loop {
        let current = (-(h as f64) * (max_set_size as f64 + 0.5)
            / (1f64 - max_error_rate.powf(1. / (h as f64))).ln()
            + 1.)
            .ceil() as usize;

        if let Some(p) = previous {
            if p < current {
                return (p, h - 1);
            }
        }

        h += 1;
        if current != 0 {
            previous = Some(current);
        }
    }
}

/// Same as gen_bloom_filter_params except that the max_error_rate is now log2
pub fn gen_bloom_filter_params_log2(
    max_error_rate_log2: f64,
    max_set_size: usize,
) -> (usize, usize) {
    let mut h = 1;
    let mut previous = None;
    loop {
        let current = (-(h as f64) * (max_set_size as f64 + 0.5)
            / (1f64 - 2f64.powf(max_error_rate_log2 / (h as f64))).ln()
            + 1.)
            .ceil() as usize;

        if let Some(p) = previous {
            if p < current {
                return (p, h - 1);
            }
        }

        h += 1;
        if current != 0 {
            previous = Some(current);
        }
    }
}

pub fn bloom_filter_indices<H: ElementHasher>(
    element: &usize,
    bin_count: usize,
    hash_count: usize,
) -> impl Iterator<Item = usize> + '_ {
    H::hash_element_multiple_seeds(
        element,
        &(0..hash_count)
            .map(|seed| seed as u64)
            .collect::<Vec<u64>>(),
    )
    .into_iter()
    .map(move |hash| hash % bin_count)
}

pub fn bloom_filter_contains<H: ElementHasher>(
    bins: &[bool],
    element: &usize,
    hash_count: usize,
) -> bool {
    let bin_count = bins.len();

    for index in bloom_filter_indices::<H>(element, bin_count, hash_count) {
        if !bins[index] {
            return false;
        }
    }

    true
}

impl Set {
    pub fn to_bloom_filter<H: ElementHasher>(
        &self,
        bin_count: usize,
        hash_count: usize,
    ) -> Vec<bool> {
        let mut bins = vec![false; bin_count];

        for element in &self.elements {
            for seed in 0..hash_count {
                bins[H::hash_element(element, seed as u64) % bin_count] = true;
            }
        }

        bins
    }
}

impl Multiset {
    pub fn to_bloom_filter<H: ElementHasher>(
        &self,
        bin_count: usize,
        hash_count: usize,
        max_multiplicity: usize,
    ) -> Vec<bool> {
        let mut bins = vec![false; bin_count];

        for (element, count) in &self.element_counts {
            for i in 0..*count {
                for seed in 0..hash_count {
                    bins[H::hash_element(&(*element * max_multiplicity + i), seed as u64)
                        % bin_count] = true;
                }
            }
        }

        bins
    }
}

pub fn bloom_filter_retrieve_count<H: ElementHasher>(
    bins: &[bool],
    element: &usize,
    hash_count: usize,
    max_multiplicity: usize,
) -> usize {
    for i in 0..max_multiplicity {
        if !bloom_filter_contains::<H>(bins, &(element * max_multiplicity + i), hash_count) {
            return i;
        }
    }

    max_multiplicity
}

#[cfg(test)]
mod tests {
    use crate::{
        bloom_filters::{bloom_filter_contains, bloom_filter_retrieve_count, Xxh3Hasher},
        multisets::Multiset,
        sets::Set,
    };

    use super::{gen_bloom_filter_params, gen_bloom_filter_params_log2};

    #[test]
    fn test_bf_parameters_smallrate() {
        let (bin_count, hash_count) = gen_bloom_filter_params(2f64.powf(-5.), 256);
        assert_eq!(bin_count, 1852);
        assert_eq!(hash_count, 5);
    }

    #[test]
    fn test_bf_parameters_largerate() {
        let (bin_count, hash_count) = gen_bloom_filter_params(2f64.powf(-10.), 4096);
        assert_eq!(bin_count, 59102);
        assert_eq!(hash_count, 10);
    }

    #[test]
    fn test_bf_parameters_smallrate_log() {
        let (bin_count, hash_count) = gen_bloom_filter_params_log2(-5., 256);
        assert_eq!(bin_count, 1852);
        assert_eq!(hash_count, 5);
    }

    #[test]
    fn test_bf_parameters_largerate_log() {
        let (bin_count, hash_count) = gen_bloom_filter_params_log2(-10., 4096);
        assert_eq!(bin_count, 59102);
        assert_eq!(hash_count, 10);
    }

    #[test]
    fn test_bf_parameters_tiny_log() {
        let (bin_count, hash_count) = gen_bloom_filter_params_log2(-160., 4096);
        assert_eq!(bin_count, 945602);
        assert_eq!(hash_count, 160);
    }
}

#[cfg(test)]
mod tests_xxh3 {
    use crate::bloom_filters::bloom_filter_contains;
    use crate::bloom_filters::bloom_filter_retrieve_count;
    use crate::bloom_filters::Xxh3Hasher;
    use crate::multisets::Multiset;
    use crate::sets::Set;

    type H = Xxh3Hasher;

    #[test]
    fn test_set_to_bloom_filter() {
        let set = Set::new(&vec![1, 3, 4]);
        let bloom_filter = set.to_bloom_filter::<H>(20, 2);

        assert!(bloom_filter_contains::<H>(&bloom_filter, &1, 2));
        assert!(!bloom_filter_contains::<H>(&bloom_filter, &2, 2));
        assert!(bloom_filter_contains::<H>(&bloom_filter, &3, 2));
        assert!(bloom_filter_contains::<H>(&bloom_filter, &4, 2));
        assert!(!bloom_filter_contains::<H>(&bloom_filter, &5, 2));
    }

    #[test]
    fn test_multiset_to_bloom_filter() {
        let multiset = Multiset::new(&vec![1, 3, 4], &vec![1, 2, 1]);
        let bloom_filter = multiset.to_bloom_filter::<H>(50, 2, 2);

        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &0, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &1, 2, 2), 1);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &2, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &3, 2, 2), 2);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &4, 2, 2), 1);
    }
}

#[cfg(test)]
mod tests_shake128 {
    use super::Shake128Hasher;
    use crate::bloom_filters::bloom_filter_contains;
    use crate::bloom_filters::bloom_filter_retrieve_count;
    use crate::multisets::Multiset;
    use crate::sets::Set;

    type H = Shake128Hasher;

    #[test]
    fn test_set_to_bloom_filter() {
        let set = Set::new(&vec![1, 3, 4]);
        let bloom_filter = set.to_bloom_filter::<H>(20, 2);

        assert!(bloom_filter_contains::<H>(&bloom_filter, &1, 2));
        assert!(!bloom_filter_contains::<H>(&bloom_filter, &2, 2));
        assert!(bloom_filter_contains::<H>(&bloom_filter, &3, 2));
        assert!(bloom_filter_contains::<H>(&bloom_filter, &4, 2));
        assert!(!bloom_filter_contains::<H>(&bloom_filter, &5, 2));
    }

    #[test]
    fn test_multiset_to_bloom_filter() {
        let multiset = Multiset::new(&vec![1, 3, 4], &vec![1, 2, 1]);
        let bloom_filter = multiset.to_bloom_filter::<H>(50, 2, 2);

        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &0, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &1, 2, 2), 1);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &2, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &3, 2, 2), 2);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &4, 2, 2), 1);
    }
}

#[cfg(test)]
mod tests_blake3 {
    use crate::bloom_filters::bloom_filter_contains;
    use crate::bloom_filters::bloom_filter_retrieve_count;
    use crate::multisets::Multiset;
    use crate::sets::Set;

    use super::Blake3Hasher;

    type H = Blake3Hasher;

    #[test]
    fn test_set_to_bloom_filter() {
        let set = Set::new(&vec![1, 3, 4]);
        let bloom_filter = set.to_bloom_filter::<H>(20, 2);

        assert!(bloom_filter_contains::<H>(&bloom_filter, &1, 2));
        assert!(!bloom_filter_contains::<H>(&bloom_filter, &2, 2));
        assert!(bloom_filter_contains::<H>(&bloom_filter, &3, 2));
        assert!(bloom_filter_contains::<H>(&bloom_filter, &4, 2));
        assert!(!bloom_filter_contains::<H>(&bloom_filter, &5, 2));
    }

    #[test]
    fn test_multiset_to_bloom_filter() {
        let multiset = Multiset::new(&vec![1, 3, 4], &vec![1, 2, 1]);
        let bloom_filter = multiset.to_bloom_filter::<H>(50, 2, 2);

        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &0, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &1, 2, 2), 1);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &2, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &3, 2, 2), 2);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &4, 2, 2), 1);
    }
}

#[cfg(test)]
mod tests_argon2 {
    use crate::bloom_filters::bloom_filter_contains;
    use crate::bloom_filters::bloom_filter_retrieve_count;
    use crate::multisets::Multiset;
    use crate::sets::Set;

    use super::Argon2Hasher;

    type H = Argon2Hasher;

    #[test]
    fn test_set_to_bloom_filter() {
        let set = Set::new(&vec![1, 3, 4]);
        let bloom_filter = set.to_bloom_filter::<H>(20, 2);
        println!("{:?}", bloom_filter);

        assert!(bloom_filter_contains::<H>(&bloom_filter, &1, 2));
        assert!(!bloom_filter_contains::<H>(&bloom_filter, &2, 2));
        assert!(bloom_filter_contains::<H>(&bloom_filter, &3, 2));
        assert!(bloom_filter_contains::<H>(&bloom_filter, &4, 2));
        assert!(!bloom_filter_contains::<H>(&bloom_filter, &5, 2));
    }

    #[test]
    fn test_multiset_to_bloom_filter() {
        let multiset = Multiset::new(&vec![1, 3, 4], &vec![1, 2, 1]);
        let bloom_filter = multiset.to_bloom_filter::<H>(50, 2, 2);

        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &0, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &1, 2, 2), 1);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &2, 2, 2), 0);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &3, 2, 2), 2);
        assert_eq!(bloom_filter_retrieve_count::<H>(&bloom_filter, &4, 2, 2), 1);
    }
}
