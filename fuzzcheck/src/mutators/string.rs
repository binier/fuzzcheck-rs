use crate::DefaultMutator;

use super::unit::UnitMutator;

impl DefaultMutator for String {
    type Mutator = UnitMutator<String>;

    fn default_mutator() -> Self::Mutator {
        UnitMutator::new(String::new())
    }
}
