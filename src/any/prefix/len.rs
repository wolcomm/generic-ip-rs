use core::fmt;

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any, Arbitrary},
    prop_oneof,
    strategy::{BoxedStrategy, Strategy},
};

use super::delegate;
use crate::{
    concrete::{self, Ipv4, Ipv6},
    traits, Error,
};

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
