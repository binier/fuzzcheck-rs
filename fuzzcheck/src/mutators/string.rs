use crate as fuzzcheck;
use crate::{DefaultMutator, ExtendedMutator, MutatorValueConverter, MutatorWrapper};

use super::enums::BasicEnumMutator;
use super::unit::UnitMutator;

impl DefaultMutator for String {
    type Mutator = UnitMutator<String>;

    fn default_mutator() -> Self::Mutator {
        UnitMutator::new(String::new())
    }
}

#[derive(DefaultMutator, Clone)]
pub enum ChainName {
    Mainnet,
}

pub struct ChainNameToStringConverter {}

impl ChainNameToStringConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl MutatorValueConverter for ChainNameToStringConverter {
    type InnerValue = ChainName;
    type Value = String;

    fn from_inner_value_ref(&self, inner_value: &Self::InnerValue) -> Self::Value {
        match inner_value {
            ChainName::Mainnet => "TEZOS_MAINNET".to_owned(),
        }
    }

    fn to_inner_value_ref(&self, value: &Self::Value) -> Self::InnerValue {
        match value.as_str() {
            "TEZOS_MAINNET" => ChainName::Mainnet,
            _ => unreachable!(),
        }
    }
}

pub type ChainNameStringMutatorInner = ExtendedMutator<String, ChainName, BasicEnumMutator, ChainNameToStringConverter>;

pub struct ChainNameStringMutator {
    inner: ChainNameStringMutatorInner,
}

impl ChainNameStringMutator {
    pub fn new() -> Self {
        Self {
            inner: ChainNameStringMutatorInner::new(
                BasicEnumMutator::new::<ChainName>(),
                ChainNameToStringConverter::new(),
            ),
        }
    }
}

impl MutatorWrapper for ChainNameStringMutator {
    type Wrapped = ChainNameStringMutatorInner;

    fn wrapped_mutator(&self) -> &Self::Wrapped {
        &self.inner
    }
}
