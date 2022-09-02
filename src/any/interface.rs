use core::fmt;
use core::str::FromStr;

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any, Arbitrary},
    prop_oneof,
    strategy::{BoxedStrategy, Strategy},
};

use super::{delegate, Address, Prefix, PrefixLength};
use crate::{
    concrete::{self, Ipv4, Ipv6},
    error::Error,
    traits,
};

/// Either an IPv4 or IPv6 interface.
///
/// # Memory Use
///
/// Rust enums are sized to accomodate their largest variant, with smaller
/// variants being padded to fill up any unused space.
///
/// As a result, users should avoid using this type in a context where only
/// [`Interface::Ipv4`] variants are expected.
///
/// # Examples
///
/// ``` rust
/// use ip::{
///     traits::{Address as _, Interface as _},
///     Any, Interface,
/// };
///
/// let interface = "192.0.2.0/24".parse::<Interface<Any>>()?;
///
/// assert!(interface.network().is_documentation());
/// # Ok::<(), ip::Error>(())
/// ```
#[allow(variant_size_differences)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Interface {
    /// IPv4 interface variant.
    Ipv4(concrete::Interface<Ipv4>),
    /// IPv6 interface variant.
    Ipv6(concrete::Interface<Ipv6>),
}

impl traits::Interface for Interface {
    type Address = Address;
    type Prefix = Prefix;
    type PrefixLength = PrefixLength;

    delegate! {
        fn network(&self) -> Self::Address;
        fn addr(&self) -> Self::Address;
        fn trunc(&self) -> Self::Prefix;
        fn prefix_len(&self) -> Self::PrefixLength;
        fn broadcast(&self) -> Self::Address;
    }
}

impl From<concrete::Interface<Ipv4>> for Interface {
    fn from(interface: concrete::Interface<Ipv4>) -> Self {
        Self::Ipv4(interface)
    }
}

impl From<concrete::Interface<Ipv6>> for Interface {
    fn from(interface: concrete::Interface<Ipv6>) -> Self {
        Self::Ipv6(interface)
    }
}

impl FromStr for Interface {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        concrete::Interface::<Ipv4>::from_str(s)
            .map(Self::from)
            .or_else(|_| concrete::Interface::<Ipv6>::from_str(s).map(Self::from))
    }
}

impl fmt::Display for Interface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4(interface) => interface.fmt(f),
            Self::Ipv6(interface) => interface.fmt(f),
        }
    }
}

#[cfg(any(test, feature = "arbitrary"))]
impl Arbitrary for Interface {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            any::<concrete::Interface<Ipv4>>().prop_map(Self::Ipv4),
            any::<concrete::Interface<Ipv6>>().prop_map(Self::Ipv6),
        ]
        .boxed()
    }
}
