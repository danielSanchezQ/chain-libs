use crate::generators::utils::{Generator, R};
use std::cell::RefCell;
use std::collections::HashSet;

/// `Unique` filters the generated items (from other generator) so there are no duplicates.
/// Internally it uses a `HashSet` to store the visited items.
/// ⚠ For huge quantities of generated items it may lead to huge times on finding unique items
/// For example, generate 100 unique random `u64` numbers:
/// ```
/// use smoke::generator::{num, Generator};
/// use std::collections::HashSet;
/// use chain_test_utils::generators::unique::Unique;
///
/// let unique_generator: Unique<u64, _> = Unique::new(num());
/// let (_, mut r) = smoke::R::new();
/// let set: HashSet<u64> = (0..100).map(|_| unique_generator.gen(&mut r)).collect();
/// ```
pub struct Unique<T, Gen: Generator<Item = T>> {
    inner_generator: Gen,
    visited: RefCell<HashSet<T>>,
}

impl<T, Gen: Generator<Item = T>> Unique<T, Gen> {
    pub fn new(inner_generator: Gen) -> Self {
        Self {
            inner_generator,
            visited: RefCell::new(HashSet::new()),
        }
    }
}

impl<T, Gen: Generator<Item = T>> Generator for Unique<T, Gen>
where
    T: std::hash::Hash + Eq + Clone,
{
    type Item = T;

    fn gen(&self, r: &mut R) -> Self::Item {
        let mut visited = self.visited.borrow_mut();
        loop {
            let new_item = self.inner_generator.gen(r);
            if !visited.contains(&new_item) {
                visited.insert(new_item.clone());
                return new_item;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::generators::unique::Unique;
    use smoke::generator::{num, Generator};
    use std::collections::HashSet;

    #[test]
    fn unique_range() {
        let unique_generator: Unique<u64, _> = Unique::new(num());
        let (_, mut r) = smoke::R::new();
        let set: HashSet<u64> = (0..100).map(|_| unique_generator.gen(&mut r)).collect();
        assert_eq!(set.len(), 100);
    }
}
