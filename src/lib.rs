use bytevec::ByteEncodable;
use fasthash::xx;

pub mod multisets;
pub mod sets;

pub fn bloom_filter_contains(bins: &[bool], element: &usize, hash_count: usize) -> bool {
    let bin_count = bins.len();

    let element_bytes = (*element as u64).encode::<u64>().unwrap();

    for seed in 0..hash_count {
        if !bins[xx::hash32_with_seed(&element_bytes, seed as u32) as usize % bin_count] {
            return false;
        }
    }

    true
}
