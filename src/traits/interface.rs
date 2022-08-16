use core::fmt::Debug;
use core::hash::Hash;

use super::{Address, Prefix, PrefixLength};

/// Address-family independent interface for IP interfaces.
///
/// Methods on `Interface` types that are well defined for all address-families
/// are implemented via this trait.
///
/// In general, methods on this trait have signatures and semantics compatible
/// with methods of the same names on the [`ipnet`] network types. Where
/// there is deviation, this is noted in the method documentation.
///
/// See also [`concrete::Interface<A>`][crate::concrete::Interface] and
/// [`any::Interface`][crate::any::Interface] for address-family specific items.
pub trait Interface: Sized + Copy + Clone + Debug + Hash + PartialEq + Eq {
    /// The type of IP address respresented by this interface type.
    type Address: Address;

    /// The type used to respresent IP prefixes for this IP interface type.
    type PrefixLength: PrefixLength;

    /// The type used to respresent IP prefix lengths for this IP interface type.
    type Prefix: Prefix;

    /// Returns the network address of the IP subnet containing this interface.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{Address, Interface, Any, Ipv4, Ipv6, traits::Interface as _};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16".parse::<Interface<Ipv4>>()?.network(),
    ///     "172.16.0.0".parse::<Address<Ipv4>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8:f00::baa/48".parse::<Interface<Ipv6>>()?.network(),
    ///     "2001:db8:f00::".parse::<Address<Ipv6>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "10.255.0.10/16".parse::<Interface<Any>>()?.network(),
    ///     Address::<Any>::Ipv4("10.255.0.0".parse()?),
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn network(&self) -> Self::Address;

    /// Returns the IP address of this interface.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{Address, Interface, Any, Ipv4, Ipv6, traits::Interface as _};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16".parse::<Interface<Ipv4>>()?.addr(),
    ///     "172.16.123.123".parse::<Address<Ipv4>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8:f00::baa/48".parse::<Interface<Ipv6>>()?.addr(),
    ///     "2001:db8:f00::baa".parse::<Address<Ipv6>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "10.255.0.10/16".parse::<Interface<Any>>()?.addr(),
    ///     Address::<Any>::Ipv4("10.255.0.10".parse()?),
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn addr(&self) -> Self::Address;

    /// Returns the IP prefix containing this interface.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{Interface, Prefix, Any, Ipv4, Ipv6, traits::Interface as _};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16".parse::<Interface<Ipv4>>()?.trunc(),
    ///     "172.16.0.0/16".parse::<Prefix<Ipv4>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8:f00::baa/48".parse::<Interface<Ipv6>>()?.trunc(),
    ///     "2001:db8:f00::/48".parse::<Prefix<Ipv6>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "10.255.0.10/16".parse::<Interface<Any>>()?.trunc(),
    ///     Prefix::<Any>::Ipv4("10.255.0.0/16".parse()?),
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn trunc(&self) -> Self::Prefix;

    /// Returns the prefix length of this interface.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{Interface, PrefixLength, Any, Ipv4, Ipv6, traits::Interface as _};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16".parse::<Interface<Ipv4>>()?.prefix_len(),
    ///     PrefixLength::<Ipv4>::from_primitive(16)?,
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8:f00::baa/48".parse::<Interface<Ipv6>>()?.prefix_len(),
    ///     PrefixLength::<Ipv6>::from_primitive(48)?,
    /// );
    ///
    /// assert_eq!(
    ///     "10.255.0.10/16".parse::<Interface<Any>>()?.prefix_len(),
    ///     PrefixLength::<Ipv4>::from_primitive(16)?.into(),
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn prefix_len(&self) -> Self::PrefixLength;

    /// Returns the broadcast address of the IP subnet containing this interface.
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
    /// use ip::{Address, Interface, Any, Ipv4, Ipv6, traits::Interface as _};
    ///
    /// assert_eq!(
    ///     "172.16.123.123/16".parse::<Interface<Ipv4>>()?.broadcast(),
    ///     "172.16.255.255".parse::<Address<Ipv4>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8:f00::baa/48".parse::<Interface<Ipv6>>()?.broadcast(),
    ///     "2001:db8:f00:ffff:ffff:ffff:ffff:ffff".parse::<Address<Ipv6>>()?,
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8:dead:beef::/64".parse::<Interface<Any>>()?.broadcast(),
    ///     Address::<Any>::Ipv6("2001:db8:dead:beef:ffff:ffff:ffff:ffff".parse()?),
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn broadcast(&self) -> Self::Address;
}
