use core::ops::{BitAnd, BitAndAssign, BitXor};

use crate::{af::Afi, prefix::PrefixLength, primitive::AddressPrimitive};

mod mask;

use self::mask::Mask;

pub use self::mask::{Hostmask, Netmask};

mod private {
    use super::*;

    /// An IP address.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Address<A: Afi>(A::Addr);

    impl<A: Afi> Address<A> {
        /// Construct a new [`Address<A>`] from an integer primitive
        /// appropriate to `A`.
        pub fn new(addr: A::Addr) -> Self {
            Self(addr)
        }

        /// Get the inner integer val, consuming `self`.
        pub fn into_primitive(self) -> A::Addr {
            self.0
        }
    }
}

pub use self::private::Address;

/// Compute the length, as a [`PrefixLength<A>`], for the common prefixes of
/// two [`Address<A>`].
pub fn common_length<A: Afi>(lhs: Address<A>, rhs: Address<A>) -> PrefixLength<A> {
    lhs.common_length(rhs)
}

impl<A: Afi> Address<A> {
    /// Compute the common length of `self` and another [`Address<A>`].
    pub fn common_length(self, other: Self) -> PrefixLength<A> {
        // ok to unwrap here as long as primitive width invariants hold
        PrefixLength::from_primitive((self ^ other).leading_zeros()).unwrap()
    }
}

impl<A: Afi, T: mask::Type> BitAnd<Mask<A, T>> for Address<A> {
    type Output = Self;

    fn bitand(self, mask: Mask<A, T>) -> Self::Output {
        Self::new(self.into_primitive().bitand(mask.into_primitive()))
    }
}

impl<A: Afi, T> BitAndAssign<T> for Address<A>
where
    Self: BitAnd<T, Output = Self>,
{
    fn bitand_assign(&mut self, rhs: T) {
        *self = self.bitand(rhs);
    }
}

impl<A: Afi> BitXor<Self> for Address<A> {
    type Output = A::Addr;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.into_primitive() ^ rhs.into_primitive()
    }
}

mod parse {
    use super::*;

    use core::str::FromStr;

    use crate::error::Error;

    impl<A: Afi> FromStr for Address<A> {
        type Err = Error<'static, A>;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            A::parse_addr(s).map(Self::new)
        }
    }
}

mod fmt {
    use super::*;

    use core::fmt;

    use crate::fmt::AddressDisplay;

    impl<A: Afi> fmt::Display for Address<A>
    where
        A::Addr: AddressDisplay<A>,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.into_primitive().fmt_addr(f)
        }
    }
}

#[cfg(feature = "std")]
mod convert {
    use super::*;

    use std::net::{Ipv4Addr, Ipv6Addr};

    use crate::af::{Ipv4, Ipv6};

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
        A::Addr: Arbitrary,
        StrategyFor<A::Addr>: 'static,
    {
        type Parameters = ParamsFor<A::Addr>;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            any_with::<A::Addr>(params).prop_map(Self::new).boxed()
        }
    }
}
