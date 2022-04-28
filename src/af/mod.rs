use core::cmp::Ord;
use core::fmt::{self, Debug};
use core::hash::Hash;

use crate::{
    addr::{AddressI, AnyAddress, ConcreteAddress},
    mask::{AnyHostmask, AnyNetmask, ConcreteHostmask, ConcreteNetmask, MaskI},
    prefix::{
        AnyPrefix, AnyPrefixLength, ConcretePrefix, ConcretePrefixLength, PrefixI, PrefixLengthI,
    },
    primitive::AddressPrimitive,
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
    type AddressPrimitive: AddressPrimitive<Self>;
    /// Get the [`AfiEnum`] variant associated with `Self`.
    fn as_enum() -> AfiEnum;
}

impl Afi for Ipv4 {
    type Octets = [u8; 4];
    type AddressPrimitive = u32;
    fn as_enum() -> AfiEnum {
        AfiEnum::Ipv4
    }
}
impl Afi for Ipv6 {
    type Octets = [u8; 16];
    type AddressPrimitive = u128;
    fn as_enum() -> AfiEnum {
        AfiEnum::Ipv6
    }
}

/// Provides an interface for describing a class of IP address families.
pub trait AfiClass: Copy + Debug + Hash + Ord {
    type Address: AddressI;
    type PrefixLength: PrefixLengthI;
    type Prefix: PrefixI;
    type Netmask: MaskI;
    type Hostmask: MaskI;
}
impl<A: Afi> AfiClass for A {
    type Address = ConcreteAddress<A>;
    type PrefixLength = ConcretePrefixLength<A>;
    type Prefix = ConcretePrefix<A>;
    type Netmask = ConcreteNetmask<A>;
    type Hostmask = ConcreteHostmask<A>;
}
impl AfiClass for Any {
    type Address = AnyAddress;
    type PrefixLength = AnyPrefixLength;
    type Prefix = AnyPrefix;
    type Netmask = AnyNetmask;
    type Hostmask = AnyHostmask;
}
