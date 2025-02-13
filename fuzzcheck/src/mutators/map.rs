use std::marker::PhantomData;

use crate::Mutator;

pub struct MapMutator<From, To, M, Parse, Map, Complexity>
where
    From: Clone,
    To: Clone,
    M: Mutator<From>,
    Parse: Fn(&To) -> Option<From>,
    Map: Fn(&From) -> To,
    Complexity: Fn(&To, f64) -> f64,
{
    pub mutator: M,
    pub parse: Parse,
    pub map: Map,
    pub complexity: Complexity,
    _phantom: PhantomData<(From, To)>,
}
impl<From, To, M, Parse, Map, Complexity> MapMutator<From, To, M, Parse, Map, Complexity>
where
    From: Clone,
    To: Clone,
    M: Mutator<From>,
    Parse: Fn(&To) -> Option<From>,
    Map: Fn(&From) -> To,
    Complexity: Fn(&To, f64) -> f64,
{
    #[no_coverage]
    pub fn new(mutator: M, parse: Parse, map: Map, complexity: Complexity) -> Self {
        Self {
            mutator,
            parse,
            map,
            complexity,
            _phantom: PhantomData,
        }
    }
}

pub struct Cache<From, M>
where
    From: Clone,
    M: Mutator<From>,
{
    from_value: From,
    from_cache: M::Cache,
}
impl<From, M> Clone for Cache<From, M>
where
    From: Clone,
    M: Mutator<From>,
{
    fn clone(&self) -> Self {
        Self {
            from_value: self.from_value.clone(),
            from_cache: self.from_cache.clone(),
        }
    }
}

impl<From, To, M, Parse, Map, Complexity> Mutator<To> for MapMutator<From, To, M, Parse, Map, Complexity>
where
    From: Clone,
    To: Clone,
    M: Mutator<From>,
    Parse: Fn(&To) -> Option<From>,
    Map: Fn(&From) -> To,
    Complexity: Fn(&To, f64) -> f64,
    Self: 'static,
{
    #[doc(hidden)]
    type Cache = Cache<From, M>;
    #[doc(hidden)]
    type MutationStep = M::MutationStep;
    #[doc(hidden)]
    type ArbitraryStep = M::ArbitraryStep;
    #[doc(hidden)]
    type UnmutateToken = M::UnmutateToken;

    #[doc(hidden)]
    #[no_coverage]
    fn default_arbitrary_step(&self) -> Self::ArbitraryStep {
        self.mutator.default_arbitrary_step()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn validate_value(&self, to_value: &To) -> Option<Self::Cache> {
        let from_value = (self.parse)(to_value)?;
        let from_cache = self.mutator.validate_value(&from_value)?;
        Some(Cache { from_value, from_cache })
    }
    #[doc(hidden)]
    #[no_coverage]
    fn default_mutation_step(&self, _value: &To, cache: &Self::Cache) -> Self::MutationStep {
        self.mutator.default_mutation_step(&cache.from_value, &cache.from_cache)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn max_complexity(&self) -> f64 {
        self.mutator.max_complexity()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn min_complexity(&self) -> f64 {
        self.mutator.min_complexity()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn complexity(&self, value: &To, cache: &Self::Cache) -> f64 {
        let cplx = self.mutator.complexity(&cache.from_value, &cache.from_cache);
        (self.complexity)(value, cplx)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn ordered_arbitrary(&self, step: &mut Self::ArbitraryStep, max_cplx: f64) -> Option<(To, f64)> {
        let (from_value, cplx) = self.mutator.ordered_arbitrary(step, max_cplx)?;
        let to_value = (self.map)(&from_value);
        let cplx = (self.complexity)(&to_value, cplx);
        Some((to_value, cplx))
    }

    #[doc(hidden)]
    #[no_coverage]
    fn random_arbitrary(&self, max_cplx: f64) -> (To, f64) {
        let (from_value, cplx) = self.mutator.random_arbitrary(max_cplx);
        let to_value = (self.map)(&from_value);
        let cplx = (self.complexity)(&to_value, cplx);
        (to_value, cplx)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn ordered_mutate(
        &self,
        value: &mut To,
        cache: &mut Self::Cache,
        step: &mut Self::MutationStep,
        max_cplx: f64,
    ) -> Option<(Self::UnmutateToken, f64)> {
        let (token, cplx) =
            self.mutator
                .ordered_mutate(&mut cache.from_value, &mut cache.from_cache, step, max_cplx)?;
        *value = (self.map)(&cache.from_value);
        let cplx = (self.complexity)(value, cplx);
        Some((token, cplx))
    }

    #[doc(hidden)]
    #[no_coverage]
    fn random_mutate(&self, value: &mut To, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64) {
        let (token, cplx) = self
            .mutator
            .random_mutate(&mut cache.from_value, &mut cache.from_cache, max_cplx);
        *value = (self.map)(&cache.from_value);
        let cplx = (self.complexity)(value, cplx);
        (token, cplx)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn unmutate(&self, value: &mut To, cache: &mut Self::Cache, t: Self::UnmutateToken) {
        self.mutator.unmutate(&mut cache.from_value, &mut cache.from_cache, t);
        *value = (self.map)(&cache.from_value);
    }

    // TODO: Not yet supported for MapMutator
    // it would require `recursing_part` to take the `cache` value as argument as well
    // maybe I should do that, but I haven't thought about it yet
    type RecursingPartIndex = ();

    fn default_recursing_part_index(&self, _value: &To, _cache: &Self::Cache) -> Self::RecursingPartIndex {}

    fn recursing_part<'a, V, N>(
        &self,
        _parent: &N,
        _value: &'a To,
        _index: &mut Self::RecursingPartIndex,
    ) -> Option<&'a V>
    where
        V: Clone + 'static,
        N: Mutator<V>,
    {
        None
    }
}
