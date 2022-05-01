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

pub enum AfiClass {
    Ipv4,
    Ipv6,
    Any,
}

impl From<concrete::Afi> for AfiClass {
    fn from(afi: concrete::Afi) -> Self {
        match afi {
            concrete::Afi::Ipv4 => AfiClass::Ipv4,
            concrete::Afi::Ipv6 => AfiClass::Ipv6,
        }
    }
}
