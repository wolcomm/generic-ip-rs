#![allow(clippy::module_name_repetitions)]

use core::cmp::min;
use core::fmt;
use core::str::FromStr;

use super::{common_length, impl_try_from_any, Address, Bitmask, Hostmask, Interface, Netmask};
use crate::{
    any,
    error::Error,
    fmt::AddressDisplay,
    traits::{self, primitive::Address as _, Afi, PrefixLength as _},
    Ipv4, Ipv6,
};

mod len;
pub use self::len::PrefixLength;

mod ord;
pub use self::ord::PrefixOrdering;

mod range;
pub use self::range::Range;

#[cfg(feature = "std")]
// TODO: remove `unknown_lints` dance when `clippy::unnecessary_box_returns` is stabilised
#[allow(unknown_lints)]
#[allow(clippy::unnecessary_box_returns)]
#[warn(unknown_lints)]
mod set;
#[cfg(feature = "std")]
pub use self::set::Set;

mod subprefixes;
pub use self::subprefixes::Subprefixes;

#[allow(clippy::wildcard_imports)]
mod private {
    use super::*;

    /// An IP prefix, consisting of a network address and prefix length.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub struct Prefix<A: Afi> {
        prefix: Address<A>,
        length: PrefixLength<A>,
    }

    impl<A: Afi> Prefix<A> {
        /// The "default" prefix containing all addresses of address family `A`.
        pub const DEFAULT: Self = Self {
            prefix: Address::ZEROS,
            length: PrefixLength::MIN,
        };

        /// Construct a new [`Prefix<A>`] from an address and prefix length.
        ///
        /// The host bits of `prefix` will be automatically set to zero.
        pub fn new(mut prefix: Address<A>, length: PrefixLength<A>) -> Self {
            prefix &= Netmask::from(length);
            Self { prefix, length }
        }

        /// Get the network address of this prefix.
        pub const fn prefix(&self) -> Address<A> {
            self.prefix
        }

        /// Get the length of this prefix.
        pub const fn length(&self) -> PrefixLength<A> {
            self.length
        }
    }
}

pub use self::private::Prefix;

impl<A: Afi> Prefix<A> {
    fn common_with(&self, other: &Self) -> Self {
        let min_length = min(self.length(), other.length());
        let common_length = common_length(self.prefix(), other.prefix());
        let length = min(min_length, common_length);
        Self::new(self.prefix(), length)
    }

    fn map_addr<F>(&self, f: F) -> Option<Self>
    where
        F: FnOnce(Address<A>) -> Option<Address<A>>,
    {
        f(self.prefix()).map(|addr| Self::new(addr, self.length()))
    }
}

impl<A: Afi> traits::Prefix for Prefix<A> {
    type Address = Address<A>;
    type Length = PrefixLength<A>;
    type Hostmask = Hostmask<A>;
    type Netmask = Netmask<A>;
    type Subprefixes = Subprefixes<A>;

    fn network(&self) -> Self::Address {
        self.prefix()
    }

    fn hostmask(&self) -> Self::Hostmask {
        self.length().into()
    }

    fn netmask(&self) -> Self::Netmask {
        self.length().into()
    }

    fn max_prefix_len(&self) -> Self::Length {
        Self::Length::MAX
    }

    fn prefix_len(&self) -> Self::Length {
        self.length()
    }

    fn broadcast(&self) -> Self::Address {
        self.network() | self.hostmask()
    }

    fn supernet(&self) -> Option<Self> {
        self.length()
            .decrement()
            .map(|len| Self::new(self.prefix(), len))
            .ok()
    }

    fn is_sibling(&self, other: &Self) -> bool {
        self.supernet() == other.supernet()
    }

    fn subprefixes(&self, new_prefix_len: Self::Length) -> Result<Self::Subprefixes, Error> {
        Self::Subprefixes::new(*self, new_prefix_len)
    }

    fn new_prefix_length(&self, length: u8) -> Result<Self::Length, Error> {
        (length as usize).try_into()
    }
}

impl<A: Afi> From<Address<A>> for Prefix<A> {
    fn from(addr: Address<A>) -> Self {
        Self::new(addr, PrefixLength::MAX)
    }
}

impl<A: Afi> From<Interface<A>> for Prefix<A> {
    fn from(interface: Interface<A>) -> Self {
        Self::new(interface.address(), interface.length())
    }
}

impl<A: Afi> FromStr for Prefix<A> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        A::Primitive::parse_prefix(s).and_then(|(addr, len)| {
            Ok(Self::new(
                Address::new(addr),
                PrefixLength::from_primitive(len)?,
            ))
        })
    }
}

impl_try_from_any! {
    any::Prefix {
        any::Prefix::Ipv4 => Prefix<Ipv4>,
        any::Prefix::Ipv6 => Prefix<Ipv6>,
    }
}

impl<A: Afi> fmt::Display for Prefix<A>
where
    A::Primitive: AddressDisplay<A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.prefix(), self.length())
    }
}

#[cfg(feature = "ipnet")]
impl From<ipnet::Ipv4Net> for Prefix<Ipv4> {
    fn from(net: ipnet::Ipv4Net) -> Self {
        let prefix = net.network().into();
        let length = PrefixLength::from_primitive(net.prefix_len())
            .expect("we trusted `ipnet` to enforce length bounds");
        Self::new(prefix, length)
    }
}

#[cfg(feature = "ipnet")]
impl From<ipnet::Ipv6Net> for Prefix<Ipv6> {
    fn from(net: ipnet::Ipv6Net) -> Self {
        let prefix = net.network().into();
        let length = PrefixLength::from_primitive(net.prefix_len())
            .expect("we trusted `ipnet` to enforce length bounds");
        Self::new(prefix, length)
    }
}

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any_with, Arbitrary, ParamsFor, StrategyFor},
    strategy::{BoxedStrategy, Strategy},
};

#[cfg(any(test, feature = "arbitrary"))]
impl<A: Afi> Arbitrary for Prefix<A>
where
    Address<A>: Arbitrary,
    StrategyFor<Address<A>>: 'static,
    PrefixLength<A>: Arbitrary,
    StrategyFor<PrefixLength<A>>: 'static,
{
    type Parameters = ParamsFor<(Address<A>, PrefixLength<A>)>;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
        (
            any_with::<Address<A>>(params.0),
            any_with::<PrefixLength<A>>(params.1),
        )
            .prop_map(|(prefix, length)| Self::new(prefix, length))
            .boxed()
    }
}
