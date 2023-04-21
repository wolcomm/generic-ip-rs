use core::fmt;

#[cfg(feature = "std")]
use super::PrefixSet;
use super::{Address, Bitmask, Hostmask, Interface, Netmask, Prefix, PrefixLength, PrefixRange};
use crate::{any, error::Error, traits};

/// The IPv4 address family.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ipv4 {}

/// The IPv6 address family.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ipv6 {}

impl traits::Afi for Ipv4 {
    type Octets = [u8; 4];
    type Primitive = u32;
    fn as_afi() -> Afi {
        Afi::Ipv4
    }
}
impl traits::Afi for Ipv6 {
    type Octets = [u8; 16];
    type Primitive = u128;
    fn as_afi() -> Afi {
        Afi::Ipv6
    }
}

impl<A: traits::Afi> traits::AfiClass for A {
    type Address = Address<A>;
    type Interface = Interface<A>;
    type PrefixLength = PrefixLength<A>;
    type Prefix = Prefix<A>;
    type Netmask = Netmask<A>;
    type Hostmask = Hostmask<A>;
    type Bitmask = Bitmask<A>;
    type PrefixRange = PrefixRange<A>;

    #[cfg(feature = "std")]
    type PrefixSet = PrefixSet<A>;

    fn as_afi_class() -> any::AfiClass {
        A::as_afi().into()
    }
}

/// Enumeration of concrete address families.
///
/// # Examples
///
/// ``` rust
/// use ip::{Afi, Ipv4, Ipv6};
///
/// assert_eq!(Ipv4::as_afi().to_string(), "ipv4");
/// assert_eq!(Ipv6::as_afi().to_string(), "ipv6");
/// ```
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Afi {
    /// Variant representing the IPv4 address family.
    Ipv4,
    /// Variant representing the IPv6 address family.
    Ipv6,
}

impl Afi {
    pub(crate) fn new_prefix_length(self, len: u8) -> Result<any::PrefixLength, Error> {
        match self {
            Self::Ipv4 => PrefixLength::<Ipv4>::from_primitive(len).map(any::PrefixLength::Ipv4),
            Self::Ipv6 => PrefixLength::<Ipv6>::from_primitive(len).map(any::PrefixLength::Ipv6),
        }
    }
}

impl fmt::Display for Afi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4 => f.write_str("ipv4"),
            Self::Ipv6 => f.write_str("ipv6"),
        }
    }
}
