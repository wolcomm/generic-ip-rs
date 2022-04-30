use core::cmp::Ord;
use core::fmt::{self, Debug};
use core::hash::Hash;

use crate::{
    any, concrete,
    traits::{self, primitive},
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ipv4 {}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ipv6 {}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Any {}

pub enum AfiEnum {
    Ipv4,
    Ipv6,
}

impl fmt::Display for AfiEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Ipv4 => f.write_str("ipv4"),
            Self::Ipv6 => f.write_str("ipv6"),
        }
    }
}

/// Provides an interface for describing an IP address family.
pub trait Afi: Copy + Debug + Hash + Ord {
    type Octets;
    type Primitive: primitive::Address<Self>;
    /// Get the [`AfiEnum`] variant associated with `Self`.
    fn as_enum() -> AfiEnum;
}

impl Afi for Ipv4 {
    type Octets = [u8; 4];
    type Primitive = u32;
    fn as_enum() -> AfiEnum {
        AfiEnum::Ipv4
    }
}
impl Afi for Ipv6 {
    type Octets = [u8; 16];
    type Primitive = u128;
    fn as_enum() -> AfiEnum {
        AfiEnum::Ipv6
    }
}

/// Provides an interface for describing a class of IP address families.
pub trait AfiClass: Copy + Debug + Hash + Ord {
    type Address: traits::Address;
    type PrefixLength: traits::PrefixLength;
    type Prefix: traits::Prefix;
    type Netmask: traits::Mask;
    type Hostmask: traits::Mask;
}
impl<A: Afi> AfiClass for A {
    type Address = concrete::Address<A>;
    type PrefixLength = concrete::PrefixLength<A>;
    type Prefix = concrete::Prefix<A>;
    type Netmask = concrete::Netmask<A>;
    type Hostmask = concrete::Hostmask<A>;
}
impl AfiClass for Any {
    type Address = any::Address;
    type PrefixLength = any::PrefixLength;
    type Prefix = any::Prefix;
    type Netmask = any::Netmask;
    type Hostmask = any::Hostmask;
}
