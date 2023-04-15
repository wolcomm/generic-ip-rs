use core::borrow::Borrow;
use core::cmp::Ord;
use core::fmt::Debug;
use core::hash::Hash;

use super::primitive;
use super::{Address, Bitmask, Hostmask, Interface, Netmask, Prefix, PrefixLength, PrefixRange};
use crate::{any, concrete, fmt};

/// An interface for describing an IP address family.
pub trait Afi: Copy + Debug + Hash + Ord + 'static {
    // This bound is required to satisfy coherence rules when implementing
    // `From<A::Octets> for Address<A>`
    /// The big-endian byte array representation of addresses of this address
    /// family.
    type Octets: Borrow<[u8]>;

    /// The primitive integer type used to store address values of this address
    /// family.
    type Primitive: primitive::Address<Self> + fmt::AddressDisplay<Self>;

    /// Get the [`concrete::Afi`] variant associated with `Self`.
    fn as_afi() -> concrete::Afi;
}

/// Provides an interface for describing a class of IP address families.
pub trait AfiClass: Copy + Debug + Hash + Ord {
    /// The type respresenting IP address values of this address family class.
    type Address: Address;

    /// The type respresenting IP netmask values of this address family class.
    type Netmask: Netmask;

    /// The type respresenting IP hostmask values of this address family class.
    type Hostmask: Hostmask;

    /// The type respresenting bitmask values of this address family class.
    type Bitmask: Bitmask;

    /// The type respresenting IP prefix-length values of this address family
    /// class.
    type PrefixLength: PrefixLength + Into<Self::Netmask> + Into<Self::Hostmask>;

    /// The type respresenting IP prefix values of this address family class.
    type Prefix: Prefix<Address = Self::Address, Length = Self::PrefixLength, Netmask = Self::Netmask>
        + Into<Self::PrefixRange>;

    /// The type respresenting IP interface values of this address family class.
    type Interface: Interface<
        Address = Self::Address,
        Prefix = Self::Prefix,
        PrefixLength = Self::PrefixLength,
    >;

    /// The type respresenting IP prefix range values of this address family
    /// class.
    type PrefixRange: PrefixRange<Prefix = Self::Prefix, Length = Self::PrefixLength>;

    /// Get the [`any::AfiClass`] variant associated with `Self`.
    fn as_afi_class() -> any::AfiClass;
}
