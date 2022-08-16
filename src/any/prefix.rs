use core::str::FromStr;

use super::{delegate, Address, Hostmask, Netmask};
use crate::{
    concrete::{self, Ipv4, Ipv6},
    error::Error,
    traits,
};

/// Either an IPv4 or IPv6 prefix.
///
/// # Memory Use
///
/// Rust enums are sized to accomodate their largest variant, with smaller
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
    type PrefixLength = PrefixLength;
    type Hostmask = Hostmask;
    type Netmask = Netmask;

    delegate! {
        fn network(&self) -> Self::Address;
        fn hostmask(&self) -> Self::Hostmask;
        fn netmask(&self) -> Self::Netmask;
        fn max_prefix_len(&self) -> Self::PrefixLength;
        fn prefix_len(&self) -> Self::PrefixLength;
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

// TODO: impl Display for Prefix
// TODO: impl Arbitrary for Prefix

/// The length of either an IPv4 or IPv6 prefix.
///
/// # Examples
///
/// ``` rust
/// use ip::{traits::Prefix as _, Any, Prefix, PrefixLength};
///
/// let prefix = "2001:db8::/32".parse::<Prefix<Any>>()?;
///
/// assert!(matches!(prefix.prefix_len(), PrefixLength::<Any>::Ipv6(_)));
/// # Ok::<(), ip::Error>(())
/// ```
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub enum PrefixLength {
    /// IPv4 prefix length variant.
    Ipv4(concrete::PrefixLength<Ipv4>),
    /// IPv6 prefix length variant.
    Ipv6(concrete::PrefixLength<Ipv6>),
}

impl traits::PrefixLength for PrefixLength {}

impl From<concrete::PrefixLength<Ipv4>> for PrefixLength {
    fn from(length: concrete::PrefixLength<Ipv4>) -> Self {
        Self::Ipv4(length)
    }
}

impl From<concrete::PrefixLength<Ipv6>> for PrefixLength {
    fn from(length: concrete::PrefixLength<Ipv6>) -> Self {
        Self::Ipv6(length)
    }
}

// TODO: impl Display for PrefixLength
// TODO: impl FromStr for PrefixLength
// TODO: impl Arbitrary for PrefixLength
