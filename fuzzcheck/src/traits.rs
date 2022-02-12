/*!
The fuzzcheck_traits crate defines the `Mutator` and `Serializer` traits
used by all fuzzcheck-related crates.
*/

use crate::fuzzer::PoolStorageIndex;
use crate::sensors_and_pools::{AndPool, MapSensor};
use fuzzcheck_common::FuzzerEvent;
use std::fmt::Display;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use crate::fuzzer::PoolStorageIndex;
use fuzzcheck_common::FuzzerEvent;

/**
 A [`Mutator`] is an object capable of generating/mutating a value for the purpose of
 fuzz-testing.

 For example, a mutator could change the value
 `v1 = [1, 4, 2, 1]` to `v1' = [1, 5, 2, 1]`.
 The idea is that if `v1` is an “interesting” value to test, then `v1'` also
 has a high chance of being “interesting” to test.

 Fuzzcheck itself provides a few mutators for `std` types as well as procedural macros
 to generate mutators. See the [`mutators`](crate::mutators) module.

 ## Complexity

 A mutator is also responsible for keeping track of the
 [complexity](crate::Mutator::complexity) of a value. The complexity is,
 roughly speaking, how large the value is.

 If the complexity is `0.0`, then it means the mutator can only produce one value.

 For example, the complexity of a vector could be the sum of the complexities
 of its elements. So `vec![]` would have a complexity of `1.0` (what we chose as
 the base complexity of a vector) and `vec![76]` would have a complexity of
 `9.0`: `1.0` for the base complexity of the vector itself + `8.0` for the 8-bit
 integer “76”. But there is no fixed rule for how to compute the complexity of a
 value, and it is up to you to judge how “large” something is.

 ## [`Cache`](Mutator::Cache)

 In order to mutate values efficiently, the mutator is able to make use of a
 per-value *cache*. The [`Cache`](Mutator::Cache) contains information associated
 with the value that will make it faster to compute its complexity or apply a
 mutation to it. For a vector, its cache is its total complexity, along with a
 vector of the caches of each of its element.

 ## [`MutationStep`](Mutator::MutationStep)

 The same values will be passed to the mutator many times, so that it is
 mutated in many different ways. There are different strategies to choose
 what mutation to apply to a value. The first one is to create a list of
 mutation operations, and choose one to apply randomly from this list.

 However, one may want to have better control over which mutation operation
 is used. For example, if the value to be mutated is of type `Option<T>`,
 then you may want to first mutate it to `None`, and then always mutate it
 to another `Some(t)`. This is where [`MutationStep`](Mutator::MutationStep)
 comes in. The mutationstep is a type you define to allow you to keep track
 of which mutation operation has already been tried. This allows you to
 deterministically apply mutations to a value such that better mutations are
 tried first, and duplicate mutations are avoided.

 It is not always possible to schedule mutations in order. For that reason,
 we have two methods: [`random_mutate`](crate::Mutator::random_mutate) executes
 a random mutation, and [`ordered_mutate`](crate::Mutator::ordered_mutate) uses
 the [`MutationStep`](Mutator::MutationStep) to schedule mutations in order.
 The fuzzing engine only ever uses [`ordered_mutate`](crate::Mutator::ordered_mutate)
 directly, but the former is sometimes necessary to compose mutators together.

 If you don't want to bother with ordered mutations, that is fine. In that
 case, only implement [`random_mutate`](crate::Mutator::random_mutate) and call it from
 the [`ordered_mutate`](crate::Mutator::ordered_mutate) method.
 ```ignore
 fn random_mutate(&self, value: &mut Value, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64) {
      // ...
 }
 fn ordered_mutate(&self, value: &mut Value, cache: &mut Self::Cache, step: &mut Self::MutationStep, max_cplx: f64) -> Option<(Self::UnmutateToken, f64)> {
     Some(self.random_mutate(value, cache, max_cplx))
 }
 ```

 ## Arbitrary

 A mutator must also be able to generate new values from nothing. This is what
 the [`random_arbitrary`](crate::Mutator::random_arbitrary) and
 [`ordered_arbitrary`](crate::Mutator::ordered_arbitrary) methods are for. The
 latter one is called by the fuzzer directly and uses an
 [`ArbitraryStep`](Mutator::ArbitraryStep) that can be used to smartly generate
 more interesting values first and avoid duplicates.

 ## Unmutate

 Finally, it is important to note that values and caches are mutated
 *in-place*. The fuzzer does not clone them before handing them to the
 mutator. Therefore, the mutator also needs to know how to reverse each
 mutation it performed. To do so, each mutation needs to return a token
 describing how to reverse it. The [unmutate](crate::Mutator::unmutate)
 method will later be called with that token to get the original value
 and cache back.

 For example, if the value is `[[1, 3], [5], [9, 8]]`, the mutator may
 mutate it to `[[1, 3], [5], [9, 1, 8]]` and return the token:
 `Element(2, Remove(1))`, which means that in order to reverse the
 mutation, the element at index 2 has to be unmutated by removing
 its element at index 1. In pseudocode:

 ```
 use fuzzcheck::Mutator;

# use fuzzcheck::DefaultMutator;
# let m = bool::default_mutator();
# let mut value = false;
# let mut cache = m.validate_value(&value).unwrap();
# let mut step = m.default_mutation_step(&value, &cache);
# let max_cplx = 8.0;
# fn test(x: &bool) {}
//  value = [[1, 3], [5], [9, 8]];
//  cache: c1 (ommitted from example)
//  step: s1 (ommitted from example)

 let (unmutate_token, _cplx) = m.ordered_mutate(&mut value, &mut cache, &mut step, max_cplx).unwrap();

 // value = [[1, 3], [5], [9, 1, 8]]
 // token = Element(2, Remove(1))
 // cache = c2
 // step = s2

 test(&value);

 m.unmutate(&mut value, &mut cache, unmutate_token);

 // value = [[1, 3], [5], [9, 8]]
 // cache = c1 (back to original cache)
 // step = s2 (step has not been reversed)
 ```

When a mutated value is deemed interesting by the fuzzing engine, the method
[`validate_value`](crate::Mutator::validate_value) is called on it in order to
get a new Cache and MutationStep for it. The same method is called when the
fuzzer reads values from a corpus to verify that they conform to the
mutator’s expectations. For example, a [`CharWithinRangeMutator`](crate::mutators::char::CharWithinRangeMutator)
will check whether the character is within a certain range.

Note that in most cases, it is completely fine to never mutate a value’s cache,
since it is recomputed by [`validate_value`](crate::Mutator::validate_value) when
needed.
**/
pub trait Mutator<Value: Clone>: 'static {
    /// Accompanies each value to help compute its complexity and mutate it efficiently.
    type Cache: Clone;
    /// Contains information about what mutations have already been tried.
    type MutationStep: Clone;
    /// Contains information about what arbitrary values have already been generated.
    type ArbitraryStep: Clone;
    /// Describes how to reverse a mutation
    type UnmutateToken;

    /// The first [`ArbitraryStep`](Mutator::ArbitraryStep) value to be passed to [`ordered_arbitrary`](crate::Mutator::ordered_arbitrary)
    fn default_arbitrary_step(&self) -> Self::ArbitraryStep;

    /// Verifies that the value conforms to the mutator’s expectations and, if it does,
    /// returns the [`Cache`](Mutator::Cache) with that value.
    fn validate_value(&self, value: &Value) -> Option<Self::Cache>;

    /// Returns the first [`MutationStep`](Mutator::MutationStep) associated with the value
    /// and cache.
    fn default_mutation_step(&self, value: &Value, cache: &Self::Cache) -> Self::MutationStep;

    /// The maximum complexity that a value can possibly have.
    ///
    /// If the maximum complexity is 0, it means that the mutator
    /// can only produce one value.
    fn max_complexity(&self) -> f64;
    /// The minimum complexity that a value can possibly have.
    ///
    /// If the minimum complexity is 0, then the maximum complexity must
    /// also be 0. It means that the mutator can only produce one value.
    fn min_complexity(&self) -> f64;

    /// Computes the complexity of the value.
    ///
    /// The returned value must be greater or equal than 0.
    /// It is only allowed to return 0 if the mutator cannot produce
    /// any other value than the one given as argument.
    fn complexity(&self, value: &Value, cache: &Self::Cache) -> f64;

    /// Generates an entirely new value based on the given [`ArbitraryStep`](Mutator::ArbitraryStep).
    ///
    /// The generated value should be smaller than the given `max_cplx`.
    ///
    /// The return value is `None` if no more new value can be generated or if
    /// it is not possible to stay within the given complexity. Otherwise, it
    /// is the value itself and its complexity, which should be equal to
    /// [`self.complexity(value, cache)`](Mutator::complexity)
    fn ordered_arbitrary(&self, step: &mut Self::ArbitraryStep, max_cplx: f64) -> Option<(Value, f64)>;

    /// Generates an entirely new value.
    ///
    /// The generated value should be smaller than the given `max_cplx`.
    /// However, if that is not possible, then it should return a value of
    /// the lowest possible complexity.
    ///
    /// Returns the value itself and its complexity, which must be equal to
    /// [`self.complexity(value, cache)`](Mutator::complexity)
    fn random_arbitrary(&self, max_cplx: f64) -> (Value, f64);

    /// Mutates a value (and optionally its cache) based on the given
    /// [`MutationStep`](Mutator::MutationStep).
    ///
    /// The mutated value should be within the given
    /// `max_cplx`.
    ///
    /// Returns `None` if it no longer possible to mutate
    /// the value to a new state, or if it is not possible to keep it under
    /// `max_cplx`. Otherwise, return the [`UnmutateToken`](Mutator::UnmutateToken)
    /// that describes how to undo the mutation, as well as the new complexity of the value.
    fn ordered_mutate(
        &self,
        value: &mut Value,
        cache: &mut Self::Cache,
        step: &mut Self::MutationStep,
        max_cplx: f64,
    ) -> Option<(Self::UnmutateToken, f64)>;

    /// Mutates a value (and optionally its cache).
    ///
    /// The mutated value should be within the given `max_cplx`. But if that
    /// is not possible, then it should mutate the value so that it has a minimal complexity.
    ///
    /// Returns the [`UnmutateToken`](Mutator::UnmutateToken) that describes how to undo
    /// the mutation as well as the new complexity of the value.
    fn random_mutate(&self, value: &mut Value, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64);

    /// Undoes a mutation performed on the given value and cache, described by
    /// the given [`UnmutateToken`](Mutator::UnmutateToken).
    fn unmutate(&self, value: &mut Value, cache: &mut Self::Cache, t: Self::UnmutateToken);

    type RecursingPartIndex: Clone;

    fn default_recursing_part_index(&self, value: &Value, cache: &Self::Cache) -> Self::RecursingPartIndex;
    fn recursing_part<'a, T, M>(
        &self,
        parent: &M,
        value: &'a Value,
        index: &mut Self::RecursingPartIndex,
    ) -> Option<&'a T>
    where
        T: Clone + 'static,
        M: Mutator<T>;
}

/**
 * A [Serializer] is used to encode and decode values into bytes.
 *
 * One possible implementation would be to use `serde` to implement
 * both required functions. But we also want to be able to fuzz-test
 * types that are not serializable with `serde`, which is why this
 * trait exists.
*/
pub trait Serializer {
    /// The type of the valeu to be serialized
    type Value;

    /// The extension of the file containing the serialized value
    fn extension(&self) -> &str;

    /// Deserialize the bytes into the value.
    ///
    /// This method can fail by returning `None`
    fn from_data(&self, data: &[u8]) -> Option<Self::Value>;

    /// Serialize the value into bytes
    ///
    /// This method should never fail.
    fn to_data(&self, value: &Self::Value) -> Vec<u8>;
}

pub trait MutatorValueConverter {
    type InnerValue;
    type Value;

    fn from_inner_value_ref(&self, inner_value: &Self::InnerValue) -> Self::Value;

    fn from_inner_value(&self, inner_value: Self::InnerValue) -> Self::Value {
        self.from_inner_value_ref(&inner_value)
    }

    fn to_inner_value_ref(&self, value: &Self::Value) -> Self::InnerValue;

    fn to_inner_value(&self, value: Self::Value) -> Self::InnerValue {
        self.to_inner_value_ref(&value)
    }
}

pub struct ExtendedMutator<V, IV, IM, MVC> {
    inner: IM,
    value_converter: MVC,
    _phantom_v: PhantomData<V>,
    _phantom_iv: PhantomData<IV>,
}

impl<V, IV, IM, MVC> ExtendedMutator<V, IV, IM, MVC> {
    pub fn new(inner: IM, value_converter: MVC) -> Self {
        Self {
            inner,
            value_converter,
            _phantom_v: PhantomData,
            _phantom_iv: PhantomData,
        }
    }
}

impl<V, IV, IM, MVC> Mutator<V> for ExtendedMutator<V, IV, IM, MVC>
where
    V: 'static + Clone,
    IV: 'static + Clone,
    IM: Mutator<IV>,
    MVC: 'static + MutatorValueConverter<InnerValue = IV, Value = V>,
{
    type Cache = IM::Cache;
    type MutationStep = IM::MutationStep;
    type ArbitraryStep = IM::ArbitraryStep;
    type UnmutateToken = IM::UnmutateToken;
    type RecursingPartIndex = ();

    fn default_arbitrary_step(&self) -> Self::ArbitraryStep {
        self.inner.default_arbitrary_step()
    }

    fn validate_value(&self, value: &V) -> Option<(Self::Cache, Self::MutationStep)> {
        self.inner
            .validate_value(&self.value_converter.to_inner_value_ref(value))
    }

    fn max_complexity(&self) -> f64 {
        self.inner.max_complexity()
    }

    fn min_complexity(&self) -> f64 {
        self.inner.min_complexity()
    }

    fn complexity(&self, value: &V, cache: &Self::Cache) -> f64 {
        self.inner
            .complexity(&self.value_converter.to_inner_value_ref(value), cache)
    }

    fn ordered_arbitrary(&self, step: &mut Self::ArbitraryStep, max_cplx: f64) -> Option<(V, f64)> {
        self.inner
            .ordered_arbitrary(step, max_cplx)
            .map(|(inner_value, cplx)| (self.value_converter.from_inner_value(inner_value), cplx))
    }

    fn random_arbitrary(&self, max_cplx: f64) -> (V, f64) {
        let (inner_value, cplx) = self.inner.random_arbitrary(max_cplx);
        (self.value_converter.from_inner_value(inner_value), cplx)
    }

    fn ordered_mutate(
        &self,
        value: &mut V,
        cache: &mut Self::Cache,
        step: &mut Self::MutationStep,
        max_cplx: f64,
    ) -> Option<(Self::UnmutateToken, f64)> {
        let mut inner_value = self.value_converter.to_inner_value_ref(value);
        let res = self.inner.ordered_mutate(&mut inner_value, cache, step, max_cplx);
        *value = self.value_converter.from_inner_value(inner_value);
        res
    }

    fn random_mutate(&self, value: &mut V, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64) {
        let mut inner_value = self.value_converter.to_inner_value_ref(value);
        let res = self.inner.random_mutate(&mut inner_value, cache, max_cplx);
        *value = self.value_converter.from_inner_value(inner_value);
        res
    }

    fn unmutate(&self, value: &mut V, cache: &mut Self::Cache, t: Self::UnmutateToken) {
        let mut inner_value = self.value_converter.to_inner_value_ref(value);
        self.inner.unmutate(&mut inner_value, cache, t);
        *value = self.value_converter.from_inner_value(inner_value);
    }

    fn default_recursing_part_index(&self, _: &V, _: &Self::Cache) -> Self::RecursingPartIndex {}

    fn recursing_part<'a, T, M>(&self, _: &M, _: &'a V, _: &mut Self::RecursingPartIndex) -> Option<&'a T>
    where
        T: Clone + 'static,
        M: Mutator<T>,
    {
        None
    }
}

/**
 A trait for types that are basic wrappers over a mutator, such as `Box<M>`.

 Such wrapper types automatically implement the [`Mutator`](Mutator) trait.
*/
pub trait MutatorWrapper {
    type Wrapped;

    fn wrapped_mutator(&self) -> &Self::Wrapped;
}

impl<T: Clone, W, M> Mutator<T> for M
where
    M: MutatorWrapper<Wrapped = W>,
    W: Mutator<T>,
    Self: 'static,
{
    #[doc(hidden)]
    type Cache = W::Cache;
    #[doc(hidden)]
    type MutationStep = W::MutationStep;
    #[doc(hidden)]
    type ArbitraryStep = W::ArbitraryStep;
    #[doc(hidden)]
    type UnmutateToken = W::UnmutateToken;

    #[doc(hidden)]
    #[no_coverage]
    fn default_arbitrary_step(&self) -> Self::ArbitraryStep {
        self.wrapped_mutator().default_arbitrary_step()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn validate_value(&self, value: &T) -> Option<Self::Cache> {
        self.wrapped_mutator().validate_value(value)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn default_mutation_step(&self, value: &T, cache: &Self::Cache) -> Self::MutationStep {
        self.wrapped_mutator().default_mutation_step(value, cache)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn max_complexity(&self) -> f64 {
        self.wrapped_mutator().max_complexity()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn min_complexity(&self) -> f64 {
        self.wrapped_mutator().min_complexity()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn complexity(&self, value: &T, cache: &Self::Cache) -> f64 {
        self.wrapped_mutator().complexity(value, cache)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn ordered_arbitrary(&self, step: &mut Self::ArbitraryStep, max_cplx: f64) -> Option<(T, f64)> {
        self.wrapped_mutator().ordered_arbitrary(step, max_cplx)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn random_arbitrary(&self, max_cplx: f64) -> (T, f64) {
        self.wrapped_mutator().random_arbitrary(max_cplx)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn ordered_mutate(
        &self,
        value: &mut T,
        cache: &mut Self::Cache,
        step: &mut Self::MutationStep,
        max_cplx: f64,
    ) -> Option<(Self::UnmutateToken, f64)> {
        self.wrapped_mutator().ordered_mutate(value, cache, step, max_cplx)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn random_mutate(&self, value: &mut T, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64) {
        self.wrapped_mutator().random_mutate(value, cache, max_cplx)
    }

    #[doc(hidden)]
    type RecursingPartIndex = W::RecursingPartIndex;
    #[doc(hidden)]
    #[no_coverage]
    fn unmutate(&self, value: &mut T, cache: &mut Self::Cache, t: Self::UnmutateToken) {
        self.wrapped_mutator().unmutate(value, cache, t)
    }
    #[doc(hidden)]
    #[no_coverage]
    fn default_recursing_part_index(&self, value: &T, cache: &Self::Cache) -> Self::RecursingPartIndex {
        self.wrapped_mutator().default_recursing_part_index(value, cache)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn recursing_part<'a, V, N>(&self, parent: &N, value: &'a T, index: &mut Self::RecursingPartIndex) -> Option<&'a V>
    where
        V: Clone + 'static,
        N: Mutator<V>,
    {
        let m = self.wrapped_mutator();
        m.recursing_part::<V, N>(parent, value, index)
    }
}

impl<M> MutatorWrapper for Box<M> {
    type Wrapped = M;
    #[no_coverage]
    fn wrapped_mutator(&self) -> &Self::Wrapped {
        self.as_ref()
    }
}

impl<M> MutatorWrapper for Rc<M> {
    type Wrapped = M;
    #[no_coverage]
    fn wrapped_mutator(&self) -> &Self::Wrapped {
        self.as_ref()
    }
}

impl<M> MutatorWrapper for Arc<M> {
    type Wrapped = M;
    #[no_coverage]
    fn wrapped_mutator(&self) -> &Self::Wrapped {
        self.as_ref()
    }
}

/// A [CorpusDelta] describes how to reflect a change in the pool’s content to the corpus on the file system.
///
/// It is used as the return type to [`pool.process(..)`](CompatibleWithObservations::process) where a test case along
/// with its associated sensor observations is given to the pool. Thus, it is always implicitly associated with
/// a specific pool and test case.
#[derive(Debug)]
pub struct CorpusDelta {
    /// The common path to the subfolder inside the main corpus where the test cases (added or removed) reside
    pub path: PathBuf,
    /// Whether the test case was added to the pool
    pub add: bool,
    /// A list of test cases that were removed
    pub remove: Vec<PoolStorageIndex>,
}

impl CorpusDelta {
    #[no_coverage]
    pub fn fuzzer_event(deltas: &[CorpusDelta]) -> FuzzerEvent {
        let mut add = 0;
        let mut remove = 0;
        for delta in deltas {
            if delta.add {
                add += 1;
            }
            remove += delta.remove.len();
        }

        if add == 0 && remove == 0 {
            FuzzerEvent::None
        } else {
            FuzzerEvent::Replace(add, remove)
        }
    }
}

/**
A [Sensor] records information when running the test function, which the
fuzzer can use to determine the importance of a test case.

For example, the sensor can record the code coverage triggered by the test case,
store the source location of a panic, measure the number of allocations made, etc.
The observations made by a sensor are then assessed by a [Pool], which must be
explicitly [compatible](CompatibleWithObservations) with the sensor’s observations.
*/
pub trait Sensor: SaveToStatsFolder + 'static {
    type Observations;

    /// Signal to the sensor that it should prepare to record observations
    fn start_recording(&mut self);
    /// Signal to the sensor that it should stop recording observations
    fn stop_recording(&mut self);

    /// Access the sensor's observations
    fn get_observations(&mut self) -> Self::Observations;
}

pub trait Stats: Display + ToCSV + 'static {}

/// An object safe trait that combines the methods of the [`Sensor`], [`Pool`], and [`CompatibleWithObservations`] traits.
///
/// While it's often useful to work with the [`Sensor`] and [`Pool`] traits separately, the
/// fuzzer doesn't actually need to know about the sensor and pool individually. By having
/// this `SensorAndPool` trait, we can give the fuzzer a `Box<dyn SensorAndPool>` and get rid of
/// two generic type parameters: `S: Sensor` and `P: Pool + CompatibleWithObservations<S::Observations>`.
///
/// This is better for compile times and simplifies the implementation of the fuzzer. Users of
/// `fuzzcheck` should feel free to ignore this trait, as it is arguably more an implementation detail
/// than a fundamental building block of the fuzzer.
///
/// Currently, there are two types implementing `SensorAndPool`:
/// 1. `(S, P)` where `S: Sensor` and `P: Pool + CompatibleWithObservations<S::Observations>`
/// 2. [`AndSensorAndPool`](crate::sensors_and_pools::AndSensorAndPool)
pub trait SensorAndPool: SaveToStatsFolder {
    fn stats(&self) -> Box<dyn Stats>;
    fn start_recording(&mut self);
    fn stop_recording(&mut self);
    fn process(&mut self, input_id: PoolStorageIndex, cplx: f64) -> Vec<CorpusDelta>;
    fn get_random_index(&mut self) -> Option<PoolStorageIndex>;
}
impl<A, B> SaveToStatsFolder for (A, B)
where
    A: SaveToStatsFolder,
    B: SaveToStatsFolder,
{
    #[no_coverage]
    fn save_to_stats_folder(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut x = self.0.save_to_stats_folder();
        x.extend(self.1.save_to_stats_folder());
        x
    }
}
impl<S, P> SensorAndPool for (S, P)
where
    S: Sensor,
    P: CompatibleWithObservations<S::Observations>,
    S: SaveToStatsFolder,
    P: SaveToStatsFolder,
{
    #[no_coverage]
    fn stats(&self) -> Box<dyn Stats> {
        Box::new(self.1.stats())
    }
    #[no_coverage]
    fn start_recording(&mut self) {
        self.0.start_recording();
    }
    #[no_coverage]
    fn stop_recording(&mut self) {
        self.0.stop_recording();
    }
    #[no_coverage]
    fn process(&mut self, input_id: PoolStorageIndex, complexity: f64) -> Vec<CorpusDelta> {
        self.1.process(input_id, &self.0.get_observations(), complexity)
    }
    #[no_coverage]
    fn get_random_index(&mut self) -> Option<PoolStorageIndex> {
        self.1.get_random_index()
    }
}

pub enum CSVField {
    Integer(isize),
    Float(f64),
    String(String),
}
impl CSVField {
    #[no_coverage]
    pub fn to_bytes(fields: &[CSVField]) -> Vec<u8> {
        let mut bytes = vec![];
        for field in fields {
            match field {
                CSVField::Integer(n) => {
                    bytes.extend(format!("{}", n).as_bytes());
                }
                CSVField::Float(f) => {
                    bytes.extend(format!("{:.4}", f).as_bytes());
                }
                CSVField::String(s) => {
                    bytes.extend(format!("{:?}", s).as_bytes());
                }
            }
            bytes.extend(b",");
        }
        bytes.extend(b"\n");
        bytes
    }
}

/**
Describes how to save a list of this value as a CSV file.

It is done via two methods:
1. [self.csv_headers\()](ToCSV::csv_headers) gives the first row of the file, as a list of [CSVField].
For example, it can be `time, score`.
2. [self.to_csv_record\()](ToCSV::to_csv_record) serializes the value as a CSV row. For example, it
can be `16:07:32, 34.0`.

Note that each call to [self.to_csv_record\()](ToCSV::to_csv_record) must return a list of [CSVField]
where the field at index `i` corresponds to the header at index `i` given by [self.csv_headers()](ToCSV::csv_headers).
Otherwise, the CSV file will be invalid.
*/
pub trait ToCSV {
    /// The headers of the CSV file
    fn csv_headers(&self) -> Vec<CSVField>;
    /// Serializes `self` as a list of [CSVField]. Each element in the vector must correspond to a header given
    /// by [self.csv_headers\()](ToCSV::csv_headers)
    fn to_csv_record(&self) -> Vec<CSVField>;
}
impl ToCSV for Box<dyn Stats> {
    #[no_coverage]
    fn csv_headers(&self) -> Vec<CSVField> {
        self.as_ref().csv_headers()
    }

    #[no_coverage]
    fn to_csv_record(&self) -> Vec<CSVField> {
        self.as_ref().to_csv_record()
    }
}
impl Stats for Box<dyn Stats> {}
/**
A [`Pool`] ranks test cases based on observations recorded by a sensor.

The pool trait is divided into two parts:
1. [`Pool`] contains general methods that are independent of the sensor used
2. [`CompatibleWithObservations<O>`] is a subtrait of [`Pool`]. It describes how the pool handles
observations made by the [`Sensor`].
*/
pub trait Pool: SaveToStatsFolder {
    /// Statistics about the pool to be printed to the terminal as the fuzzer is running and
    /// saved to a .csv file after the run
    type Stats: Stats;

    /// The pool’s statistics
    fn stats(&self) -> Self::Stats;

    /// Get the index of a random test case.
    ///
    /// Most [Pool] implementations will want to prioritise certain test cases
    /// over others based on their associated observations.
    fn get_random_index(&mut self) -> Option<PoolStorageIndex>;

    /// Gives the relative importance of the pool. It must be a positive number.
    ///
    /// The value is 1.0 by default.
    fn weight(&self) -> f64 {
        1.0
    }
}

pub trait PoolExt: Pool + Sized {
    fn and<P, SM>(self, p: P, override_weight: Option<f64>, _sensor_marker: SM) -> AndPool<Self, P, SM>
    where
        P: Pool,
    {
        let self_weight = self.weight();
        let p_weight = p.weight();
        AndPool::<_, _, SM>::new(self, p, self_weight, override_weight.unwrap_or(p_weight))
    }
}

impl<P> PoolExt for P where P: Pool {}

/**
A subtrait of [Pool] describing how the pool handles observations made by a sensor.

This trait is separate from [Pool] because a single pool type may handle multiple different kinds of sensors.

It is responsible for judging whether the observations are interesting, and then adding the test case to the pool
if they are. It communicates to the rest of the fuzzer what test cases were added or removed from the pool via the
[`CorpusDelta`] type. This ensures that the right message can be printed to the terminal and that the corpus on the
file system, which reflects the content of the pool, can be properly updated.
*/
pub trait CompatibleWithObservations<O>: Pool {
    fn process(&mut self, input_id: PoolStorageIndex, observations: &O, complexity: f64) -> Vec<CorpusDelta>;
}

/// A trait for types that want to save their content to the `stats` folder which is created after a fuzzing run.
pub trait SaveToStatsFolder {
    /// Save information about `self` to the stats folder
    ///
    /// Return a vector of tuples `(path_to_file, serialised_content)` representing a list of files to create under
    /// the `stats_folder`. The first element of each tuple is the path of the new created file. If this path is relative,
    /// it is relative to the `stats` folder path. The second element is the content of the file, as bytes.
    fn save_to_stats_folder(&self) -> Vec<(PathBuf, Vec<u8>)>;
}

/// A trait for convenience methods automatically implemented
/// for all types that conform to [`Sensor`].
pub trait SensorExt: Sensor {
    /// Maps the observations of the sensor using the given closure.
    ///
    /// For example, if a sensor has observations of type `Vec<u64>`, then we
    /// can create a sensor with observations `(Vec<u64>, u64)`, where the
    /// second element of the tuple is the sum of all observations:
    /// ```
    /// use fuzzcheck::SensorExt;
    /// # use fuzzcheck::sensors_and_pools::ArrayOfCounters;
    /// # static mut COUNTERS: [u64; 2] = [0; 2];
    /// # // inside the fuzz test, you can create the sensor as follows
    /// # let sensor = ArrayOfCounters::new(unsafe { &mut COUNTERS });
    /// let sensor = sensor.map(|observations| {
    ///    let sum = observations.iter().sum::<u64>();
    ///    (observations, sum)
    /// });
    /// ```
    #[no_coverage]
    fn map<ToObservations, F>(self, map_f: F) -> MapSensor<Self, ToObservations, F>
    where
        Self: Sized,
        F: Fn(Self::Observations) -> ToObservations,
    {
        MapSensor::new(self, map_f)
    }
}
impl<T> SensorExt for T where T: Sensor {}
