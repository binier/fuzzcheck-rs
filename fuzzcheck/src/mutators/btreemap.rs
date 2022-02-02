use std::collections::BTreeMap;

use super::map::MapMutator;
use super::wrapper::Wrapper;
use super::{tuples::TupleMutatorWrapper, vector::VecMutator};
use crate::mutators::tuples::Tuple2;
use crate::{mutators::tuples::Tuple2Mutator, DefaultMutator};

type AssociativeVecMutator<K, V> = VecMutator<
    (K, V),
    TupleMutatorWrapper<Tuple2Mutator<<K as DefaultMutator>::Mutator, <V as DefaultMutator>::Mutator>, Tuple2<K, V>>,
>;

pub type BTreeMapMutator<K, V> = Wrapper<
    MapMutator<
        Vec<(K, V)>,
        BTreeMap<K, V>,
        AssociativeVecMutator<K, V>,
        fn(&BTreeMap<K, V>) -> Option<Vec<(K, V)>>,
        fn(&Vec<(K, V)>) -> BTreeMap<K, V>,
        fn(&BTreeMap<K, V>, f64) -> f64,
    >,
>;

#[no_coverage]
fn avec_from_btreemap<K: Clone, V: Clone>(btree: &BTreeMap<K, V>) -> Option<Vec<(K, V)>> {
    Some(btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

#[no_coverage]
fn btreemap_from_avec<K: Clone + Ord, V: Clone>(avec: &Vec<(K, V)>) -> BTreeMap<K, V> {
    BTreeMap::<K, V>::from_iter(avec.iter().map(|(k, v)| (k.clone(), v.clone())))
}

#[no_coverage]
fn complexity<K: Clone, V: Clone>(_t: &BTreeMap<K, V>, cplx: f64) -> f64 {
    cplx
}

impl<K, V> BTreeMapMutator<K, V>
where
    K: Clone + Ord + DefaultMutator,
    V: Clone + DefaultMutator,
{
    #[no_coverage]
    pub fn new() -> Self {
        Wrapper(MapMutator::new(
            VecMutator::new(
                TupleMutatorWrapper::new(Tuple2Mutator::new(K::default_mutator(), V::default_mutator())),
                0..=10,
            ),
            avec_from_btreemap,
            btreemap_from_avec,
            complexity,
        ))
    }
}

impl<K, V> DefaultMutator for BTreeMap<K, V>
where
    K: 'static + Clone + Ord + DefaultMutator,
    V: 'static + Clone + DefaultMutator,
{
    type Mutator = BTreeMapMutator<K, V>;
    #[no_coverage]
    fn default_mutator() -> Self::Mutator {
        Self::Mutator::new()
    }
}
