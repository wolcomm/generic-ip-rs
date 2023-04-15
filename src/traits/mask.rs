use core::fmt::Debug;
use core::hash::Hash;

/// Address-family independent interface for IP masks.
///
/// Methods on `Mask` types that are well defined for all address-families
/// are implemented via this trait.
///
/// See also [`concrete::Mask<T, A>`][crate::concrete::Mask] and
/// [`any::Mask<T>`][crate::any::Mask] for address-family specific items.
pub trait Mask: Clone + Copy + Debug + Hash + PartialEq + Eq {}

/// Address-family independent interface for IP netmasks.
///
/// Methods on `Netmask` types that are well defined for all address-families
/// are implemented via this trait.
///
/// See also [`concrete::Netmask<A>`][crate::concrete::Netmask] and
/// [`any::Netmask`][crate::any::Netmask] for address-family specific items.
pub trait Netmask: Mask {}

/// Address-family independent interface for IP hostmasks.
///
/// Methods on `Hostmask` types that are well defined for all address-families
/// are implemented via this trait.
///
/// See also [`concrete::Hostmask<A>`][crate::concrete::Hostmask] and
/// [`any::Hostmask`][crate::any::Hostmask] for address-family specific items.
pub trait Hostmask: Mask {}

/// Address-family independent interface for IP bitmasks.
///
/// Methods on `Bitmask` types that are well defined for all address-families
/// are implemented via this trait.
///
/// See also [`concrete::Bitmask<A>`][crate::concrete::Bitmask] and
/// [`any::Bitmask`][crate::any::Bitmask] for address-family specific items.
pub trait Bitmask: Mask {}
