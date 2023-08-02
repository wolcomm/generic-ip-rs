use core::cmp::Ordering::{self, Equal, Greater, Less};

use super::{Address, Prefix};
use crate::traits::Afi;

/// Ordering relationship between a pair of [`Prefix<A>`] `P` and `Q`.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum PrefixOrdering<A: Afi> {
    /// `Q` is equal to `P`.
    Equal,
    /// `Q` is a subprefix of `P`.
    Subprefix(Prefix<A>),
    /// `Q` is a superprefix of `P`.
    Superprefix(Prefix<A>),
    /// Neither `P` nor `Q` is a subprefix of the other.
    Divergent(Prefix<A>),
}

impl<A: Afi> Prefix<A> {
    /// Perform ordinal comparison with another [`Prefix<A>`], calculating the
    /// longest common prefix in the process.
    pub fn compare(&self, other: &Self) -> PrefixOrdering<A> {
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

impl<A: Afi> PartialOrd<Self> for Prefix<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.compare(other) {
            PrefixOrdering::Equal => Some(Equal),
            PrefixOrdering::Subprefix(_) => Some(Greater),
            PrefixOrdering::Superprefix(_) => Some(Less),
            PrefixOrdering::Divergent(_) => None,
        }
    }
}

impl<A: Afi> PartialEq<Address<A>> for Prefix<A> {
    fn eq(&self, other: &Address<A>) -> bool {
        self.eq(&Self::from(*other))
    }
}

impl<A: Afi> PartialOrd<Address<A>> for Prefix<A> {
    fn partial_cmp(&self, other: &Address<A>) -> Option<Ordering> {
        self.partial_cmp(&Self::from(*other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ipv4, Ipv6};

    mod ipv4 {
        use super::*;

        #[test]
        fn equal() {
            let x = "10.0.0.0/8".parse::<Prefix<Ipv4>>().unwrap();
            let y = "10.0.0.0/8".parse::<Prefix<Ipv4>>().unwrap();
            assert!(x == y);
        }

        #[test]
        fn lt() {
            let x = "10.0.0.0/16".parse::<Prefix<Ipv4>>().unwrap();
            let y = "10.0.0.0/8".parse::<Prefix<Ipv4>>().unwrap();
            assert!(x < y);
        }

        #[test]
        fn gt() {
            let x = "10.0.0.0/16".parse::<Prefix<Ipv4>>().unwrap();
            let y = "10.0.0.0/24".parse::<Prefix<Ipv4>>().unwrap();
            assert!(x > y);
        }

        #[test]
        fn divergent() {
            let x = "10.0.0.0/16".parse::<Prefix<Ipv4>>().unwrap();
            let y = "10.1.0.0/16".parse::<Prefix<Ipv4>>().unwrap();
            assert!(x.partial_cmp(&y).is_none());
        }
    }

    mod ipv6 {
        use super::*;

        #[test]
        fn equal() {
            let x = "2001:db8::/32".parse::<Prefix<Ipv6>>().unwrap();
            let y = "2001:db8::/32".parse::<Prefix<Ipv6>>().unwrap();
            assert!(x == y);
        }

        #[test]
        fn lt() {
            let x = "2001:db8::/48".parse::<Prefix<Ipv6>>().unwrap();
            let y = "2001:db8::/32".parse::<Prefix<Ipv6>>().unwrap();
            assert!(x < y);
        }

        #[test]
        fn gt() {
            let x = "2001:db8::/48".parse::<Prefix<Ipv6>>().unwrap();
            let y = "2001:db8::/64".parse::<Prefix<Ipv6>>().unwrap();
            assert!(x > y);
        }

        #[test]
        fn divergent() {
            let x = "2001:db8:f::/48".parse::<Prefix<Ipv6>>().unwrap();
            let y = "2001:db8:a::/48".parse::<Prefix<Ipv6>>().unwrap();
            assert!(x.partial_cmp(&y).is_none());
        }
    }
}
