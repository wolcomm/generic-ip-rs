use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::str::FromStr;

use crate::{concrete, error::Error};

/// Address-family independent interface for IP addresses
///
/// Methods on `Address` types that are well defined for all address-families
/// are implemented via this trait.
///
/// In general, methods on this trait have signatures and semantics compatible
/// with methods of the same names on the [`std::net`] address types. Where
/// there is deviation, this is noted in the method documentation.
///
/// See also [`concrete::Address<A>`][crate::concrete::Address] and
/// [`any::Address`][crate::any::Address] for address-family specific items.
pub trait Address:
    Clone + Copy + Debug + Display + FromStr<Err = Error> + Hash + PartialEq + Eq + PartialOrd
{
    fn afi(&self) -> concrete::Afi;

    /// Returns [`true`] if this is an IPv4 broadcast address
    /// (`255.255.255.255`). Returns [`false`] for all IPv6 addresses.
    ///
    /// An IPv4 broadcast address has all octets set to `255` as defined in
    /// [RFC 919].
    ///
    /// [RFC 919]: https://tools.ietf.org/html/rfc919
    ///
    /// # [`std::net`] Compatibility
    ///
    /// This method is defined on [`Ipv4Addr`][std::net::Ipv4Addr] but not on
    /// [`Ipv6Addr`][std::net::Ipv6Addr] or [`IpAddr`][std::net::IpAddr].
    ///
    /// This implementation provides the method for addresses of all families,
    /// but always returns [`false`] for IPv6 addresses.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_broadcast = "255.255.255.255".parse::<Address<Ipv4>>()?;
    /// let v4_unicast = "203.0.113.1".parse::<Address<Ipv4>>()?;
    /// let v6_all_ones = "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff".parse::<Address<Ipv6>>()?;
    ///
    /// assert_eq!(v4_broadcast.is_broadcast(), true);
    /// assert_eq!(v4_unicast.is_broadcast(), false);
    /// assert_eq!(v6_all_ones.is_broadcast(), false);
    /// assert_eq!(Address::<Any>::Ipv4(v4_broadcast).is_broadcast(), true);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn is_broadcast(&self) -> bool;

    /// Returns [`true`] if the address is link-local.
    ///
    /// Link-local addresses are those within the prefixes `169.254.0.0/16` for
    /// IPv4 and `fe80::/10` for IPv6.
    ///
    /// See also [`Address::<Ipv6>::unicast_link_local`][crate::concrete::Address::is_unicast_link_local].
    ///
    /// See [RFC 3927] and [RFC 4291].
    ///
    /// [RFC 3927]: https://tools.ietf.org/html/rfc3927
    /// [RFC 4291]: https://tools.ietf.org/html/rfc4291
    ///
    /// # [`std::net`] Compatibility
    ///
    /// This method is defined on [`Ipv4Addr`][std::net::Ipv4Addr] but not on
    /// [`Ipv6Addr`][std::net::Ipv6Addr] or [`IpAddr`][std::net::IpAddr].
    ///
    /// This implementation provides the method for addresses of all families.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_link_local = "169.254.254.1".parse::<Address<Ipv4>>()?;
    /// let v6_link_local = "fe80::1".parse::<Address<Ipv6>>()?;
    /// let v4_unicast = "203.0.113.1".parse::<Address<Ipv4>>()?;
    ///
    /// assert_eq!(v4_link_local.is_link_local(), true);
    /// assert_eq!(v6_link_local.is_link_local(), true);
    /// assert_eq!(v4_unicast.is_link_local(), false);
    /// assert_eq!(Address::<Ipv6>::LOCALHOST.is_link_local(), false);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn is_link_local(&self) -> bool;

    /// Returns [`true`] if this is a private IPv4 address. Returns [`false`]
    /// for all IPv6 addresses.
    ///
    /// The private address ranges are defined in [RFC 1918] and include:
    ///
    ///  - `10.0.0.0/8`
    ///  - `172.16.0.0/12`
    ///  - `192.168.0.0/16`
    ///
    /// [RFC 1918]: https://tools.ietf.org/html/rfc1918
    ///
    /// # [`std::net`] Compatibility
    ///
    /// This method is defined on [`Ipv4Addr`][std::net::Ipv4Addr] but not on
    /// [`Ipv6Addr`][std::net::Ipv6Addr] or [`IpAddr`][std::net::IpAddr].
    ///
    /// This implementation provides the method for addresses of all families,
    /// but always returns [`false`] for IPv6 addresses.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_private = "172.18.0.1".parse::<Address<Ipv4>>()?;
    /// let v4_unicast = "203.0.113.1".parse::<Address<Ipv4>>()?;
    ///
    /// assert_eq!(v4_private.is_private(), true);
    /// assert_eq!(v4_unicast.is_private(), false);
    /// assert_eq!(Address::<Ipv6>::LOCALHOST.is_private(), false);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn is_private(&self) -> bool;

    /// Returns [`true`] if this address is an IPv4 address reserved by IANA
    /// for future use. [RFC 1112] defines the block of reserved addresses
    /// as `240.0.0.0/4`. This range normally includes the broadcast address
    /// `255.255.255.255`, but this implementation explicitly excludes it,
    /// since it is obviously not reserved for future use.
    ///
    /// [RFC 1112]: https://tools.ietf.org/html/rfc1112
    ///
    /// # [`std::net`] Compatibility
    ///
    /// This method is defined on [`Ipv4Addr`][std::net::Ipv4Addr] but not on
    /// [`Ipv6Addr`][std::net::Ipv6Addr] or [`IpAddr`][std::net::IpAddr].
    ///
    /// This implementation provides the method for addresses of all families,
    /// but always returns [`false`] for IPv6 addresses.
    ///
    /// # Warning
    ///
    /// As IANA assigns new addresses, this method will be updated. This may
    /// result in non-reserved addresses being treated as reserved in code that
    /// relies on an outdated version of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_reserved = "240.0.0.1".parse::<Address<Ipv4>>()?;
    /// let v4_unicast = "203.0.113.1".parse::<Address<Ipv4>>()?;
    ///
    /// assert_eq!(v4_reserved.is_reserved(), true);
    /// assert_eq!(v4_unicast.is_reserved(), false);
    /// assert_eq!(Address::<Ipv4>::BROADCAST.is_reserved(), false);
    /// # Ok::<(), ip::Error>(())
    /// ```
    // TODO: unstable
    fn is_reserved(&self) -> bool;

    /// Returns [`true`] if this address is part of the IPv4 Shared Address
    /// Space defined in [RFC 6598] (`100.64.0.0/10`).
    ///
    /// [RFC 6598]: https://tools.ietf.org/html/rfc6598
    ///
    /// # [`std::net`] Compatibility
    ///
    /// This method is defined on [`Ipv4Addr`][std::net::Ipv4Addr] but not on
    /// [`Ipv6Addr`][std::net::Ipv6Addr] or [`IpAddr`][std::net::IpAddr].
    ///
    /// This implementation provides the method for addresses of all families,
    /// but always returns [`false`] for IPv6 addresses.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_shared = "100.72.1.1".parse::<Address<Ipv4>>()?;
    /// let v4_unicast = "192.0.2.1".parse::<Address<Ipv4>>()?;
    /// let v6_ula = "fc00::1".parse::<Address<Any>>()?;
    ///
    /// assert_eq!(v4_shared.is_shared(), true);
    /// assert_eq!(v4_unicast.is_shared(), false);
    /// assert_eq!(v6_ula.is_shared(), false);
    /// # Ok::<(), ip::Error>(())
    /// ```
    // TODO: unstable
    fn is_shared(&self) -> bool;

    /// Returns [`true`] if this address is part of the IPv4 "This network"
    /// prefix defined in [RFC 791] (`0.0.0.0/8`).
    ///
    /// [RFC 791]: https://tools.ietf.org/html/rfc791
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_thisnet = "0.255.255.255".parse::<Address<Ipv4>>()?;
    ///
    /// assert_eq!(v4_thisnet.is_thisnet(), true);
    /// assert_eq!(Address::<Ipv6>::UNSPECIFIED.is_thisnet(), false);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn is_thisnet(&self) -> bool;

    /// Returns [`true`] if this is an address reserved for network device
    /// benchmarking:
    ///
    /// - IPv4: `198.18.0.0/15` ([RFC 2544])
    /// - IPv6: `2001:2::/48` ([RFC 5180])
    ///
    /// # Errata
    ///
    /// The IPv4 benchmarking range is mistakenly defined in [RFC 2544] as
    /// `192.18.0.0` through `198.19.255.255`, corrected in [errata 423].
    ///
    /// The IPv6 benchmarking range is mistakenly defined in [RFC 5180] as
    /// `2001:200::/48`, corrected in [errata 1752].
    ///
    /// [RFC 2544]: https://tools.ietf.org/html/rfc2544
    /// [RFC 5180]: https://tools.ietf.org/html/rfc5180
    /// [errata 423]: https://www.rfc-editor.org/errata/eid423
    /// [errata 1752]: https://www.rfc-editor.org/errata/eid1752
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_benchmarking = "198.19.0.1".parse::<Address<Ipv4>>()?;
    /// let v6_benchmarking = "2001:2::1".parse::<Address<Ipv6>>()?;
    ///
    /// assert_eq!(v4_benchmarking.is_benchmarking(), true);
    /// assert_eq!(v6_benchmarking.is_benchmarking(), true);
    /// # Ok::<(), ip::Error>(())
    /// ```
    // TODO: unstable
    fn is_benchmarking(&self) -> bool;

    /// Returns [`true`] if this is an address reserved for documentation:
    ///
    /// - IPv4 (defined in [RFC 5737]):
    ///     - `192.0.2.0/24` (`TEST-NET-1`)
    ///     - `198.51.100.0/24` (`TEST-NET-2`)
    ///     - `203.0.113.0/24` (`TEST-NET-3`)
    /// - IPv6: `2001:db8::/32` (defined in [RFC 3849])
    ///
    /// [RFC 3849]: https://tools.ietf.org/html/rfc3849
    /// [RFC 5737]: https://tools.ietf.org/html/rfc5737
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_test_net_2 = "198.51.100.1".parse::<Address<Ipv4>>()?;
    /// let v6_documentation = "2001:db8::1".parse::<Address<Ipv6>>()?;
    ///
    /// assert_eq!(v4_test_net_2.is_documentation(), true);
    /// assert_eq!(v6_documentation.is_documentation(), true);
    /// # Ok::<(), ip::Error>(())
    /// ```
    // TODO: unstable
    fn is_documentation(&self) -> bool;

    /// Returns [`true`] if this address appears to be globally reachable.
    ///
    /// # IPv4
    ///
    /// An IPv4 address is considered globally reachable unless it is contained
    /// in a prefix appearing in the [IANA IPv4 Special-Purpose Address
    /// Registry], with the value "False" in the column "Globally Reachable"
    ///
    /// # IPv6
    ///
    /// An IPv6 address is considered globally reachable unless it is contained
    /// in a prefix appearing in the [IANA IPv6 Special-Purpose Address
    /// Registry], with the value "False" in the column "Globally Reachable"
    ///
    /// [IANA IPv4 Special-Purpose Address Registry]:
    /// https://www.iana.org/assignments/iana-ipv4-special-registry/iana-ipv4-special-registry.xhtml
    /// [IANA IPv6 Special-Purpose Address Registry]:
    /// https://www.iana.org/assignments/iana-ipv6-special-registry/iana-ipv6-special-registry.xhtml
    ///
    /// # [`std::net`] Compatibility
    ///
    /// The implementation of this method on the items of [`std::net`] do not
    /// consider the scope of IPv4 multicast addresses, all of which return
    /// [`true`].
    ///
    /// This implementation correctly returns [`true`] or [`false`] for IPv4
    /// multicast addresses, according to their designated scope.
    // TODO: Add RFC references
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// // private addresses are not global
    /// assert_eq!("10.254.0.0".parse::<Address<Ipv4>>()?.is_global(), false);
    /// assert_eq!("192.168.10.65".parse::<Address<Ipv4>>()?.is_global(), false);
    /// assert_eq!("172.16.10.65".parse::<Address<Ipv4>>()?.is_global(), false);
    ///
    /// // ULA addresses are not global
    /// assert_eq!("fc00::1".parse::<Address<Ipv6>>()?.is_global(), false);
    ///
    /// // the 0.0.0.0/8 block is not global
    /// assert_eq!("0.1.2.3".parse::<Address<Ipv4>>()?.is_global(), false);
    /// // in particular, the unspecified addresses are not global
    /// assert_eq!(Address::<Ipv4>::UNSPECIFIED.is_global(), false);
    /// assert_eq!(Address::<Ipv6>::UNSPECIFIED.is_global(), false);
    ///
    /// // the loopback address is not global
    /// assert_eq!(Address::<Ipv4>::LOCALHOST.is_global(), false);
    /// assert_eq!(Address::<Ipv6>::LOCALHOST.is_global(), false);
    ///
    /// // link local addresses are not global
    /// assert_eq!("169.254.45.1".parse::<Address<Ipv4>>()?.is_global(), false);
    /// assert_eq!("fe80::1".parse::<Address<Ipv6>>()?.is_global(), false);
    ///
    /// // the broadcast address is not global
    /// assert_eq!(Address::<Ipv4>::BROADCAST.is_global(), false);
    ///
    /// // the address space designated for documentation is not global
    /// assert_eq!("192.0.2.255".parse::<Address<Ipv4>>()?.is_global(), false);
    /// assert_eq!("198.51.100.65".parse::<Address<Ipv4>>()?.is_global(), false);
    /// assert_eq!("203.0.113.6".parse::<Address<Ipv4>>()?.is_global(), false);
    /// assert_eq!("2001:db8::1".parse::<Address<Ipv6>>()?.is_global(), false);
    ///
    /// // shared addresses are not global
    /// assert_eq!("100.100.0.0".parse::<Address<Ipv4>>()?.is_global(), false);
    ///
    /// // addresses reserved for protocol assignment are not global in general
    /// assert_eq!("192.0.0.0".parse::<Address<Ipv4>>()?.is_global(), false);
    /// assert_eq!("192.0.0.255".parse::<Address<Ipv4>>()?.is_global(), false);
    /// assert_eq!("2001:100::1".parse::<Address<Ipv6>>()?.is_global(), false);
    /// // but exceptions exist
    /// assert_eq!("192.0.0.9".parse::<Address<Ipv4>>()?.is_global(), true);
    /// assert_eq!("2001:20::1".parse::<Address<Ipv6>>()?.is_global(), true);
    ///
    /// // addresses reserved for future use are not global
    /// assert_eq!("250.10.20.30".parse::<Address<Ipv4>>()?.is_global(), false);
    ///
    /// // addresses reserved for network devices benchmarking are not global
    /// assert_eq!("198.18.0.0".parse::<Address<Ipv4>>()?.is_global(), false);
    /// assert_eq!("2001:2::1".parse::<Address<Ipv6>>()?.is_global(), false);
    ///
    /// // multicast addresses are global if so permitted by their scope
    /// assert_eq!("224.0.0.1".parse::<Address<Ipv4>>()?.is_global(), false);
    /// assert_eq!("239.0.0.1".parse::<Address<Ipv4>>()?.is_global(), false);
    /// assert_eq!("ff08::1".parse::<Address<Ipv6>>()?.is_global(), false);
    /// assert_eq!("224.0.1.1".parse::<Address<Ipv4>>()?.is_global(), true);
    /// assert_eq!("ff0e::1".parse::<Address<Ipv6>>()?.is_global(), true);
    ///
    /// // All the other addresses are global
    /// assert_eq!("1.1.1.1".parse::<Address<Ipv4>>()?.is_global(), true);
    /// assert_eq!(
    ///     "2606:4700:4700::1111".parse::<Address<Ipv6>>()?.is_global(),
    ///     true
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    // TODO: unstable
    fn is_global(&self) -> bool;

    /// Returns [`true`] if this is a loopback address.
    ///
    /// - IPv4: `127.0.0.0/8` (defined in [RFC 1122]):
    /// - IPv6: `::1` (defined in [RFC 4291])
    ///
    /// [RFC 1122]: https://tools.ietf.org/html/rfc1122
    /// [RFC 4291]: https://tools.ietf.org/html/rfc4291
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_loopback = "127.0.0.53".parse::<Address<Ipv4>>()?;
    /// let v6_loopback = "::1".parse::<Address<Ipv6>>()?;
    ///
    /// assert_eq!(v4_loopback.is_loopback(), true);
    /// assert_eq!(v6_loopback.is_loopback(), true);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn is_loopback(&self) -> bool;

    /// Returns [`true`] if this is a multicast address.
    ///
    /// - IPv4: `224.0.0.0/8` (defined in [RFC 5771]):
    /// - IPv6: `ff00::/8` (defined in [RFC 4291])
    ///
    /// [RFC 5771]: https://tools.ietf.org/html/rfc5771
    /// [RFC 4291]: https://tools.ietf.org/html/rfc4291
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_multicast = "224.254.0.0".parse::<Address<Ipv4>>()?;
    /// let v4_unicast = "172.16.10.65".parse::<Address<Ipv4>>()?;
    /// let v6_multicast = "ff01::1".parse::<Address<Ipv6>>()?;
    ///
    /// assert_eq!(v4_multicast.is_multicast(), true);
    /// assert_eq!(v6_multicast.is_multicast(), true);
    /// assert_eq!(v4_unicast.is_multicast(), false);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn is_multicast(&self) -> bool;

    /// Returns [`true`] for the special "unspecified" address, also called
    /// "this host on this network" in IPv4.
    ///
    /// - IPv4: `0.0.0.0` (defined in [RFC 1122]):
    /// - IPv6: `::` (defined in [RFC 4291])
    ///
    /// [RFC 1122]: https://tools.ietf.org/html/rfc1122
    /// [RFC 4291]: https://tools.ietf.org/html/rfc4291
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_unspecified = "0.0.0.0".parse::<Address<Ipv4>>()?;
    /// let v6_unspecified = "::".parse::<Address<Ipv6>>()?;
    ///
    /// assert_eq!(v4_unspecified.is_unspecified(), true);
    /// assert_eq!(v6_unspecified.is_unspecified(), true);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn is_unspecified(&self) -> bool;

    /// Returns [`true`] if this is an IPv6 unique local address (`fc00::/7`
    /// [RFC 4193]).
    ///
    /// # [`std::net`] Compatibility
    ///
    /// This method is defined on [`Ipv6Addr`][std::net::Ipv6Addr] but not on
    /// [`Ipv4Addr`][std::net::Ipv4Addr] or [`IpAddr`][std::net::IpAddr].
    ///
    /// Unique local addresses are specific to IPv6. The closest analogue for
    /// IPv4 is the "private" address space: see
    /// [`is_private()`][Self::is_private()].
    ///
    /// This implementation provides the method for addresses of all families,
    /// but always returns [`false`] for IPv4 addresses.
    ///
    /// [RFC 4193]: https://tools.ietf.org/html/rfc4193
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v6_ula = "fc01::1".parse::<Address<Ipv6>>()?;
    /// let v6_doc = "2001:db8::1".parse::<Address<Ipv6>>()?;
    /// let v4_private = "192.168.1.1".parse::<Address<Ipv4>>()?;
    ///
    /// assert_eq!(v6_ula.is_unique_local(), true);
    /// assert_eq!(v6_doc.is_unique_local(), false);
    /// assert_eq!(v4_private.is_unique_local(), false);
    /// # Ok::<(), ip::Error>(())
    /// ```
    // TODO: unstable
    fn is_unique_local(&self) -> bool;

    /// Returns [`true`] if this is neither a multicase nor a broadcast
    /// address. See [`is_multicast()`][Self::is_multicast()] and
    /// [`is_broadcast()`][Self::is_broadcast()].
    ///
    /// # [`std::net`] Compatibility
    ///
    /// This method is defined on [`Ipv6Addr`][std::net::Ipv6Addr] but not on
    /// [`Ipv4Addr`][std::net::Ipv4Addr] or [`IpAddr`][std::net::IpAddr].
    ///
    /// This implementation provides the method for addresses of all families.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v6_unicast = "2001:db8::1".parse::<Address<Ipv6>>()?;
    /// let v6_multicast = "ffaa::1".parse::<Address<Ipv6>>()?;
    /// let v4_unicast = "192.168.1.1".parse::<Address<Ipv4>>()?;
    /// let v4_multicast = "239.0.0.1".parse::<Address<Ipv4>>()?;
    /// let v4_broadcast = "255.255.255.255".parse::<Address<Ipv4>>()?;
    ///
    /// assert_eq!(v6_unicast.is_unicast(), true);
    /// assert_eq!(v6_multicast.is_unicast(), false);
    /// assert_eq!(v4_unicast.is_unicast(), true);
    /// assert_eq!(v4_multicast.is_unicast(), false);
    /// assert_eq!(v4_broadcast.is_unicast(), false);
    /// # Ok::<(), ip::Error>(())
    /// ```
    // TODO: unstable
    fn is_unicast(&self) -> bool {
        !(self.is_multicast() || self.is_broadcast())
    }

    /// Returns [`true`] if this is a unicast address that is gloablly
    /// routable. See [`is_unicast()`][Self::is_unicast()] and
    /// [`is_global()`][Self::is_global()].
    ///
    /// # [`std::net`] Compatibility
    ///
    /// This method is defined on [`Ipv6Addr`][std::net::Ipv6Addr] but not on
    /// [`Ipv4Addr`][std::net::Ipv4Addr] or [`IpAddr`][std::net::IpAddr].
    ///
    /// This implementation provides the method for addresses of all families.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::Address as _, Address, Any, Ipv4, Ipv6};
    ///
    /// let v4_unicast_global = "1.1.1.1".parse::<Address<Ipv4>>()?;
    /// let v4_unicast_private = "192.168.1.1".parse::<Address<Ipv4>>()?;
    /// let v4_multicast_global = "225.0.0.1".parse::<Address<Ipv4>>()?;
    /// let v6_unicast_global = "2606:4700:4700::1111".parse::<Address<Ipv6>>()?;
    /// let v6_unicast_doc = "2001:db8::1".parse::<Address<Ipv6>>()?;
    /// let v6_multicast_global = "ff0e::1".parse::<Address<Ipv6>>()?;
    ///
    /// assert_eq!(v4_unicast_global.is_unicast_global(), true);
    /// assert_eq!(v4_unicast_private.is_unicast_global(), false);
    /// assert_eq!(v4_multicast_global.is_unicast_global(), false);
    /// assert_eq!(v6_unicast_global.is_unicast_global(), true);
    /// assert_eq!(v6_unicast_doc.is_unicast_global(), false);
    /// assert_eq!(v6_multicast_global.is_unicast_global(), false);
    /// # Ok::<(), ip::Error>(())
    /// ```
    // TODO: unstable
    fn is_unicast_global(&self) -> bool {
        self.is_unicast() && self.is_global()
    }
}
