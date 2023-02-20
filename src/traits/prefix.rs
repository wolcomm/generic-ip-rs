use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::ops::RangeInclusive;
use core::str::FromStr;

use crate::{
    concrete,
    error::{self, Error},
};

use super::{Address, Mask};

/// Address-family independent interface for IP prefixes
///
/// Methods on `Prefix` types that are well defined for all address-families
/// are implemented via this trait.
///
/// In general, methods on this trait have signatures and semantics compatible
/// with methods of the same names on the [`ipnet`] network types. Where
/// there is deviation, this is noted in the method documentation.
///
/// See also [`concrete::Prefix<A>`][crate::concrete::Prefix] and
/// [`any::Prefix`][crate::any::Prefix] for address-family specific items.
pub trait Prefix:
    Sized + Copy + Clone + Debug + Display + FromStr<Err = Error> + Hash + PartialEq + Eq + PartialOrd
{
    /// The type of IP address respresented by this prefix type.
    type Address: Address;

    /// The type used to respresent lengths for this IP prefix type.
    type Length: Length;

    /// The type of IP hostmask corresponding to this prefix type.
    type Hostmask: Mask;

    /// The type of IP netmask corresponding to this prefix type.
    type Netmask: Mask;

    type Subprefixes: Iterator<Item = Self>;
    // TODO:
    // type Hosts: Iterator<Item = Self::Address>;

    /// Returns the network address of the IP subnet respresented by this
    /// prefix.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Prefix as _, Address, Any, Ipv4, Ipv6, Prefix};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16".parse::<Prefix<Ipv4>>()?.network(),
    ///     "172.16.0.0".parse::<Address<Ipv4>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8:f00::/48".parse::<Prefix<Ipv6>>()?.network(),
    ///     "2001:db8:f00::".parse::<Address<Ipv6>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "10.255.0.0/16".parse::<Prefix<Any>>()?.network(),
    ///     Address::<Any>::Ipv4("10.255.0.0".parse()?),
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn network(&self) -> Self::Address;

    /// Returns the hostmask of the IP subnet respresented by this prefix.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Prefix as _, Ipv4, Prefix};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16"
    ///         .parse::<Prefix<Ipv4>>()?
    ///         .hostmask()
    ///         .to_string(),
    ///     "0.0.255.255",
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn hostmask(&self) -> Self::Hostmask;

    /// Returns the netmask of the IP subnet respresented by this prefix.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Prefix as _, Ipv4, Prefix};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16"
    ///         .parse::<Prefix<Ipv4>>()?
    ///         .netmask()
    ///         .to_string(),
    ///     "255.255.0.0",
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn netmask(&self) -> Self::Netmask;

    /// Returns the maximum valid prefix length for prefixes of this type.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Prefix as _, Any, Ipv4, Ipv6, Prefix, PrefixLength};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16".parse::<Prefix<Any>>()?.max_prefix_len(),
    ///     PrefixLength::<Ipv4>::MAX.into(),
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8:f00::/48".parse::<Prefix<Any>>()?.max_prefix_len(),
    ///     PrefixLength::<Ipv6>::MAX.into(),
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn max_prefix_len(&self) -> Self::Length;

    /// Returns the length of this prefix.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Prefix as _, Ipv4, Ipv6, Prefix, PrefixLength};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16".parse::<Prefix<Ipv4>>()?.prefix_len(),
    ///     PrefixLength::<Ipv4>::from_primitive(16)?,
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8:f00::/48".parse::<Prefix<Ipv6>>()?.prefix_len(),
    ///     PrefixLength::<Ipv6>::from_primitive(48)?,
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn prefix_len(&self) -> Self::Length;

    /// Returns the broadcast address of the IP subnet respresented by this
    /// prefix.
    ///
    /// # [`ipnet`] Compatibility
    ///
    /// The term "broadcast address" has no meaning when applied to IPv6
    /// subnets. However, for compatibility with [`ipnet::Ipv6Net`], this
    /// method will return the last address covered by the prefix in all
    /// cases.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Prefix as _, Address, Any, Ipv4, Ipv6, Prefix};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16".parse::<Prefix<Ipv4>>()?.broadcast(),
    ///     "172.16.255.255".parse::<Address<Ipv4>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8:f00::/48".parse::<Prefix<Ipv6>>()?.broadcast(),
    ///     "2001:db8:f00:ffff:ffff:ffff:ffff:ffff".parse::<Address<Ipv6>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8:dead:beef::/64"
    ///         .parse::<Prefix<Any>>()?
    ///         .broadcast(),
    ///     Address::<Any>::Ipv6("2001:db8:dead:beef:ffff:ffff:ffff:ffff".parse()?),
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn broadcast(&self) -> Self::Address;

    /// Returns the prefix of length `self.prefix_len() + 1` containing `self`,
    /// if it exists.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Prefix as _, Ipv4, Ipv6, Prefix};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16".parse::<Prefix<Ipv4>>()?.supernet(),
    ///     Some("172.16.0.0/15".parse()?),
    /// );
    ///
    /// assert_eq!("::/0".parse::<Prefix<Ipv6>>()?.supernet(), None,);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn supernet(&self) -> Option<Self>;

    /// Returns [`true`] if `self` and `other` share the same immediate
    /// supernet. See also [`supernet()`][Self::supernet()].
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Prefix as _, Ipv6, Prefix};
    ///
    /// let a: Prefix<Ipv6> = "2001:db8:a::/48".parse()?;
    /// let b: Prefix<Ipv6> = "2001:db8:b::/48".parse()?;
    /// let c: Prefix<Ipv6> = "2001:db8:c::/48".parse()?;
    ///
    /// assert!(a.is_sibling(&b));
    /// assert!(!b.is_sibling(&c));
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn is_sibling(&self, other: &Self) -> bool;

    /// Returns [`true`] if `self` contains `other` (in the set-theoretic
    /// sense).
    ///
    /// # Note
    ///
    /// This method is defined for all types `T` where `Self: PartialOrd<T>`.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Prefix as _, Address, Ipv6, Prefix};
    ///
    /// let a: Prefix<Ipv6> = "2001:db8:a::/48".parse()?;
    /// let b: Prefix<Ipv6> = "2001:db8:a:b::/64".parse()?;
    /// let c: Prefix<Ipv6> = "2001:db8::/32".parse()?;
    ///
    /// assert!(a.contains(&b));
    /// assert!(!a.contains(&c));
    ///
    /// let x: Address<Ipv6> = "2001:db8:a::1".parse()?;
    /// let y: Address<Ipv6> = "2001:db8::1".parse()?;
    ///
    /// assert!(a.contains(&x));
    /// assert!(!a.contains(&y));
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn contains<T>(&self, other: &T) -> bool
    where
        Self: PartialOrd<T>,
    {
        self.ge(other)
    }

    // TODO:
    // #[cfg(feature = "std")]
    // fn aggregate(networks: &std::vec::Vec<Self>) -> std::vec::Vec<Self>;
    // fn hosts(&self) -> Self::Hosts;

    /// Returns an iterator over the subprefixes of `self` of length
    /// `new_prefix_len`.
    ///
    /// # Errors
    ///
    /// An error is returned if `new_prefix_len < self.prefix_len()`.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Prefix as _, Prefix, Ipv4, Ipv6};
    ///
    /// let prefix: Prefix<Ipv4> = "192.0.2.0/24".parse()?;
    /// let new_length = PrefixLength::<Ipv4>::from_primitive(26)?;
    ///
    /// assert_eq!(ipv4_prefix.subprefixes(new_length)?.count(), 4);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn subprefixes(&self, new_prefix_len: Self::Length) -> Result<Self::Subprefixes, Error>;

    fn afi(&self) -> concrete::Afi {
        self.network().afi()
    }

    #[cfg(feature = "std")]
    #[allow(box_pointers)]
    fn new_prefix_length(&self, length: u8) -> Result<Self::Length, Error> {
        self.afi()
            .new_prefix_length(length)
            .and_then(|l| {
                l.downcast()
                    .map_err(|_| Error::new(error::Kind::Downcast, None::<&str>, None))
            })
            .map(|l| *l)
    }
}

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
    /// An [`Error`] of kind [`error::Kind::PrefixLength`] is returned if
    /// `self` is maximally-valued.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::{Prefix as _, PrefixLength as _}, Ipv4, Ipv6, Prefix, PrefixLength};
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
    /// An [`Error`] of kind [`error::Kind::PrefixLength`] is returned if
    /// `self` is zero-valued.
    fn decrement(self) -> Result<Self, Error>;
}

/// Address-family independent interface for IP prefix ranges.
///
/// See also [`concrete::PrefixRange<A>`][crate::concrete::PrefixRange] and
/// [`any::PrefixRange`][crate::any::PrefixRange] for address-family specific
/// items.
pub trait Range: Clone + Debug + Display + Hash + PartialEq + Eq + PartialOrd {
    type Prefix: Prefix<Length = Self::Length>;
    type Length: Length;

    fn prefix(&self) -> Self::Prefix;
    fn lower(&self) -> Self::Length;
    fn upper(&self) -> Self::Length;
    fn with_length_range(self, len_range: RangeInclusive<Self::Length>) -> Option<Self>;

    fn or_longer(self) -> Self {
        let lower = self.lower();
        let upper = self.prefix().max_prefix_len();
        // OK to unwrap here as we can guarantee that `len_range` in non-empty.
        self.with_length_range(lower..=upper).unwrap()
    }

    fn or_longer_excl(self) -> Option<Self> {
        let lower = self.lower().increment().ok()?;
        let upper = self.prefix().max_prefix_len();
        self.with_length_range(lower..=upper)
    }

    fn with_length(self, len: Self::Length) -> Option<Self> {
        self.with_length_range(len..=len)
    }

    fn afi(&self) -> concrete::Afi {
        self.prefix().afi()
    }

    #[cfg(feature = "std")]
    #[allow(box_pointers)]
    fn new_prefix_length(&self, length: u8) -> Result<Self::Length, Error> {
        self.prefix().new_prefix_length(length)
    }
}
