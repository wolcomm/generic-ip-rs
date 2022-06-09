use core::fmt;

use crate::{any, traits};

use super::{Address, Hostmask, Netmask, Prefix, PrefixLength};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ipv4 {}

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
    type PrefixLength = PrefixLength<A>;
    type Prefix = Prefix<A>;
    type Netmask = Netmask<A>;
    type Hostmask = Hostmask<A>;

    fn as_afi_class() -> any::AfiClass {
        A::as_afi().into()
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Afi {
    Ipv4,
    Ipv6,
}

impl fmt::Display for Afi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4 => f.write_str("ipv4"),
            Self::Ipv6 => f.write_str("ipv6"),
        }
    }
}
