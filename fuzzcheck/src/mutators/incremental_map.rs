use crate::mutators::either::Either;
use crate::mutators::recursive::{RecurToMutator, RecursiveMutator};
use crate::Mutator;
use std::marker::PhantomData;

/**
    A type that that can propagate updates to a value (From) to
    another value (To) based on a Token.

    The token should be the UnmutateToken of a mutator.
*/
#[doc(hidden)] // hide the documentation because it pollutes the documentation of all other types
pub trait IncrementalMapping<From: Clone, To, M: Mutator<From>> {
    /// The `from_value` was already mutated. The `token` encodes how to unmutate `from_value`. Based on `token` and `from_value`,
    /// this function updates the `to_value`.
    fn mutate_value_from_token(&mut self, from_value: &From, to_value: &mut To, token: &M::UnmutateToken);
    /// The `from_value` has not yet been unmutated. The `token` encodes how to unmutate `from_value`. Based on `token` alone,
    /// this function updates the `to_value`.
    fn unmutate_value_from_token(&mut self, to_value: &mut To, token: &M::UnmutateToken);
}

pub struct Cache<From, To, M, Map>
where
    From: Clone,
    To: Clone + std::convert::From<From>,
    M: Mutator<From>,
    Map: IncrementalMapping<From, To, M> + for<'a> std::convert::From<&'a From>,
{
    from_value: From,
    from_cache: M::Cache,
    map: Map,
    _phantom: PhantomData<(M, To, Map)>,
}

pub struct IncrementalMapMutator<From, To, M, Map, Parse>
where
    From: Clone,
    To: Clone + std::convert::From<From>,
    M: Mutator<From>,
    Map: IncrementalMapping<From, To, M> + for<'a> std::convert::From<&'a From>,
    Parse: Fn(&To) -> Option<From>,
{
    parse: Parse,
    mutator: M,
    _phantom: PhantomData<(From, To, Map)>,
}
impl<From, To, M, Map, Parse> IncrementalMapMutator<From, To, M, Map, Parse>
where
    From: Clone,
    To: Clone + std::convert::From<From>,
    M: Mutator<From>,
    Map: IncrementalMapping<From, To, M> + for<'a> std::convert::From<&'a From>,
    Parse: Fn(&To) -> Option<From>,
{
    #[no_coverage]
    pub fn new(parse: Parse, mutator: M) -> Self {
        Self {
            parse,
            mutator,
            _phantom: PhantomData,
        }
    }
}

impl<From, To, M, Map, Parse> Mutator<To> for IncrementalMapMutator<From, To, M, Map, Parse>
where
    From: Clone,
    To: Clone + std::convert::From<From>,
    M: Mutator<From>,
    Map: IncrementalMapping<From, To, M> + for<'a> std::convert::From<&'a From>,
    Parse: Fn(&To) -> Option<From>,
{
    type Cache = Cache<From, To, M, Map>;
    type MutationStep = M::MutationStep;
    type ArbitraryStep = M::ArbitraryStep;
    type UnmutateToken = M::UnmutateToken;

    #[no_coverage]
    fn default_arbitrary_step(&self) -> Self::ArbitraryStep {
        self.mutator.default_arbitrary_step()
    }

    #[no_coverage]
    fn validate_value(&self, value: &To) -> Option<(Self::Cache, Self::MutationStep)> {
        let from_value = (self.parse)(value)?;
        let (from_cache, mutation_step) = self.mutator.validate_value(&from_value).unwrap();
        let map = Map::from(&from_value);
        let cache = Cache {
            from_value,
            from_cache,
            map,
            _phantom: PhantomData,
        };
        Some((cache, mutation_step))
    }

    #[no_coverage]
    fn max_complexity(&self) -> f64 {
        self.mutator.max_complexity()
    }

    #[no_coverage]
    fn min_complexity(&self) -> f64 {
        self.mutator.min_complexity()
    }

    #[no_coverage]
    fn complexity(&self, _value: &To, cache: &Self::Cache) -> f64 {
        self.mutator.complexity(&cache.from_value, &cache.from_cache)
    }

    #[no_coverage]
    fn ordered_arbitrary(&self, step: &mut Self::ArbitraryStep, max_cplx: f64) -> Option<(To, f64)> {
        let (value, cplx) = self.mutator.ordered_arbitrary(step, max_cplx)?;
        let x = To::from(value);
        Some((x, cplx))
    }

    #[no_coverage]
    fn random_arbitrary(&self, max_cplx: f64) -> (To, f64) {
        let (value, cplx) = self.mutator.random_arbitrary(max_cplx);
        let x = To::from(value);
        (x, cplx)
    }

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
        cache.map.mutate_value_from_token(&cache.from_value, value, &token);
        Some((token, cplx))
    }

    #[no_coverage]
    fn random_mutate(&self, value: &mut To, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64) {
        let (token, cplx) = self
            .mutator
            .random_mutate(&mut cache.from_value, &mut cache.from_cache, max_cplx);
        cache.map.mutate_value_from_token(&cache.from_value, value, &token);
        (token, cplx)
    }

    #[no_coverage]
    fn unmutate(&self, value: &mut To, cache: &mut Self::Cache, t: Self::UnmutateToken) {
        cache.map.unmutate_value_from_token(value, &t);
        self.mutator.unmutate(&mut cache.from_value, &mut cache.from_cache, t);
    }
}

impl<From, To, M, Map> IncrementalMapping<From, To, RecursiveMutator<M>> for Map
where
    From: Clone,
    To: Clone,
    M: Mutator<From>,
    Self: IncrementalMapping<From, To, M>,
{
    #[no_coverage]
    fn mutate_value_from_token(
        &mut self,
        from_value: &From,
        to_value: &mut To,
        token: &<RecursiveMutator<M> as Mutator<From>>::UnmutateToken,
    ) {
        <Self as IncrementalMapping<From, To, M>>::mutate_value_from_token(self, from_value, to_value, token);
    }

    #[no_coverage]
    fn unmutate_value_from_token(
        &mut self,
        to_value: &mut To,
        token: &<RecursiveMutator<M> as Mutator<From>>::UnmutateToken,
    ) {
        <Self as IncrementalMapping<From, To, M>>::unmutate_value_from_token(self, to_value, token);
    }
}

impl<From, To, Map, M1, M2> IncrementalMapping<From, To, Either<M1, M2>> for Map
where
    From: Clone,
    To: Clone,
    M1: Mutator<From>,
    M2: Mutator<From>,
    Self: IncrementalMapping<From, To, M1> + IncrementalMapping<From, To, M2>,
{
    #[no_coverage]
    fn mutate_value_from_token(
        &mut self,
        from_value: &From,
        to_value: &mut To,
        token: &<Either<M1, M2> as Mutator<From>>::UnmutateToken,
    ) {
        match token {
            Either::Left(token) => {
                <Self as IncrementalMapping<From, To, M1>>::mutate_value_from_token(self, from_value, to_value, token);
            }
            Either::Right(token) => {
                <Self as IncrementalMapping<From, To, M2>>::mutate_value_from_token(self, from_value, to_value, token);
            }
        }
    }

    #[no_coverage]
    fn unmutate_value_from_token(
        &mut self,
        to_value: &mut To,
        token: &<Either<M1, M2> as Mutator<From>>::UnmutateToken,
    ) {
        match token {
            Either::Left(token) => {
                <Self as IncrementalMapping<From, To, M1>>::unmutate_value_from_token(self, to_value, token);
            }
            Either::Right(token) => {
                <Self as IncrementalMapping<From, To, M2>>::unmutate_value_from_token(self, to_value, token)
            }
        }
    }
}

impl<From, To, M, Map> IncrementalMapping<From, To, RecurToMutator<M>> for Map
where
    From: Clone,
    To: Clone,
    M: Mutator<From>,
    Self: IncrementalMapping<From, To, M>,
{
    #[no_coverage]
    fn mutate_value_from_token(
        &mut self,
        from_value: &From,
        to_value: &mut To,
        token: &<RecurToMutator<M> as Mutator<From>>::UnmutateToken,
    ) {
        <Self as IncrementalMapping<From, To, M>>::mutate_value_from_token(self, from_value, to_value, token);
    }

    #[no_coverage]
    fn unmutate_value_from_token(
        &mut self,
        to_value: &mut To,
        token: &<RecurToMutator<M> as Mutator<From>>::UnmutateToken,
    ) {
        <Self as IncrementalMapping<From, To, M>>::unmutate_value_from_token(self, to_value, token);
    }
}