use core::str::FromStr;

use crate::{
    concrete::{self, Ipv4, Ipv6},
    error::Error,
    traits,
};

use super::{delegate, Address, Prefix, PrefixLength};

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
/// use ip::{Interface, Any, traits::{Address as _, Interface as _}};
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

// TODO: impl Display for Prefix
// TODO: impl Arbitrary for Prefix
