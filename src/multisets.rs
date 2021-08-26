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
    use crate::multisets::Multiset;

    #[test]
    fn test_multiset_from_iter() {
        let elements = vec![1usize, 3, 4];
        let counts = vec![2usize, 2usize, 5usize];

        let multiset_a = Multiset::new(&elements, &counts);
        let multiset_b: Multiset = elements.into_iter().zip(counts).collect();

        assert_eq!(multiset_a, multiset_b);
    }
}
