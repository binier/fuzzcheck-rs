use std::time::Duration;

use super::integer::U64Mutator;
use super::map::MapMutator;
use super::wrapper::Wrapper;

use crate::DefaultMutator;

pub type DurationMutator = Wrapper<
    MapMutator<
        u64,
        Duration,
        U64Mutator,
        fn(&Duration) -> Option<u64>,
        fn(&u64) -> Duration,
        fn(&Duration, f64) -> f64,
    >,
>;

#[no_coverage]
fn u64_from_duration(d: &Duration) -> Option<u64> {
    Some(d.as_secs())
}

#[no_coverage]
fn duration_from_u64(u: &u64) -> Duration {
    Duration::from_secs(*u)
}

#[no_coverage]
fn complexity(_t: &Duration, cplx: f64) -> f64 {
    cplx
}

impl DurationMutator {
    #[no_coverage]
    pub fn new() -> Self {
        Wrapper(MapMutator::new(
            U64Mutator::default(),
            u64_from_duration,
            duration_from_u64,
            complexity,
        ))
    }
}

impl DefaultMutator for Duration {
    type Mutator = DurationMutator;
    #[no_coverage]
    fn default_mutator() -> Self::Mutator {
        Self::Mutator::new()
    }
}
