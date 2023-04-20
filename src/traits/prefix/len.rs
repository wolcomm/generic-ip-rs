use core::fmt::{Debug, Display};
use core::hash::Hash;

use crate::error::Error;

/// Address-family independent interface for IP prefix-lengths
///
/// See also [`concrete::PrefixLength<A>`][crate::concrete::PrefixLength] and
/// [`any::PrefixLength`][crate::any::PrefixLength] for address-family specific
/// items.
pub trait Length:
    Copy + Clone + Debug + Display + Hash + PartialEq + Eq + PartialOrd + 'static
{
    /// Returns a new `Self` that is one greater than `self` unless `self` is
    /// already the maximum possible value.
    ///
    /// # Errors
    ///
    /// An [`Error`] of kind
    /// [`Kind::PrefixLength`][crate::error::Kind::PrefixLength] is returned if
    /// `self` is maximally-valued.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{
    ///     traits::{Prefix as _, PrefixLength as _},
    ///     Ipv4, Ipv6, Prefix, PrefixLength,
    /// };
    ///
    /// let ipv4_default: Prefix<Ipv4> = "0.0.0.0/0".parse()?;
    /// let ipv6_host: Prefix<Ipv6> = "2001:db8::1/128".parse()?;
    ///
    /// assert_eq!(
    ///     ipv4_default.prefix_len().increment()?,
    ///     PrefixLength::<Ipv4>::from_primitive(1)?,
    /// );
    /// assert!(ipv4_default.prefix_len().decrement().is_err());
    /// assert_eq!(
    ///     ipv6_host.prefix_len().decrement()?,
    ///     PrefixLength::<Ipv6>::from_primitive(127)?,
    /// );
    /// assert!(ipv6_host.prefix_len().increment().is_err());
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn increment(self) -> Result<Self, Error>;

    /// Returns a new `Self` that is one less than `self` unless `self` is
    /// already the minimum possible value.
    ///
    /// # Errors
    ///
    /// An [`Error`] of kind
    /// [`Kind::PrefixLength`][crate::error::Kind::PrefixLength] is returned if
    /// `self` is zero-valued.
    fn decrement(self) -> Result<Self, Error>;
}
