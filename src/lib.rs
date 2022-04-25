//! Types and traits for working with IP addresses and prefixes generically
//! over address families.
#![doc(html_root_url = "https://docs.rs/generic-ip/0.1.0-alpha.2")]
#![no_std]
// #![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

/// IP address and address-mask types and helper functions.
pub mod addr;
/// IP address family traits and marker types.
pub mod af;
/// IP address formatting traits
pub mod fmt;
/// IP prefix and prefix-length types.
pub mod prefix;
/// Number-like primitives for IP address and prefix representation.
pub mod primitive;

mod parser;

mod error;

#[cfg(test)]
mod tests;
