//! Types to combine multiple sensors and pools together
//!
//! If we have two tuples of compatible sensors and pools:
//! * `s1` and `p1`
//! * `s2` and `p2`
//!
//! Then we can combine them into a single sensor and pool as follows:
//! ```
//! use fuzzcheck::sensors_and_pools::{AndSensor, AndPool, DifferentObservations};
//! # use fuzzcheck::sensors_and_pools::{NoopSensor, UniqueValuesPool};
//! # let (s1, s2) = (NoopSensor, NoopSensor);
//! # let (p1, p2) = (UniqueValuesPool::<u8>::new("a", 0), UniqueValuesPool::<bool>::new("b", 0));
//! let s = AndSensor(s1, s2);
//! let p = AndPool::<_, _, DifferentObservations>::new(p1, p2, 128);
//! // 128 is the ratio of times the first pool is chosen when selecting a test case to mutate.
//! // The implicit denominator is 256. So the first pool is chosen 128 / 256 = 50% of the time.
//! ```
//!
//! At every iteration of the fuzz test, both pools have a chance to provide a test case to mutate.
//! After the test function is run, both sensors will collect data and feed them to their respective pool.
use std::{fmt::Display, marker::PhantomData, path::PathBuf};

use crate::{
    traits::{CompatibleWithObservations, CorpusDelta, Pool, SaveToStatsFolder, Sensor, SensorAndPool, Stats},
    CSVField, PoolStorageIndex, ToCSV,
};
pub struct SameObservations;
pub struct DifferentObservations;

/// A pool that combines two pools
pub struct AndPool<P1, P2, SensorMarker>
where
    P1: Pool,
    P2: Pool,
{
    pub p1: P1,
    pub p2: P2,

    pub p1_weight: f64,
    pub p2_weight: f64,

    p1_number_times_chosen_since_last_progress: usize,
    p2_number_times_chosen_since_last_progress: usize,

    rng: fastrand::Rng,
    _phantom: PhantomData<SensorMarker>,
}
impl<P1, P2, SensorMarker> AndPool<P1, P2, SensorMarker>
where
    P1: Pool,
    P2: Pool,
{
    #[no_coverage]
    pub fn new(p1: P1, p2: P2, p1_weight: f64, p2_weight: f64) -> Self {
        Self {
            p1,
            p2,
            p1_weight,
            p2_weight,
            p1_number_times_chosen_since_last_progress: 1,
            p2_number_times_chosen_since_last_progress: 1,
            rng: fastrand::Rng::new(),
            _phantom: PhantomData,
        }
    }
}
impl<P1, P2, SensorMarker> AndPool<P1, P2, SensorMarker>
where
    P1: Pool,
    P2: Pool,
{
    fn p1_weight(&self) -> f64 {
        self.p1_weight / self.p1_number_times_chosen_since_last_progress as f64
    }
    fn p2_weight(&self) -> f64 {
        self.p2_weight / self.p2_number_times_chosen_since_last_progress as f64
    }
}
impl<P1, P2, SensorMarker> Pool for AndPool<P1, P2, SensorMarker>
where
    P1: Pool,
    P2: Pool,
{
    type Stats = AndPoolStats<P1::Stats, P2::Stats>;

    #[no_coverage]
    fn stats(&self) -> Self::Stats {
        AndPoolStats(self.p1.stats(), self.p2.stats())
    }
    #[no_coverage]
    fn get_random_index(&mut self) -> Option<PoolStorageIndex> {
        let choice = self.rng.f64() * self.weight();
        if choice <= self.p1_weight() {
            if let Some(idx) = self.p1.get_random_index() {
                self.p1_number_times_chosen_since_last_progress += 1;
                Some(idx)
            } else {
                self.p2_number_times_chosen_since_last_progress += 1;
                self.p2.get_random_index()
            }
        } else if let Some(idx) = self.p2.get_random_index() {
            self.p2_number_times_chosen_since_last_progress += 1;
            Some(idx)
        } else {
            self.p1_number_times_chosen_since_last_progress += 1;
            self.p1.get_random_index()
        }
    }

    fn weight(&self) -> f64 {
        self.p1_weight() + self.p2_weight()
    }
}

impl<P1, P2, SensorMarker> SaveToStatsFolder for AndPool<P1, P2, SensorMarker>
where
    P1: Pool,
    P2: Pool,
{
    #[no_coverage]
    fn save_to_stats_folder(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut x = self.p1.save_to_stats_folder();
        x.extend(self.p2.save_to_stats_folder());
        x
    }
}

/// A sensor that combines two sensors
///
/// This type assumes nothing about the relationship between the two sensors.
/// It is most likely that you are also using two different pools to process
/// each sensor’s observations. Then, you can use an [`AndPool`] to combine these
/// two pools and make them compatible with this `AndSensor`.
pub struct AndSensor<S1, S2>(pub S1, pub S2)
where
    S1: Sensor,
    S2: Sensor;

impl<S1, S2> Sensor for AndSensor<S1, S2>
where
    S1: Sensor,
    S2: Sensor,
{
    type Observations = (S1::Observations, S2::Observations);

    #[no_coverage]
    fn start_recording(&mut self) {
        self.0.start_recording();
        self.1.start_recording();
    }
    #[no_coverage]
    fn stop_recording(&mut self) {
        self.0.stop_recording();
        self.1.stop_recording();
    }
    #[no_coverage]
    fn get_observations(&mut self) -> Self::Observations {
        (self.0.get_observations(), self.1.get_observations())
    }
}

impl<S1, S2> SaveToStatsFolder for AndSensor<S1, S2>
where
    S1: Sensor,
    S2: Sensor,
{
    #[no_coverage]
    fn save_to_stats_folder(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut x = self.0.save_to_stats_folder();
        x.extend(self.1.save_to_stats_folder());
        x
    }
}

/// The statistics of an [AndPool]
#[derive(Clone)]
pub struct AndPoolStats<S1: Display, S2: Display>(pub S1, pub S2);
impl<S1: Display, S2: Display> Display for AndPoolStats<S1, S2> {
    #[no_coverage]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}
impl<S1: Display, S2: Display> Stats for AndPoolStats<S1, S2>
where
    S1: Stats,
    S2: Stats,
{
}

impl<O1, O2, P1, P2> CompatibleWithObservations<(O1, O2)> for AndPool<P1, P2, DifferentObservations>
where
    P1: Pool,
    P2: Pool,
    P1: CompatibleWithObservations<O1>,
    P2: CompatibleWithObservations<O2>,
{
    #[no_coverage]
    fn process(&mut self, input_id: PoolStorageIndex, observations: &(O1, O2), complexity: f64) -> Vec<CorpusDelta> {
        let AndPool {
            p1,
            p2,
            p1_number_times_chosen_since_last_progress,
            p2_number_times_chosen_since_last_progress,
            ..
        } = self;
        let deltas_1 = p1.process(input_id, &observations.0, complexity);
        if !deltas_1.is_empty() {
            *p1_number_times_chosen_since_last_progress = 1;
        }
        let deltas_2 = p2.process(input_id, &observations.1, complexity);
        if !deltas_2.is_empty() {
            *p2_number_times_chosen_since_last_progress = 1;
        }
        let mut deltas = deltas_1;
        deltas.extend(deltas_2);
        deltas
    }
}

impl<P1, P2, O> CompatibleWithObservations<O> for AndPool<P1, P2, SameObservations>
where
    P1: CompatibleWithObservations<O>,
    P2: CompatibleWithObservations<O>,
{
    #[no_coverage]
    fn process(&mut self, input_id: PoolStorageIndex, observations: &O, complexity: f64) -> Vec<CorpusDelta> {
        let AndPool {
            p1,
            p2,
            p1_number_times_chosen_since_last_progress,
            p2_number_times_chosen_since_last_progress,
            ..
        } = self;
        let deltas_1 = p1.process(input_id, observations, complexity);
        if !deltas_1.is_empty() {
            *p1_number_times_chosen_since_last_progress = 1;
        }
        let deltas_2 = p2.process(input_id, observations, complexity);
        if !deltas_2.is_empty() {
            *p2_number_times_chosen_since_last_progress = 1;
        }
        let mut deltas = deltas_1;
        deltas.extend(deltas_2);
        deltas
    }
}

impl<S1, S2> ToCSV for AndPoolStats<S1, S2>
where
    S1: Display,
    S2: Display,
    S1: ToCSV,
    S2: ToCSV,
{
    #[no_coverage]
    fn csv_headers(&self) -> Vec<CSVField> {
        let mut h = self.0.csv_headers();
        h.extend(self.1.csv_headers());
        h
    }

    #[no_coverage]
    fn to_csv_record(&self) -> Vec<CSVField> {
        let mut h = self.0.to_csv_record();
        h.extend(self.1.to_csv_record());
        h
    }
}

/// Combines two [`SensorAndPool`](crate::traits::SensorAndPool) trait objects into one.
pub struct AndSensorAndPool {
    sap1: Box<dyn SensorAndPool>,
    sap2: Box<dyn SensorAndPool>,
    sap1_weight: f64,
    sap2_weight: f64,
    sap1_number_times_chosen_since_last_progress: usize,
    sap2_number_times_chosen_since_last_progress: usize,
    rng: fastrand::Rng,
}
impl AndSensorAndPool {
    #[no_coverage]
    pub fn new(sap1: Box<dyn SensorAndPool>, sap2: Box<dyn SensorAndPool>, sap1_weight: f64, sap2_weight: f64) -> Self {
        Self {
            sap1,
            sap2,
            sap1_weight,
            sap2_weight,
            sap1_number_times_chosen_since_last_progress: 1,
            sap2_number_times_chosen_since_last_progress: 1,
            rng: fastrand::Rng::new(),
        }
    }
}
impl SaveToStatsFolder for AndSensorAndPool {
    #[no_coverage]
    fn save_to_stats_folder(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut x = self.sap1.save_to_stats_folder();
        x.extend(self.sap2.save_to_stats_folder());
        x
    }
}
impl SensorAndPool for AndSensorAndPool {
    #[no_coverage]
    fn stats(&self) -> Box<dyn crate::traits::Stats> {
        Box::new(AndPoolStats(self.sap1.stats(), self.sap2.stats()))
    }

    #[no_coverage]
    fn start_recording(&mut self) {
        self.sap1.start_recording();
        self.sap2.start_recording();
    }

    #[no_coverage]
    fn stop_recording(&mut self) {
        self.sap1.stop_recording();
        self.sap2.stop_recording();
    }

    #[no_coverage]
    fn process(&mut self, input_id: PoolStorageIndex, cplx: f64) -> Vec<CorpusDelta> {
        let AndSensorAndPool {
            sap1,
            sap2,
            sap1_number_times_chosen_since_last_progress,
            sap2_number_times_chosen_since_last_progress,
            ..
        } = self;
        let deltas_1 = sap1.process(input_id, cplx);
        if !deltas_1.is_empty() {
            *sap1_number_times_chosen_since_last_progress = 1;
        }
        let deltas_2 = sap2.process(input_id, cplx);
        if !deltas_2.is_empty() {
            *sap2_number_times_chosen_since_last_progress = 1;
        }
        let mut deltas = deltas_1;
        deltas.extend(deltas_2);
        deltas
    }

    #[no_coverage]
    fn get_random_index(&mut self) -> Option<PoolStorageIndex> {
        let sum_weight = self.sap1_weight + self.sap2_weight;
        if self.rng.f64() <= sum_weight {
            if let Some(idx) = self.sap1.get_random_index() {
                self.sap1_number_times_chosen_since_last_progress += 1;
                Some(idx)
            } else {
                self.sap2_number_times_chosen_since_last_progress += 1;
                self.sap2.get_random_index()
            }
        } else if let Some(idx) = self.sap2.get_random_index() {
            self.sap2_number_times_chosen_since_last_progress += 1;
            Some(idx)
        } else {
            self.sap1_number_times_chosen_since_last_progress += 1;
            self.sap1.get_random_index()
        }
    }
}
