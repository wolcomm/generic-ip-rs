use core::cmp::Ordering;

use super::{Address, Bitmask, Hostmask, Interface, Netmask, Prefix, PrefixLength, PrefixRange};
use crate::{concrete, traits};

/// The class of address families consisting of `{ IPv4, IPv6 }`.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Any {}

impl traits::AfiClass for Any {
    type Address = Address;
    type Interface = Interface;
    type PrefixLength = PrefixLength;
    type Prefix = Prefix;
    type Netmask = Netmask;
    type Hostmask = Hostmask;
    type Bitmask = Bitmask;
    type PrefixRange = PrefixRange;

    fn as_afi_class() -> AfiClass {
        AfiClass::Any
    }
}

/// Enumeration of address family classes.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum AfiClass {
    /// Variant representing the class `{ IPv4 }`.
    Ipv4,
    /// Variant representing the class `{ IPv6 }`.
    Ipv6,
    /// Variant representing the [`Any`] class.
    Any,
}

impl From<concrete::Afi> for AfiClass {
    fn from(afi: concrete::Afi) -> Self {
        match afi {
            concrete::Afi::Ipv4 => Self::Ipv4,
            concrete::Afi::Ipv6 => Self::Ipv6,
        }
    }
}

impl PartialOrd for AfiClass {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            _ if self == other => Some(Ordering::Equal),
            (Self::Any, Self::Ipv4 | Self::Ipv6) => Some(Ordering::Greater),
            (Self::Ipv4 | Self::Ipv6, Self::Any) => Some(Ordering::Less),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_contains_ipv4() {
        assert!(AfiClass::Any > AfiClass::Ipv4);
    }

    #[test]
    fn ipv6_contained_in_any() {
        assert!(AfiClass::Ipv6 < AfiClass::Any);
    }
}
