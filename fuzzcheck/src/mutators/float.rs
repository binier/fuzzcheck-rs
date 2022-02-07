use super::integer::U64Mutator;
use super::map::MapMutator;
use super::wrapper::Wrapper;

use crate::DefaultMutator;

pub type F64Mutator =
    Wrapper<MapMutator<u64, f64, U64Mutator, fn(&f64) -> Option<u64>, fn(&u64) -> f64, fn(&f64, f64) -> f64>>;

#[no_coverage]
fn u64_from_f64(f: &f64) -> Option<u64> {
    Some(f.to_bits())
}

#[no_coverage]
fn f64_from_u64(u: &u64) -> f64 {
    f64::from_bits(*u)
}

#[no_coverage]
fn complexity(_t: &f64, cplx: f64) -> f64 {
    cplx
}

impl F64Mutator {
    #[no_coverage]
    pub fn new() -> Self {
        Wrapper(MapMutator::new(
            U64Mutator::default(),
            u64_from_f64,
            f64_from_u64,
            complexity,
        ))
    }
}

impl DefaultMutator for f64 {
    type Mutator = F64Mutator;
    #[no_coverage]
    fn default_mutator() -> Self::Mutator {
        Self::Mutator::new()
    }
}
