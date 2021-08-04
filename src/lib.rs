use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Eq, PartialEq, Debug)]
pub struct Set {
    pub elements: HashSet<usize>,
}

impl Set {

    pub fn new(elements: &[usize]) -> Self {
        Set {
            elements: HashSet::from_iter(elements.iter().copied()),
        }
    }

    pub fn intersect(&self, other: &Set) -> Set {
        Set {
            elements: HashSet::from_iter(self.elements.intersection(&other.elements).copied()),
        }
    }

    pub fn intersection(sets: &[Set]) -> Set {
        let mut result = sets[0].intersect(&sets[1]);

        for set in &sets[2..] {
            result = result.intersect(set);
        }

        result
    }

}

#[cfg(test)]
mod tests {
    use crate::Set;

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
}
