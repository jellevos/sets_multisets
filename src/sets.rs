use rand::rngs::OsRng;
use rand::seq::index::sample;
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp;
use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Eq, PartialEq, Debug, Clone)]
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

    pub fn contains(&self, element: &usize) -> bool {
        self.elements.contains(element)
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

    pub fn unify(&self, other: &Set) -> Set {
        Set {
            elements: self.elements.union(&other.elements).copied().collect(),
        }
    }

    pub fn union(sets: &[Set]) -> Set {
        let mut result = sets[0].unify(&sets[1]);

        for set in &sets[2..] {
            result = result.unify(set);
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
}

/// Generates `set_count` random sets so that the size of the intersection is `intersection_size`.
pub fn gen_sets_with_intersection(
    set_count: usize,
    element_count: usize,
    universe: usize,
    intersection_size: usize,
) -> Vec<Set> {
    let intersection = Set::random(intersection_size, universe);

    let mut sets: Vec<Set> = (0..set_count).map(|_| intersection.clone()).collect();

    // Fill with other random elements
    for i in 0..set_count {
        while sets[i].len() < element_count {
            let element = OsRng.gen_range(0..universe);

            // Check if at least one of the other sets does not contain this element
            let mut can_insert = false;
            for (j, set) in sets.iter().enumerate() {
                if i == j {
                    continue;
                }

                if !set.contains(&element) {
                    can_insert = true;
                    break;
                }
            }

            if can_insert && !sets[i].contains(&element) {
                sets[i].elements.insert(element);
            }
        }
    }

    sets
}

/// Generates `set_count` random sets so that the size of the union is `union_size`.
pub fn gen_sets_with_union(
    set_count: usize,
    element_count: usize,
    universe: usize,
    union_size: usize,
) -> Vec<Set> {
    let union = Set::random(union_size, universe);

    let mut sets = vec![vec![]; set_count];

    // Distribute elements randomly
    for element in &union.elements {
        loop {
            let index = OsRng.gen_range(0..set_count);
            if sets[index].len() < element_count {
                sets[index].push(*element);
                break;
            }
        }
    }

    // Fill with other random elements
    for set in sets.iter_mut() {
        let mut elements = union.elements.iter().collect::<Vec<&usize>>();
        elements.shuffle(&mut OsRng);
        let mut shuffled_elements = elements.into_iter();

        while set.len() < element_count {
            let element = shuffled_elements.next().unwrap();
            if !set.contains(element) {
                set.push(*element);
            }
        }
    }

    sets.iter().map(|set| Set::new(set)).collect()
}

/// Generates `set_count` random sets so that the size of the intersection is decided uniformly at random (it is at least 1).
pub fn gen_sets_with_uniform_intersection(
    set_count: usize,
    element_count: usize,
    universe: usize,
) -> Vec<Set> {
    let intersection_size = OsRng.gen_range(
        cmp::max(
            1,
            (set_count * element_count) as isize - (universe * (set_count - 1)) as isize,
        ) as usize..=element_count,
    );
    gen_sets_with_intersection(set_count, element_count, universe, intersection_size)
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
    use crate::sets::{gen_sets_with_intersection, gen_sets_with_union, Set};

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
    fn test_set_from_iter() {
        let elements = vec![1usize, 3, 4];
        let set: Set = elements.iter().map(|e| *e).collect();
        assert_eq!(Set::new(&elements), set);
    }

    #[test]
    fn test_gen_sets_with_intersection() {
        let sets = gen_sets_with_intersection(3, 10, 100, 4);
        assert_eq!(Set::intersection(&sets).len(), 4);
        assert_eq!(sets[2].len(), 10);
        assert_ne!(sets[0], sets[1]);
    }

    #[test]
    fn test_gen_sets_with_union() {
        let sets = gen_sets_with_union(3, 10, 100, 20);
        assert_eq!(Set::union(&sets).len(), 20);
        assert_eq!(sets[2].len(), 10);
        assert_ne!(sets[0], sets[1]);
    }
}
