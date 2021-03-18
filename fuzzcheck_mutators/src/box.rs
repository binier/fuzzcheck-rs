use std::marker::PhantomData;

use fuzzcheck_traits::Mutator;

use crate::DefaultMutator;

pub struct BoxMutator<T: Clone, M: Mutator<T>> {
    pub mutator: M,
    _phantom: PhantomData<T>,
}
impl<T: Clone, M: Mutator<T>> BoxMutator<T, M> {
    pub fn new(mutator: M) -> Self {
        Self {
            mutator,
            _phantom: PhantomData,
        }
    }
}
impl<T: Clone, M: Mutator<T>> Default for BoxMutator<T, M>
where
    M: Default,
{
    fn default() -> Self {
        Self::new(<_>::default())
    }
}

impl<T: Clone, M: Mutator<T>> Mutator<Box<T>> for BoxMutator<T, M> {
    type Cache = M::Cache;
    type MutationStep = M::MutationStep;
    type ArbitraryStep = M::ArbitraryStep;
    type UnmutateToken = M::UnmutateToken;

    fn default_arbitrary_step(&self) -> Self::ArbitraryStep {
        self.mutator.default_arbitrary_step()
    }

    fn validate_value(&self, value: &Box<T>) -> Option<(Self::Cache, Self::MutationStep)> {
        self.mutator.validate_value(value)
    }

    fn max_complexity(&self) -> f64 {
        self.mutator.max_complexity()
    }

    fn min_complexity(&self) -> f64 {
        self.mutator.min_complexity()
    }

    fn complexity(&self, value: &Box<T>, cache: &Self::Cache) -> f64 {
        self.mutator.complexity(value, cache)
    }

    fn ordered_arbitrary(
        &self,
        step: &mut Self::ArbitraryStep,
        max_cplx: f64,
    ) -> Option<(Box<T>, Self::Cache, Self::MutationStep)> {
        if let Some((value, cache, step)) = self.mutator.ordered_arbitrary(step, max_cplx) {
            Some((Box::new(value), cache, step))
        } else {
            None
        }
    }

    fn random_arbitrary(&self, max_cplx: f64) -> (Box<T>, Self::Cache, Self::MutationStep) {
        let (value, cache, step) = self.mutator.random_arbitrary(max_cplx);
        (Box::new(value), cache, step)
    }

    fn ordered_mutate(
        &self,
        value: &mut Box<T>,
        cache: &mut Self::Cache,
        step: &mut Self::MutationStep,
        max_cplx: f64,
    ) -> Option<Self::UnmutateToken> {
        self.mutator.ordered_mutate(value, cache, step, max_cplx)
    }

    fn random_mutate(&self, value: &mut Box<T>, cache: &mut Self::Cache, max_cplx: f64) -> Self::UnmutateToken {
        self.mutator.random_mutate(value, cache, max_cplx)
    }

    fn unmutate(&self, value: &mut Box<T>, cache: &mut Self::Cache, t: Self::UnmutateToken) {
        self.mutator.unmutate(value, cache, t)
    }
}

impl<T> DefaultMutator for Box<T>
where
    T: DefaultMutator,
{
    type Mutator = BoxMutator<T, <T as DefaultMutator>::Mutator>;
    fn default_mutator() -> Self::Mutator {
        Self::Mutator::new(T::default_mutator())
    }
}
