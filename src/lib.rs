//! Types and traits for working with IP addresses and prefixes generically
//! over address families.
#![doc(html_root_url = "https://docs.rs/generic-ip/0.1.0-alpha.2")]
#![no_std]
// #![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

/// IP address types and helper functions.
mod addr;
/// IP address family traits and marker types.
mod af;
pub use self::af::{Afi, AfiClass, Any, Ipv4, Ipv6};

/// IP address formatting traits
mod fmt;
/// IP address-mask types and helper functions.
mod mask;
/// IP prefix and prefix-length types.
mod prefix;
pub use self::prefix::Ordering as PrefixOrdering;

/// Number-like primitives for IP address and prefix representation.
mod primitive;
pub use self::primitive::AddressPrimitive;

mod parser;

mod error;

pub type Address<A> = <A as AfiClass>::Address;
pub type PrefixLength<A> = <A as AfiClass>::PrefixLength;
pub type Prefix<A> = <A as AfiClass>::Prefix;
pub type Netmask<A> = <A as AfiClass>::Netmask;
pub type Hostmask<A> = <A as AfiClass>::Hostmask;

#[cfg(test)]
mod tests;
