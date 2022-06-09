use core::cmp::Ordering;

use crate::{concrete, traits};

use super::{Address, Hostmask, Netmask, Prefix, PrefixLength};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Any {}

impl traits::AfiClass for Any {
    type Address = Address;
    type PrefixLength = PrefixLength;
    type Prefix = Prefix;
    type Netmask = Netmask;
    type Hostmask = Hostmask;

    fn as_afi_class() -> AfiClass {
        AfiClass::Any
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum AfiClass {
    Ipv4,
    Ipv6,
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
