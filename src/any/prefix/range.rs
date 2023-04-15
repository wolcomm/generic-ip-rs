use core::fmt;
use core::ops::RangeInclusive;
use core::str::FromStr;

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any, Arbitrary},
    prop_oneof,
    strategy::{BoxedStrategy, Strategy},
};

use super::{delegate, Length, Prefix};
use crate::{
    concrete::{self, Ipv4, Ipv6},
    traits, Error,
};

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

    fn with_intersection(self, len_range: RangeInclusive<Self::Length>) -> Option<Self> {
        match (self, *len_range.start(), *len_range.end()) {
            (Self::Ipv4(range), Length::Ipv4(lower), Length::Ipv4(upper)) => {
                range.with_intersection(lower..=upper).map(Self::Ipv4)
            }
            (Self::Ipv6(range), Length::Ipv6(lower), Length::Ipv6(upper)) => {
                range.with_intersection(lower..=upper).map(Self::Ipv6)
            }
            _ => None,
        }
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

#[allow(clippy::fallible_impl_from)]
impl From<Prefix> for Range {
    fn from(prefix: Prefix) -> Self {
        match prefix {
            Prefix::Ipv4(p) => Self::Ipv4(p.into()),
            Prefix::Ipv6(p) => Self::Ipv6(p.into()),
        }
    }
}

impl FromStr for Range {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        concrete::PrefixRange::<Ipv4>::from_str(s)
            .map(Self::from)
            .or_else(|_| concrete::PrefixRange::<Ipv6>::from_str(s).map(Self::from))
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

impl IntoIterator for Range {
    type IntoIter = IntoIter;
    type Item = Prefix;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Ipv4(range) => Self::IntoIter::Ipv4(range.into_iter()),
            Self::Ipv6(range) => Self::IntoIter::Ipv6(range.into_iter()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IntoIter {
    Ipv4(<concrete::PrefixRange<Ipv4> as IntoIterator>::IntoIter),
    Ipv6(<concrete::PrefixRange<Ipv6> as IntoIterator>::IntoIter),
}

impl Iterator for IntoIter {
    type Item = Prefix;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Ipv4(iter) => iter.next().map(Prefix::Ipv4),
            Self::Ipv6(iter) => iter.next().map(Prefix::Ipv6),
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
