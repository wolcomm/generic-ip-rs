use core::fmt;
use core::str::FromStr;

use crate::{
    error::Error,
    fmt::AddressDisplay,
    traits::{self, primitive::Address as _, Afi},
};

use super::{AddressRange, PrefixLength};

mod private;
pub use self::private::Address;

mod ops;

mod ipv4;
mod ipv6;

impl<A: Afi> Address<A> {
    pub fn octets(&self) -> A::Octets {
        self.into_primitive().to_be_bytes()
    }

    /// Compute the common length of `self` and another [`Address<A>`].
    pub fn common_length(self, other: Self) -> PrefixLength<A> {
        // ok to unwrap here as long as primitive width invariants hold
        PrefixLength::<A>::from_primitive((self ^ other).leading_zeros()).unwrap()
    }
}
/// Compute the length, as a [`PrefixLength<A>`], for the common prefixes of
/// two [`Address<A>`].
pub fn common_length<A: Afi>(lhs: Address<A>, rhs: Address<A>) -> PrefixLength<A> {
    lhs.common_length(rhs)
}

impl<A: Afi> traits::Address for Address<A> {
    fn is_broadcast(&self) -> bool {
        if let Some(broadcast) = A::Primitive::BROADCAST {
            self.into_primitive() == broadcast
        } else {
            false
        }
    }
    fn is_link_local(&self) -> bool {
        AddressRange::from(&A::Primitive::LINK_LOCAL_RANGE).contains(self)
    }

    fn is_private(&self) -> bool {
        if let Some(ranges) = A::Primitive::PRIVATE_RANGES {
            ranges
                .iter()
                .any(|range| AddressRange::from(range).contains(self))
        } else {
            false
        }
    }

    fn is_reserved(&self) -> bool {
        if let Some(range) = A::Primitive::RESERVED_RANGE {
            AddressRange::from(&range).contains(self)
        } else {
            false
        }
    }

    fn is_shared(&self) -> bool {
        if let Some(range) = A::Primitive::SHARED_RANGE {
            AddressRange::from(&range).contains(self)
        } else {
            false
        }
    }

    fn is_thisnet(&self) -> bool {
        if let Some(range) = A::Primitive::THISNET_RANGE {
            AddressRange::from(&range).contains(self)
        } else {
            false
        }
    }

    fn is_benchmarking(&self) -> bool {
        AddressRange::from(&A::Primitive::BENCHMARK_RANGE).contains(self)
    }

    fn is_documentation(&self) -> bool {
        A::Primitive::DOCUMENTATION_RANGES
            .iter()
            .any(|range| AddressRange::from(range).contains(self))
    }

    fn is_global(&self) -> bool {
        self.into_primitive().is_global()
    }

    fn is_loopback(&self) -> bool {
        AddressRange::from(&A::Primitive::LOCALHOST_RANGE).contains(self)
    }

    fn is_multicast(&self) -> bool {
        AddressRange::from(&A::Primitive::MULTICAST_RANGE).contains(self)
    }

    fn is_unspecified(&self) -> bool {
        self == &Self::UNSPECIFIED
    }

    fn is_unique_local(&self) -> bool {
        if let Some(range) = A::Primitive::ULA_RANGE {
            AddressRange::from(&range).contains(self)
        } else {
            false
        }
    }
}

impl<A: Afi> FromStr for Address<A> {
    type Err = Error<'static, A>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        A::Primitive::parse_addr(s).map(Self::new)
    }
}

impl<A: Afi> fmt::Display for Address<A>
where
    A::Primitive: AddressDisplay<A>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.into_primitive().fmt_addr(f)
    }
}

#[cfg(feature = "std")]
mod convert {
    use super::*;

    use std::net::{Ipv4Addr, Ipv6Addr};

    use crate::concrete::{Ipv4, Ipv6};

    impl From<Ipv4Addr> for Address<Ipv4> {
        fn from(addr: Ipv4Addr) -> Self {
            Self::new(addr.into())
        }
    }

    impl From<Ipv6Addr> for Address<Ipv6> {
        fn from(addr: Ipv6Addr) -> Self {
            Self::new(addr.into())
        }
    }
}

#[cfg(any(test, feature = "arbitrary"))]
mod arbitrary {
    use super::*;

    use proptest::{
        arbitrary::{any_with, Arbitrary, ParamsFor, StrategyFor},
        strategy::{BoxedStrategy, Strategy},
    };

    impl<A: Afi> Arbitrary for Address<A>
    where
        A: 'static,
        A::Primitive: Arbitrary + 'static,
        StrategyFor<A::Primitive>: 'static,
    {
        type Parameters = ParamsFor<A::Primitive>;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            any_with::<A::Primitive>(params).prop_map(Self::new).boxed()
        }
    }
}
