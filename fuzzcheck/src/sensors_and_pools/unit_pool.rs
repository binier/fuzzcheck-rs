use std::path::PathBuf;

use crate::sensors_and_pools::stats::EmptyStats;
use crate::traits::{CorpusDelta, Pool, SaveToStatsFolder};
use crate::CompatibleWithObservations;
use crate::PoolStorageIndex;

/// A pool that stores only one given test case.
///
/// Currently, it can only be used by fuzzcheck itself
/// because it requires a `PoolStorageIndex`, which only
/// fuzzcheck can create. This will change at some point.
pub struct UnitPool {
    input_index: PoolStorageIndex,
}
impl UnitPool {
    #[no_coverage]
    pub(crate) fn new(input_index: PoolStorageIndex) -> Self {
        Self { input_index }
    }
}

impl Pool for UnitPool {
    type Stats = EmptyStats;
    #[no_coverage]
    fn stats(&self) -> Self::Stats {
        EmptyStats
    }

    #[no_coverage]
    fn get_random_index(&mut self) -> Option<PoolStorageIndex> {
        Some(self.input_index)
    }
}
impl SaveToStatsFolder for UnitPool {
    #[no_coverage]
    fn save_to_stats_folder(&self) -> Vec<(PathBuf, Vec<u8>)> {
        vec![]
    }
}

impl<O> CompatibleWithObservations<O> for UnitPool {
    #[no_coverage]
    fn process<'a>(&'a mut self, _input_id: PoolStorageIndex, _observations: &O, _complexity: f64) -> Vec<CorpusDelta> {
        vec![]
    }
}
