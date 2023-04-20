use super::Address;
use crate::{
    any,
    concrete::{Ipv4, Ipv6},
    traits::{primitive::IntoIpv6Segments as _, Address as _, Afi},
};

// TODO: make methods `const fn`
impl Address<Ipv6> {
    /// Returns [`true`] if the address is unicast link local.
    ///
    /// This method is provided for compatibility with [`std::net::Ipv6Addr`],
    /// and is just a wrapper around
    /// [`Address::is_link_local()`][crate::traits::Address::is_link_local()].
    #[must_use]
    pub fn is_unicast_link_local(&self) -> bool {
        self.is_link_local()
    }

    /// Returns the [`Ipv6MulticastScope`][MulticastScope] variant of the
    /// address if the address is a multicast address, or [`None`]
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{concrete::Ipv6MulticastScope, Address, Ipv6};
    ///
    /// let ipv6_site_local_multicast = "ff05::1".parse::<Address<Ipv6>>()?;
    /// assert_eq!(
    ///     ipv6_site_local_multicast.multicast_scope(),
    ///     Some(Ipv6MulticastScope::SiteLocal),
    /// );
    ///
    /// let ipv6_global_multicast = "ff0e::1".parse::<Address<Ipv6>>()?;
    /// assert_eq!(
    ///     ipv6_global_multicast.multicast_scope(),
    ///     Some(Ipv6MulticastScope::Global),
    /// );
    ///
    /// let ipv6_unicast = "2001:db8::1".parse::<Address<Ipv6>>()?;
    /// assert_eq!(ipv6_unicast.multicast_scope(), None,);
    /// # Ok::<(), ip::Error>(())
    /// ```
    #[allow(clippy::match_same_arms)]
    #[must_use]
    pub fn multicast_scope(&self) -> Option<MulticastScope> {
        if self.is_multicast() {
            match self.octets()[1] & 0x0f {
                0x0 => Some(MulticastScope::Reserved),
                0x1 => Some(MulticastScope::InterfaceLocal),
                0x2 => Some(MulticastScope::LinkLocal),
                0x3 => Some(MulticastScope::RealmLocal),
                0x4 => Some(MulticastScope::AdminLocal),
                0x5 => Some(MulticastScope::SiteLocal),
                0x6..=0x7 => Some(MulticastScope::Unassigned),
                0x8 => Some(MulticastScope::OrganizationLocal),
                0x9..=0xd => Some(MulticastScope::Unassigned),
                0xe => Some(MulticastScope::Global),
                0xf => Some(MulticastScope::Reserved),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    /// Returns a big-endian [`[u16; 8]`] representing the segments of the
    /// address.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{Address, Ipv6};
    ///
    /// assert_eq!(
    ///     "2001:db8:f00::1".parse::<Address<Ipv6>>()?.segments(),
    ///     [0x2001, 0xdb8, 0xf00, 0x0, 0x0, 0x0, 0x0, 0x1],
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    #[must_use]
    pub fn segments(&self) -> [u16; 8] {
        self.into_primitive().into_segments()
    }

    // TODO: move to `traits::Address`
    /// Convert the address to its canonical representation as an
    /// [`any::Address`], by converting an IPv4-mapped address to a
    /// [`any::Address::Ipv4`], and returning an [`any::Address::Ipv6`]
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{Address, Any, Ipv6};
    ///
    /// let ipv4_mapped = "::ffff:192.168.1.1".parse::<Address<Ipv6>>()?;
    /// assert_eq!(
    ///     ipv4_mapped.to_canonical(),
    ///     "192.168.1.1".parse::<Address<Any>>()?,
    /// );
    ///
    /// let ipv6_unicast = "2001:db8::1".parse::<Address<Ipv6>>()?;
    /// assert_eq!(
    ///     ipv6_unicast.to_canonical(),
    ///     Address::<Any>::Ipv6(ipv6_unicast),
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    #[allow(clippy::wrong_self_convention)]
    #[allow(clippy::option_if_let_else)]
    #[must_use]
    pub fn to_canonical(&self) -> any::Address {
        if let Some(ipv4_addr) = self.to_ipv4_mapped() {
            any::Address::Ipv4(ipv4_addr)
        } else {
            any::Address::Ipv6(*self)
        }
    }

    /// Returns the embedded [`Address<Ipv4>`] in an IPv4-compatible or
    /// IPv4-mapped [`Address<Ipv6>`], or [`None`] otherwise.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{Address, Ipv6};
    ///
    /// assert_eq!(
    ///     "::192.168.1.1"
    ///         .parse::<Address<Ipv6>>()?
    ///         .to_ipv4()
    ///         .map(|ipv4| ipv4.octets()),
    ///     Some([192, 168, 1, 1]),
    /// );
    ///
    /// assert_eq!("2001:db8::1".parse::<Address<Ipv6>>()?.to_ipv4(), None,);
    /// # Ok::<(), ip::Error>(())
    /// ```
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_ipv4(&self) -> Option<Address<Ipv4>> {
        self.to_ipv4_mapped().or_else(|| match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, octets @ ..] => Some(Address::new(
                <Ipv4 as Afi>::Primitive::from_be_bytes(octets),
            )),
            _ => None,
        })
    }

    /// Returns the embedded [`Address<Ipv4>`] in an IPv4-mapped
    /// [`Address<Ipv6>`], or [`None`] otherwise.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{Address, Ipv6};
    ///
    /// assert_eq!(
    ///     "::ffff:172.16.1.1"
    ///         .parse::<Address<Ipv6>>()?
    ///         .to_ipv4_mapped()
    ///         .map(|ipv4| ipv4.octets()),
    ///     Some([172, 16, 1, 1]),
    /// );
    ///
    /// assert_eq!(
    ///     "::192.168.1.1".parse::<Address<Ipv6>>()?.to_ipv4_mapped(),
    ///     None,
    /// );
    ///
    /// assert_eq!(
    ///     "2001:db8::1".parse::<Address<Ipv6>>()?.to_ipv4_mapped(),
    ///     None,
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_ipv4_mapped(&self) -> Option<Address<Ipv4>> {
        match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, octets @ ..] => Some(Address::new(
                <Ipv4 as Afi>::Primitive::from_be_bytes(octets),
            )),
            _ => None,
        }
    }
}

// TODO: document omission of `non_exhaustive`
/// IPv6 multicast address scopes, as defined in [RFC 4291].
///
/// See also [`Address::multicast_scope()`].
///
/// [RFC 4291]: https://tools.ietf.org/html/rfc4291
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum MulticastScope {
    /// Reserved.
    Reserved,
    /// Scope values not yet assigned to a multicast scope.
    Unassigned,
    /// Interface local scope.
    InterfaceLocal,
    /// Link local scope.
    LinkLocal,
    /// Realm local scope.
    RealmLocal,
    /// Locally administered scope.
    AdminLocal,
    /// Site local scope.
    SiteLocal,
    /// Organization local scope.
    OrganizationLocal,
    /// Global scope.
    Global,
}
