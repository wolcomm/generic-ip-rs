use core::borrow::Borrow;
use core::cmp::Ord;
use core::fmt::Debug;
use core::hash::Hash;

use crate::{any, concrete, fmt};

use super::{Address, Mask, Prefix, PrefixLength};

use super::primitive;

/// Provides an interface for describing an IP address family.
pub trait Afi: Copy + Debug + Hash + Ord {
    // This bound required to satisfy coherence rules when implementing
    // `From<A::Octets> for Address<A>`
    type Octets: Borrow<[u8]>;
    type Primitive: primitive::Address<Self> + fmt::AddressDisplay<Self>;

    /// Get the [`concrete::Afi`] variant associated with `Self`.
    fn as_afi() -> concrete::Afi;
}

/// Provides an interface for describing a class of IP address families.
pub trait AfiClass: Copy + Debug + Hash + Ord {
    type Address: Address;
    type PrefixLength: PrefixLength;
    type Prefix: Prefix;
    type Netmask: Mask;
    type Hostmask: Mask;

    /// Get the [`any::AfiClass`] variant associated with `Self`.
    fn as_afi_class() -> any::AfiClass;
}
