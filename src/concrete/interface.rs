use core::fmt;
use core::str::FromStr;

use super::{impl_try_from_any, Address, Prefix, PrefixLength};
use crate::{
    any,
    error::Error,
    fmt::AddressDisplay,
    traits::{self, primitive::Address as _, Afi, Prefix as _},
    Ipv4, Ipv6,
};

#[allow(clippy::wildcard_imports)]
mod private {
    use super::*;

    /// An IP interface, consisting of am address and prefix length.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub struct Interface<A: Afi> {
        address: Address<A>,
        length: PrefixLength<A>,
    }

    impl<A: Afi> Interface<A> {
        /// Construct a new [`Interface<A>`] from an address and prefix length.
        pub const fn new(address: Address<A>, length: PrefixLength<A>) -> Self {
            Self { address, length }
        }

        /// Get the address of this interface.
        pub const fn address(&self) -> Address<A> {
            self.address
        }

        /// Get the prefix length of this interface.
        pub const fn length(&self) -> PrefixLength<A> {
            self.length
        }
    }
}

pub use self::private::Interface;

impl<A: Afi> traits::Interface for Interface<A> {
    type Address = Address<A>;
    type Prefix = Prefix<A>;
    type PrefixLength = PrefixLength<A>;

    fn network(&self) -> Self::Address {
        self.trunc().network()
    }

    fn addr(&self) -> Self::Address {
        self.address()
    }

    fn trunc(&self) -> Self::Prefix {
        Self::Prefix::from(*self)
    }

    fn prefix_len(&self) -> Self::PrefixLength {
        self.length()
    }

    fn broadcast(&self) -> Self::Address {
        let prefix = self.trunc();
        prefix.network() | prefix.hostmask()
    }
}

impl<A: Afi> From<Address<A>> for Interface<A> {
    fn from(addr: Address<A>) -> Self {
        Self::new(addr, PrefixLength::MAX)
    }
}

impl<A: Afi> FromStr for Interface<A> {
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
    any::Interface {
        any::Interface::Ipv4 => Interface<Ipv4>,
        any::Interface::Ipv6 => Interface<Ipv6>,
    }
}

impl<A: Afi> fmt::Display for Interface<A>
where
    A::Primitive: AddressDisplay<A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.address(), self.length())
    }
}

#[cfg(feature = "ipnet")]
impl From<ipnet::Ipv4Net> for Interface<Ipv4> {
    fn from(net: ipnet::Ipv4Net) -> Self {
        let address = net.addr().into();
        let length = PrefixLength::from_primitive(net.prefix_len())
            .expect("we trusted `ipnet` to enforce length bounds");
        Self::new(address, length)
    }
}

#[cfg(feature = "ipnet")]
impl From<ipnet::Ipv6Net> for Interface<Ipv6> {
    fn from(net: ipnet::Ipv6Net) -> Self {
        let address = net.addr().into();
        let length = PrefixLength::from_primitive(net.prefix_len())
            .expect("we trusted `ipnet` to enforce length bounds");
        Self::new(address, length)
    }
}

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any_with, Arbitrary, ParamsFor, StrategyFor},
    strategy::{BoxedStrategy, Strategy},
};

#[cfg(any(test, feature = "arbitrary"))]
impl<A: Afi> Arbitrary for Interface<A>
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
            .prop_map(|(address, length)| Self::new(address, length))
            .boxed()
    }
}
