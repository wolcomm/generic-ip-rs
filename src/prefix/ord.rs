use core::cmp::Ordering::{self, Equal, Greater, Less};

use super::ConcretePrefix;
use crate::{addr::ConcreteAddress, af::Afi};

/// Ordering relationship between a pair of [`Prefix<A>`] `P` and `Q`.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum PrefixOrdering<A: Afi> {
    /// `Q` is equal to `P`.
    Equal,
    /// `Q` is a subprefix of `P`.
    Subprefix(ConcretePrefix<A>),
    /// `Q` is a superprefix of `P`.
    Superprefix(ConcretePrefix<A>),
    /// Neither `P` nor `Q` is a subprefix of the other.
    Divergent(ConcretePrefix<A>),
}

impl<A: Afi> ConcretePrefix<A> {
    /// Perform ordinal comparison with another [`Prefix<A>`], calculating the
    /// longest common prefix in the process.
    pub fn compare(self, other: Self) -> PrefixOrdering<A> {
        let common = self.common_with(other);
        match (
            self.length().cmp(&common.length()),
            other.length().cmp(&common.length()),
        ) {
            (Equal, Equal) => PrefixOrdering::Equal,
            (Equal, Greater) => PrefixOrdering::Subprefix(common),
            (Greater, Equal) => PrefixOrdering::Superprefix(common),
            (Greater, Greater) => PrefixOrdering::Divergent(common),
            (Less, _) | (_, Less) => {
                unreachable!("common must be shorter than both prefixes")
            }
        }
    }
}

impl<A: Afi> PartialOrd<Self> for ConcretePrefix<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.compare(*other) {
            PrefixOrdering::Equal => Some(Equal),
            PrefixOrdering::Subprefix(_) => Some(Less),
            PrefixOrdering::Superprefix(_) => Some(Greater),
            PrefixOrdering::Divergent(_) => None,
        }
    }
}

impl<A: Afi> PartialEq<ConcreteAddress<A>> for ConcretePrefix<A> {
    fn eq(&self, other: &ConcreteAddress<A>) -> bool {
        self.eq(&ConcretePrefix::from(*other))
    }
}

impl<A: Afi> PartialOrd<ConcreteAddress<A>> for ConcretePrefix<A> {
    fn partial_cmp(&self, other: &ConcreteAddress<A>) -> Option<Ordering> {
        self.partial_cmp(&ConcretePrefix::from(*other))
    }
}

#[cfg(test)]
mod tests {
    // TODO: check Eq invariants for Prefix<A>
}
