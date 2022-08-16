//! Types and traits for working with IP addresses and prefixes generically
//! over address families.
//!
//! The IP address types in [`std::net`] do not share any common trait that
//! expresses "this thing is an IP address".
//!
//! This limitation makes writing code that deals with IP addresses in an
//! address-family-independent way unnecessarily difficult.
//!
//! This crate provides a collection of types that seek to be compatible
//! with the address types from [`std::net`] and prefix types from the
//! popular [`ipnet`] crate, but which are generic over address-families.
//!
//! For example:
//!
//! ``` rust
//! use ip::{Address, Afi, Error, Ipv4, Ipv6, Prefix};
//!
//! struct RibEntry<A: Afi> {
//!     prefix: Prefix<A>,
//!     next_hop: Address<A>,
//! }
//!
//! impl<A: Afi> RibEntry<A> {
//!     fn get_next_hop(&self, addr: Address<A>) -> Option<Address<A>> {
//!         (self.prefix >= addr).then(|| self.next_hop)
//!     }
//! }
//!
//! fn main() -> Result<(), Error> {
//!     let v4: RibEntry<Ipv4> = RibEntry {
//!         prefix: "192.0.2.0/24".parse()?,
//!         next_hop: "198.51.100.1".parse()?,
//!     };
//!
//!     let v6: RibEntry<Ipv6> = RibEntry {
//!         prefix: "2001:db8::/48".parse()?,
//!         next_hop: "2001:db8:f00::1".parse()?,
//!     };
//!
//!     assert_eq!(
//!         v4.get_next_hop("192.0.2.127".parse()?),
//!         Some("198.51.100.1".parse()?)
//!     );
//!     assert_eq!(v6.get_next_hop("2001:db8:ffff::ffff".parse()?), None);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Orientation
//!
//! Names such as `Address`, `Interface`, `Prefix` or `Afi` are re-used in
//! various different modules within the crate.
//! For example `Address` is used to name:
//! - the type alias [`ip::Address<A>`][crate::Address]
//! - the types [`ip::concrete::Address<A>`][crate::concrete::Address] and
//!   [`ip::any::Address`][crate::any::Address]
//! - the trait [`ip::traits::Address`][crate::traits::Address]
//!
//! This can make understanding which item a given name is referring to
//! difficult without understanding the crate layout.
//!
//! ### Address-families
//!
//! The IP address-families `ipv4` and `ipv6` are respresented in the type
//! system by the zero-sized types [`concrete::Ipv4`] and [`concrete::Ipv6`].
//!
//! These "concrete" address-families implement [`traits::Afi`], which in turn
//! bounds the generic parameter of the items exported by the [`concrete`]
//! module, such as [`concrete::Address<A>`] and [`concrete::Prefix<A>`].
//!
//! Conversely, the [`any`] module exports a collection of `enum`s with
//! variants corresponding to the two concrete address families, with each
//! variant containing the corresponding `concrete::*` item.
//!
//! ### Address-family classes
//!
//! Usually a given use-case will call for *either* processing objects of a
//! single known (at compile time) address-family or objects that may be of
//! either address-family, as in the following:
//!
//! ``` rust
//! use ip::{any, concrete, Afi, Ipv4, Ipv6};
//!
//! // `x` and `y` must be the same address-family
//! fn longer_concrete<A: Afi>(
//!     x: concrete::Prefix<A>,
//!     y: concrete::Prefix<A>,
//! ) -> concrete::Prefix<A> {
//!     if x.length() > y.length() {
//!         x
//!     } else {
//!         y
//!     }
//! }
//!
//! // `x` and `y` may be of different address families, so may not be
//! // comparable
//! fn longer_any(x: any::Prefix, y: any::Prefix) -> Option<any::Prefix> {
//!     match (x, y) {
//!         (any::Prefix::Ipv4(x), any::Prefix::Ipv4(y)) => Some(longer_concrete(x, y).into()),
//!         (any::Prefix::Ipv6(x), any::Prefix::Ipv6(y)) => Some(longer_concrete(x, y).into()),
//!         _ => None,
//!     }
//! }
//!
//! let x4: concrete::Prefix<Ipv4> = "192.0.2.0/24".parse().unwrap();
//! let y4: concrete::Prefix<Ipv4> = "203.0.113.128/25".parse().unwrap();
//!
//! let x6: concrete::Prefix<Ipv6> = "2001:db8:f00::/48".parse().unwrap();
//! let y6: concrete::Prefix<Ipv6> = "2001:db8::/32".parse().unwrap();
//!
//! assert_eq!(longer_concrete(x4, y4), y4);
//! assert_eq!(longer_concrete(x6, y6), x6);
//!
//! assert_eq!(longer_any(x4.into(), y4.into()), Some(y4.into()));
//! assert_eq!(longer_any(x4.into(), y6.into()), None);
//! ```
//!
//! Occassionally, however, one may need a data structure that may
//! sometimes contain a mix of address-families, but at other times must
//! contain only a single address-family.
//!
//! To deal with such a requirement, [`traits::AfiClass`] provides
//! further generalisation to avoid choosing between items from [`any`]
//! or [`concrete`], by defining a type-level mapping from an
//! "address-family class" to its associated type for `Address`, `Prefix`, etc.
//!
//! [`AfiClass`] is implemented for each of [`Ipv4`] and [`Ipv6`]. In this
//! context [`Ipv4`]/[`Ipv6`] can be conceptually considered to be the singleton
//! classes of address-families `{ ipv4 }` and `{ ipv6 }`.
//!
//! Additionally, the [`any::Any`] type implements [`AfiClass`], providing
//! type-level mappings to the items of the [`any`] module. [`Any`] can be
//! thought of as the class `{ ipv4, ipv6 }`.
//!
//! Various type aliases are defined at the crate root to provide easy
//! access to this mapping. In general, it is easier and clearer to use
//! [`Address<Ipv4>`] or [`Address<Any>`] than [`concrete::Address<Ipv4>`] or
//! [`any::Address`].
//!
//! #### Example
//!
//! ``` rust
//! use ip::{Address, Afi, AfiClass, Any, Ipv4};
//!
//! #[derive(Debug, PartialEq)]
//! struct Foo<A: AfiClass> {
//!     addr: Address<A>,
//! }
//!
//! impl<A: AfiClass> Foo<A> {
//!     fn new(addr: Address<A>) -> Self {
//!         Self { addr }
//!     }
//!
//!     fn into_concrete<C>(self) -> Option<Foo<C>>
//!     where
//!         C: Afi,
//!         Address<C>: TryFrom<Address<A>>,
//!     {
//!         self.addr.try_into().map(Foo::new).ok()
//!     }
//! }
//!
//! let anys: Vec<Foo<Any>> = vec![
//!     Foo {
//!         addr: Address::<Any>::Ipv4("192.0.2.1".parse().unwrap()),
//!     },
//!     Foo {
//!         addr: Address::<Any>::Ipv6("2001:db8::1".parse().unwrap()),
//!     },
//!     Foo {
//!         addr: Address::<Any>::Ipv4("198.51.100.1".parse().unwrap()),
//!     },
//! ];
//!
//! let filtered: Vec<Foo<Ipv4>> = vec![
//!     Foo {
//!         addr: "192.0.2.1".parse().unwrap(),
//!     },
//!     Foo {
//!         addr: "198.51.100.1".parse().unwrap(),
//!     },
//! ];
//!
//! assert_eq!(
//!     anys.into_iter()
//!         .filter_map(Foo::into_concrete)
//!         .collect::<Vec<Foo<Ipv4>>>(),
//!     filtered
//! );
//! ```
#![doc(html_root_url = "https://docs.rs/generic-ip/0.1.0-alpha.2")]
// clippy lints
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![allow(clippy::redundant_pub_crate)]
// rustc lints
#![warn(absolute_paths_not_starting_with_crate)]
#![warn(box_pointers)]
#![warn(deprecated_in_future)]
#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
#![warn(keyword_idents)]
#![warn(macro_use_extern_crate)]
#![warn(meta_variable_misuse)]
#![warn(missing_abi)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(noop_method_call)]
#![warn(pointer_structural_match)]
#![warn(rust_2021_incompatible_closure_captures)]
#![warn(rust_2021_incompatible_or_patterns)]
#![warn(rust_2021_prefixes_incompatible_syntax)]
#![warn(rust_2021_prelude_collisions)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(unstable_features)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]
// docs.rs build config
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// no_std support
#![no_std]
#[cfg(feature = "std")]
extern crate std;

/// Types for working with IP objects of either address family.
pub mod any;
pub use self::any::Any;

/// Types for working with IP objects of a specific address family.
pub mod concrete;
pub use self::concrete::{Ipv4, Ipv6};

/// Traits describing address family independent interfaces for IP objects.
pub mod traits;
pub use self::traits::{Afi, AfiClass};

/// Error types.
pub mod error;
pub use self::error::Error;

/// IP address formatting traits
mod fmt;

/// Parsers for IP object textual representations.
mod parser;

/// Convenience alias to name types implementing [`traits::Address`].
pub type Address<A> = <A as AfiClass>::Address;

/// Convenience alias to name types implementing [`traits::Interface`].
pub type Interface<A> = <A as AfiClass>::Interface;

/// Convenience alias to name types implementing [`traits::PrefixLength`].
pub type PrefixLength<A> = <A as AfiClass>::PrefixLength;

/// Convenience alias to name types implementing [`traits::Prefix`].
pub type Prefix<A> = <A as AfiClass>::Prefix;

/// Convenience alias to name types implementing [`traits::Netmask`].
pub type Netmask<A> = <A as AfiClass>::Netmask;

/// Convenience alias to name types implementing [`traits::Hostmask`].
pub type Hostmask<A> = <A as AfiClass>::Hostmask;

#[cfg(test)]
mod tests;
