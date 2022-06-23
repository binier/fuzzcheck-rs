use std::collections::BTreeSet;

use super::map::MapMutator;
use super::vector::VecMutator;
use super::wrapper::Wrapper;
use crate::DefaultMutator;

type AssociativeVecMutator<T> = VecMutator<T, <T as DefaultMutator>::Mutator>;

pub type BTreeSetMutator<T> = Wrapper<
    MapMutator<
        Vec<T>,
        BTreeSet<T>,
        AssociativeVecMutator<T>,
        fn(&BTreeSet<T>) -> Option<Vec<T>>,
        fn(&Vec<T>) -> BTreeSet<T>,
        fn(&BTreeSet<T>, f64) -> f64,
    >,
>;

#[no_coverage]
fn avec_from_btreeset<T: Clone>(btree: &BTreeSet<T>) -> Option<Vec<T>> {
    Some(btree.iter().cloned().collect())
}

#[no_coverage]
fn btreeset_from_avec<T: Clone + Ord>(avec: &Vec<T>) -> BTreeSet<T> {
    avec.iter().cloned().collect()
    // BTreeSet::<T>::from_iter()
}

#[no_coverage]
fn complexity<T: Clone>(_t: &BTreeSet<T>, cplx: f64) -> f64 {
    cplx
}

impl<T> BTreeSetMutator<T>
where
    T: Clone + Ord + DefaultMutator,
{
    #[no_coverage]
    pub fn new() -> Self {
        Wrapper(MapMutator::new(
            VecMutator::new(T::default_mutator(), 0..=10),
            avec_from_btreeset,
            btreeset_from_avec,
            complexity,
        ))
    }
}

impl<T> DefaultMutator for BTreeSet<T>
where
    T: 'static + Clone + Ord + DefaultMutator,
    T::Mutator: Clone,
{
    type Mutator = BTreeSetMutator<T>;
    #[no_coverage]
    fn default_mutator() -> Self::Mutator {
        Self::Mutator::new()
    }
}
