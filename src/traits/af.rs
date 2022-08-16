use core::borrow::Borrow;
use core::cmp::Ord;
use core::fmt::Debug;
use core::hash::Hash;

use crate::{any, concrete, fmt};

use super::{Address, Hostmask, Interface, Netmask, Prefix, PrefixLength};

use super::primitive;

/// An interface for describing an IP address family.
pub trait Afi: Copy + Debug + Hash + Ord {
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

    /// The type respresenting IP interface values of this address family class.
    type Interface: Interface;

    /// The type respresenting IP prefix values of this address family class.
    type Prefix: Prefix;

    /// The type respresenting IP prefix-length values of this address family class.
    type PrefixLength: PrefixLength;

    /// The type respresenting IP netmask values of this address family class.
    type Netmask: Netmask;

    /// The type respresenting IP hostmask values of this address family class.
    type Hostmask: Hostmask;

    /// Get the [`any::AfiClass`] variant associated with `Self`.
    fn as_afi_class() -> any::AfiClass;
}
