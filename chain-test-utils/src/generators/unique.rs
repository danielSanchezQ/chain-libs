use smoke::generator::SuchThat;
use smoke::{Generator, R};
use std::collections::HashSet;
use std::hash::Hash;

struct UniqueItemGenerator<Gen: Generator> {
    produced: HashSet<Gen::Item>,
    generator: Gen,
}

impl<Gen: Generator> UniqueItemGenerator<Gen> {
    pub fn new(gen: Gen) -> Self
    where
        Gen::Item: Clone + Hash + Eq,
    {
        Self {
            produced: HashSet::new(),
            generator: gen,
        }
    }

    fn has_item(&self) -> bool {
        true
    }

    // fn next_unique_item(&self, r: &mut R) -> Gen::Item
    // where
    //     Gen::Item: Clone + Hash + Eq,
    // {
    //     loop {
    //         let new_item = self.generator.gen(r);
    //         if !self.produced.contains(&new_item) {
    //             self.produced.insert(new_item.clone());
    //             return new_item;
    //         }
    //     }
    // }
}

impl<Gen: Generator> Generator for UniqueItemGenerator<Gen> {
    type Item = Gen::Item;

    fn gen(&self, r: &mut R) -> Self::Item {
        self.such_that(|item| self.produced.contains(&item))
            .into_boxed()
            .gen(r)
    }
}
