//! Types and traits for working with IP addresses and prefixes generically
//! over address families.
#![doc(html_root_url = "https://docs.rs/generic-ip/0.1.0-alpha.2")]
#![no_std]
// #![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

pub mod any;
pub mod concrete;
pub mod traits;

pub use self::any::Any;
pub use self::concrete::{Ipv4, Ipv6};
pub use self::traits::{Afi, AfiClass};

/// IP address formatting traits
mod fmt;

mod parser;

mod error;

pub type Address<A> = <A as AfiClass>::Address;
pub type PrefixLength<A> = <A as AfiClass>::PrefixLength;
pub type Prefix<A> = <A as AfiClass>::Prefix;
pub type Netmask<A> = <A as AfiClass>::Netmask;
pub type Hostmask<A> = <A as AfiClass>::Hostmask;

#[cfg(test)]
mod tests;
