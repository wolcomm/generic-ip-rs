use core::fmt::Debug;
use core::hash::Hash;

#[cfg(feature = "std")]
use super::PrefixSet;
use super::{
    primitive, Address, Bitmask, Hostmask, Interface, Netmask, Prefix, PrefixLength, PrefixRange,
};
use crate::{any, concrete, fmt};

/// An interface for describing an IP address family.
pub trait Afi: Copy + Debug + Hash + Ord + 'static {
    // This bound is required to satisfy coherence rules when implementing
    // `From<A::Octets> for Address<A>`
    /// The big-endian byte array representation of addresses of this address
    /// family.
    type Octets: primitive::Octets;

    /// The primitive integer type used to store address values of this address
    /// family.
    type Primitive: primitive::Address<Self> + fmt::AddressDisplay<Self>;

    /// Get the [`concrete::Afi`] variant associated with `Self`.
    fn as_afi() -> concrete::Afi;
}

/// Provides an interface for describing a class of IP address families.
pub trait AfiClass: Copy + Debug + Hash + Ord {
    /// The type representing IP address values of this address family class.
    type Address: Address;

    /// The type representing IP netmask values of this address family class.
    type Netmask: Netmask;

    /// The type representing IP hostmask values of this address family class.
    type Hostmask: Hostmask;

    /// The type representing bitmask values of this address family class.
    type Bitmask: Bitmask;

    /// The type representing IP prefix-length values of this address family
    /// class.
    type PrefixLength: PrefixLength + Into<Self::Netmask> + Into<Self::Hostmask>;

    /// The type representing IP prefix values of this address family class.
    type Prefix: Prefix<Address = Self::Address, Length = Self::PrefixLength, Netmask = Self::Netmask>
        + Into<Self::PrefixRange>;

    /// The type representing IP interface values of this address family class.
    type Interface: Interface<
        Address = Self::Address,
        Prefix = Self::Prefix,
        PrefixLength = Self::PrefixLength,
    >;

    /// The type representing IP prefix range values of this address family
    /// class.
    type PrefixRange: PrefixRange<Prefix = Self::Prefix, Length = Self::PrefixLength>;

    /// The type representing IP prefix-sets of this address family class.
    #[cfg(feature = "std")]
    type PrefixSet: for<'a> PrefixSet<'a, Prefix = Self::Prefix, Range = Self::PrefixRange>;

    /// Get the [`any::AfiClass`] variant associated with `Self`.
    fn as_afi_class() -> any::AfiClass;
}
