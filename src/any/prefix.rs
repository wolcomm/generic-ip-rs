use core::fmt;
use core::ops::RangeInclusive;
use core::str::FromStr;

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any, any_with, Arbitrary, ParamsFor},
    prop_oneof,
    strategy::{BoxedStrategy, Strategy},
};

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
    type Length = Length;
    type Hostmask = Hostmask;
    type Netmask = Netmask;

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
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub enum Length {
    /// IPv4 prefix length variant.
    Ipv4(concrete::PrefixLength<Ipv4>),
    /// IPv6 prefix length variant.
    Ipv6(concrete::PrefixLength<Ipv6>),
}

impl traits::PrefixLength for Length {
    fn increment(self) -> Result<Self, Error> {
        match self {
            Self::Ipv4(length) => length.increment().map(Self::from),
            Self::Ipv6(length) => length.increment().map(Self::from),
        }
    }

    fn decrement(self) -> Result<Self, Error> {
        match self {
            Self::Ipv4(length) => length.decrement().map(Self::from),
            Self::Ipv6(length) => length.decrement().map(Self::from),
        }
    }
}

impl From<concrete::PrefixLength<Ipv4>> for Length {
    fn from(length: concrete::PrefixLength<Ipv4>) -> Self {
        Self::Ipv4(length)
    }
}

impl From<concrete::PrefixLength<Ipv6>> for Length {
    fn from(length: concrete::PrefixLength<Ipv6>) -> Self {
        Self::Ipv6(length)
    }
}

impl AsRef<u8> for Length {
    delegate! {
        fn as_ref(&self) -> &u8;
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4(len) => len.fmt(f),
            Self::Ipv6(len) => len.fmt(f),
        }
    }
}

#[cfg(any(test, feature = "arbitrary"))]
impl Arbitrary for Length {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            any::<concrete::PrefixLength<Ipv4>>().prop_map(Self::Ipv4),
            any::<concrete::PrefixLength<Ipv6>>().prop_map(Self::Ipv6),
        ]
        .boxed()
    }
}

/// Either an IPv4 or IPv6 prefix range.
///
/// See also: [`concrete::PrefixRange`][crate::concrete::PrefixRange].
///
/// # Memory Use
///
/// Rust enums are sized to accomodate their largest variant, with smaller
/// variants being padded to fill up any unused space.
///
/// As a result, users should avoid using this type in a context where only
/// [`PrefixRange::Ipv4`] variants are expected.
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
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub enum Range {
    /// IPv4 prefix variant.
    Ipv4(concrete::PrefixRange<Ipv4>),
    /// IPv6 prefix variant.
    Ipv6(concrete::PrefixRange<Ipv6>),
}

impl traits::PrefixRange for Range {
    type Prefix = Prefix;
    type Length = Length;

    delegate! {
        fn prefix(&self) -> Self::Prefix;
        fn lower(&self) -> Self::Length;
        fn upper(&self) -> Self::Length;
    }

    fn with_length_range(self, len_range: RangeInclusive<Self::Length>) -> Option<Self> {
        match (self, *len_range.start(), *len_range.end()) {
            (Self::Ipv4(range), Length::Ipv4(lower), Length::Ipv4(upper)) => {
                range.with_length_range(lower..=upper).map(Self::Ipv4)
            }
            (Self::Ipv6(range), Length::Ipv6(lower), Length::Ipv6(upper)) => {
                range.with_length_range(lower..=upper).map(Self::Ipv6)
            }
            _ => None,
        }
    }
}

impl From<concrete::PrefixRange<Ipv4>> for Range {
    fn from(range: concrete::PrefixRange<Ipv4>) -> Self {
        Self::Ipv4(range)
    }
}

impl From<concrete::PrefixRange<Ipv6>> for Range {
    fn from(range: concrete::PrefixRange<Ipv6>) -> Self {
        Self::Ipv6(range)
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4(range) => range.fmt(f),
            Self::Ipv6(range) => range.fmt(f),
        }
    }
}

#[cfg(any(test, feature = "arbitrary"))]
impl Arbitrary for Range {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            any::<concrete::PrefixRange<Ipv4>>().prop_map(Self::Ipv4),
            any::<concrete::PrefixRange<Ipv6>>().prop_map(Self::Ipv6),
        ]
        .boxed()
    }
}
