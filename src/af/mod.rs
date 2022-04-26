use core::cmp::Ord;
use core::fmt::Debug;
use core::hash::Hash;

use crate::{
    addr::{AddressI, AnyAddress, ConcreteAddress},
    error::Error,
    mask::{AnyHostmask, AnyNetmask, ConcreteHostmask, ConcreteNetmask, MaskI},
    parser,
    prefix::{
        AnyPrefix, AnyPrefixLength, ConcretePrefix, ConcretePrefixLength, PrefixI, PrefixLengthI,
    },
    primitive::AddressPrimitive,
};

mod macros;

use self::macros::afi_definitions;

/// Provides an interface for describing an IP address family.
pub trait Afi: DefaultPrimitives + Copy + Debug + Hash + Ord {
    type Octets;
    /// Get the [`AfiEnum`] variant associated with `Self`.
    fn as_enum() -> AfiEnum;
}

pub trait Afis {}
impl<A: Afi> Afis for A {}

pub trait Primitives<As: Afis> {}
impl<A: Afi, P: AddressPrimitive<A>> Primitives<A> for P {}

pub trait DefaultPrimitives: Afis + Sized {
    type Type: Primitives<Self>;
}
pub type DefaultPrimitive<A> = <A as DefaultPrimitives>::Type;

/// Provides an interface for describing a class of IP address families.
pub trait AfiClass<As: Afis, Ps: Primitives<As>>: Copy + Debug + Hash + Ord {
    type Address: AddressI;
    type PrefixLength: PrefixLengthI;
    type Prefix: PrefixI;
    type Netmask: MaskI;
    type Hostmask: MaskI;
}
impl<A: Afi, P: AddressPrimitive<A>> AfiClass<A, P> for A {
    type Address = ConcreteAddress<A, P>;
    type PrefixLength = ConcretePrefixLength<A, P>;
    type Prefix = ConcretePrefix<A, P>;
    type Netmask = ConcreteNetmask<A, P>;
    type Hostmask = ConcreteHostmask<A, P>;
}
pub type Address<A, P = DefaultPrimitive<A>> = <A as AfiClass<A, P>>::Address;
pub type PrefixLength<A, P = DefaultPrimitive<A>> = <A as AfiClass<A, P>>::PrefixLength;
pub type Prefix<A, P = DefaultPrimitive<A>> = <A as AfiClass<A, P>>::Prefix;
pub type Netmask<A, P = DefaultPrimitive<A>> = <A as AfiClass<A, P>>::Netmask;
pub type Hostmask<A, P = DefaultPrimitive<A>> = <A as AfiClass<A, P>>::Hostmask;

afi_definitions! {
    pub class Any {
        type Address = AnyAddress;
        type PrefixLength = AnyPrefixLength;
        type Prefix = AnyPrefix;
        type Netmask = AnyNetmask;
        type Hostmask = AnyHostmask;
        /// IPv4 address family marker type.
        pub afi Ipv4 (P4) {
            type Octets = [u8; 4];
            type DefaultPrimitive = u32;
            primitive u32 {
                type Width = u8;

                const MAX_LENGTH = 32;
                const ZERO = 0x0000_0000;
                const ONES = 0xffff_ffff;

                const BROADCAST = Some(Self::ONES);
                const LOCALHOST = 0x7f00_0001;
                const UNSPECIFIED = Self::ZERO;

                const LOCALHOST_NET = (0x7f00_0000, 8);
                const BENCHMARK_NET = (0xc612_0000, 15);
                const MULTICAST_NET = (0xe000_0000, 4);

                fn parse_addr = parser::ipv4::parse_addr;
                fn parse_prefix = parser::ipv4::parse_prefix;
            }
        }
        /// IPv6 address family marker type.
        pub afi Ipv6 (P6) {
            type Octets = [u8; 16];
            type DefaultPrimitive = u128;
            primitive u128 {
                type Width = u8;

                const MAX_LENGTH = 128;
                const ZERO = 0x0000_0000_0000_0000_0000_0000_0000_0000;
                const ONES = 0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff;

                const BROADCAST = None;
                const LOCALHOST = 0x0000_0000_0000_0000_0000_0000_0000_0001;
                const UNSPECIFIED = Self::ZERO;

                const LOCALHOST_NET = (0x0000_0000_0000_0000_0000_0000_0000_0001, 128);
                const BENCHMARK_NET = (0x2001_0002_0000_0000_0000_0000_0000_0000, 48);
                const MULTICAST_NET = (0xff00_0000_0000_0000_0000_0000_0000_0000, 8);

                fn parse_addr = parser::ipv6::parse_addr;
                fn parse_prefix = parser::ipv6::parse_prefix;
            }
        }
    }
}
