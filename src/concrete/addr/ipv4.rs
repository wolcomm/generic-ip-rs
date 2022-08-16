use super::Address;
use crate::{
    concrete::{Ipv4, Ipv6},
    traits::{primitive::Address as _, Afi},
};

// TODO: make methods `const fn`
impl Address<Ipv4> {
    /// The IPv4 subnet-local broadcast address `255.255.255.255`.
    pub const BROADCAST: Self = {
        if let Some(inner) = <Ipv4 as Afi>::Primitive::BROADCAST {
            Self::new(inner)
        } else {
            panic!("failed to get BROADCAST address value")
        }
    };

    /// Converts this [`Address<Ipv4>`] to an IPv4-compatible
    /// [`Address<Ipv6>`].
    ///
    /// IPv4-compatible IPv6 addresses are of the form `::a.b.c.d`, where
    /// `a.b.c.d` is the corresponding IPv4 address. See [RFC 4291].
    ///
    /// [RFC 4291]: https://tools.ietf.org/html/rfc4291
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{Address, Ipv4, Ipv6};
    ///
    /// assert_eq!(
    ///     "172.16.12.1".parse::<Address<Ipv4>>()?.to_ipv6_compatible(),
    ///     "::172.16.12.1".parse::<Address<Ipv6>>()?,
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_ipv6_compatible(&self) -> Address<Ipv6> {
        Address::from_octets(self.to_ipv6_lo_octets())
    }

    /// Converts this [`Address<Ipv4>`] to an IPv4-mapped [`Address<Ipv6>`].
    ///
    /// IPv4-mapped IPv6 addresses are of the form `::ffff:a.b.c.d`, where
    /// `a.b.c.d` is the corresponding IPv4 address. See [RFC 4291].
    ///
    /// [RFC 4291]: https://tools.ietf.org/html/rfc4291
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{Address, Ipv4, Ipv6};
    ///
    /// assert_eq!(
    ///     "172.16.12.1".parse::<Address<Ipv4>>()?.to_ipv6_mapped(),
    ///     "::ffff:172.16.12.1".parse::<Address<Ipv6>>()?,
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_ipv6_mapped(&self) -> Address<Ipv6> {
        let mut octets = self.to_ipv6_lo_octets();
        octets[10..12].copy_from_slice(&[0xffu8, 0xffu8]);
        Address::from_octets(octets)
    }

    fn to_ipv6_lo_octets(self) -> <Ipv6 as Afi>::Octets {
        let mut octets = <Ipv6 as Afi>::Octets::default();
        octets[12..].copy_from_slice(&self.octets());
        octets
    }
}
