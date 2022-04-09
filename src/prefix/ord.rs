use core::cmp::Ordering::{self, Equal, Greater, Less};

use crate::af::Afi;

use super::Prefix;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum PrefixOrdering<A: Afi> {
    Equal,
    Subprefix(Prefix<A>),
    Superprefix(Prefix<A>),
    Divergent(Prefix<A>),
}

impl<A: Afi> Prefix<A> {
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

impl<A: Afi> PartialOrd for Prefix<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.compare(*other) {
            PrefixOrdering::Equal => Some(Equal),
            PrefixOrdering::Subprefix(_) => Some(Less),
            PrefixOrdering::Superprefix(_) => Some(Greater),
            PrefixOrdering::Divergent(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: check Eq invariants for Prefix<A>
}
