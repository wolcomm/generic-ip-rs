use core::fmt;
use core::str::FromStr;

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any_with, Arbitrary, ParamsFor},
    prop_oneof,
    strategy::{BoxedStrategy, Strategy},
};

use super::{delegate, Address, Hostmask, Netmask};
use crate::{
    concrete::{self, Ipv4, Ipv6},
    error::{err, Error, Kind},
    traits,
};

mod len;
pub use self::len::Length;

mod range;
pub use self::range::Range;

#[cfg(feature = "std")]
mod set;
#[cfg(feature = "std")]
pub use self::set::Set;

mod subprefixes;
pub use self::subprefixes::Subprefixes;

/// Either an IPv4 or IPv6 prefix.
///
/// # Memory Use
///
/// Rust enums are sized to accommodate their largest variant, with smaller
/// variants being padded to fill up any unused space.
///
/// As a result, users should avoid using this type in a context where only
/// [`Prefix::Ipv4`] variants are expected.
///
/// # Examples
///
/// ``` rust
/// use ip::{
///     traits::{Address as _, Prefix as _},
///     Any, Prefix,
/// };
///
/// let prefix = "192.0.2.0/24".parse::<Prefix<Any>>()?;
///
/// assert!(prefix.network().is_documentation());
/// # Ok::<(), ip::Error>(())
/// ```
#[allow(variant_size_differences)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub enum Prefix {
    /// IPv4 prefix variant.
    Ipv4(concrete::Prefix<Ipv4>),
    /// IPv6 prefix variant.
    Ipv6(concrete::Prefix<Ipv6>),
}

impl traits::Prefix for Prefix {
    type Address = Address;
    type Length = Length;
    type Hostmask = Hostmask;
    type Netmask = Netmask;
    type Subprefixes = Subprefixes;

    delegate! {
        fn network(&self) -> Self::Address;
        fn hostmask(&self) -> Self::Hostmask;
        fn netmask(&self) -> Self::Netmask;
        fn max_prefix_len(&self) -> Self::Length;
        fn prefix_len(&self) -> Self::Length;
        fn broadcast(&self) -> Self::Address;
    }

    fn supernet(&self) -> Option<Self> {
        match self {
            Self::Ipv4(prefix) => prefix.supernet().map(Self::Ipv4),
            Self::Ipv6(prefix) => prefix.supernet().map(Self::Ipv6),
        }
    }

    fn is_sibling(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ipv4(prefix), Self::Ipv4(other)) => prefix.is_sibling(other),
            (Self::Ipv6(prefix), Self::Ipv6(other)) => prefix.is_sibling(other),
            _ => false,
        }
    }

    fn subprefixes(&self, new_prefix_length: Self::Length) -> Result<Self::Subprefixes, Error> {
        match (self, new_prefix_length) {
            (Self::Ipv4(prefix), Self::Length::Ipv4(length)) => {
                prefix.subprefixes(length).map(Self::Subprefixes::Ipv4)
            }
            (Self::Ipv6(prefix), Self::Length::Ipv6(length)) => {
                prefix.subprefixes(length).map(Self::Subprefixes::Ipv6)
            }
            _ => Err(err!(Kind::AfiMismatch)),
        }
    }

    fn new_prefix_length(&self, length: u8) -> Result<Self::Length, Error> {
        self.afi().new_prefix_length(length)
    }
}

impl From<concrete::Prefix<Ipv4>> for Prefix {
    fn from(prefix: concrete::Prefix<Ipv4>) -> Self {
        Self::Ipv4(prefix)
    }
}

impl From<concrete::Prefix<Ipv6>> for Prefix {
    fn from(prefix: concrete::Prefix<Ipv6>) -> Self {
        Self::Ipv6(prefix)
    }
}

impl FromStr for Prefix {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        concrete::Prefix::<Ipv4>::from_str(s)
            .map(Self::from)
            .or_else(|_| concrete::Prefix::<Ipv6>::from_str(s).map(Self::from))
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4(prefix) => prefix.fmt(f),
            Self::Ipv6(prefix) => prefix.fmt(f),
        }
    }
}

#[cfg(any(test, feature = "arbitrary"))]
impl Arbitrary for Prefix {
    type Parameters = (
        ParamsFor<concrete::Prefix<Ipv4>>,
        ParamsFor<concrete::Prefix<Ipv6>>,
    );
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            any_with::<concrete::Prefix<Ipv4>>(params.0).prop_map(Self::Ipv4),
            any_with::<concrete::Prefix<Ipv6>>(params.1).prop_map(Self::Ipv6),
        ]
        .boxed()
    }
}
