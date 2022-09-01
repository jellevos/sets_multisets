use rand::rngs::OsRng;
use rand::seq::index::sample;
use rand::Rng;
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

    /// `max_multiplicity` is inclusive, so `max_multiplicity = 5` will generate counts that are
    /// uniformly chosen from 1, 2, 3, 4, 5.
    pub fn random(element_count: usize, universe: usize, max_multiplicity: usize) -> Self {
        let elements = sample(&mut OsRng, universe, element_count).into_iter();
        let counts = (0..element_count).map(|_| OsRng.gen_range(1..=max_multiplicity));

        Multiset {
            element_counts: elements.zip(counts).collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.element_counts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.element_counts.is_empty()
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
}

#[derive(Eq, PartialEq, Debug, Clone)]
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
    use crate::multisets::Multiset;

    #[test]
    fn test_random() {
        let multiset1 = Multiset::random(5, 100, 10);
        let multiset2 = Multiset::random(5, 100, 10);

        assert_eq!(multiset1.len(), 5);
        assert_eq!(multiset2.len(), 5);

        assert_ne!(multiset1, multiset2);

        for (element, count) in multiset1.element_counts {
            assert!(element < 100);
            assert!(count > 0);
            assert!(count <= 10);
        }
        for (element, count) in multiset2.element_counts {
            assert!(element < 100);
            assert!(count > 0);
            assert!(count <= 10);
        }
    }

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
}
