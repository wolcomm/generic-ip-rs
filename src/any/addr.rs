use core::cmp::Ordering;
use core::fmt;
use core::str::FromStr;

use super::delegate;
use crate::{
    concrete::{self, Ipv4, Ipv6},
    error::Error,
    traits::{
        self,
        primitive::{Address as _, IntoIpv6Segments as _},
        Afi,
    },
};

/// Either an IPv4 or IPv6 address.
///
/// # Memory Use
///
/// Rust enums are sized to accommodate their largest variant, with smaller
/// variants being padded to fill up any unused space.
///
/// As a result, users should avoid using this type in a context where only
/// [`Address::Ipv4`] variants are expected.
///
/// # Examples
///
/// ``` rust
/// use ip::{traits::Address as _, Address, Any};
///
/// let addr = "2001:db8::1".parse::<Address<Any>>()?;
///
/// assert!(addr.is_documentation());
/// # Ok::<(), ip::Error>(())
/// ```
#[allow(variant_size_differences)]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Address {
    /// IPv4 address variant.
    Ipv4(concrete::Address<Ipv4>),
    /// IPv6 address variant.
    Ipv6(concrete::Address<Ipv6>),
}

impl Address {
    /// Returns [`true`] if this is an IPv4 address.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Address as _, Address, Any};
    ///
    /// let ipv4_addr = "192.0.2.1".parse::<Address<Any>>()?;
    /// let ipv6_addr = "2001:db8::1".parse::<Address<Any>>()?;
    ///
    /// assert!(ipv4_addr.is_ipv4());
    /// assert!(!ipv6_addr.is_ipv4());
    /// # Ok::<(), ip::Error>(())
    /// ```
    #[must_use]
    pub const fn is_ipv4(&self) -> bool {
        matches!(self, Self::Ipv4(_))
    }

    /// Returns [`true`] if this is an IPv6 address.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{traits::Address as _, Address, Any};
    ///
    /// let ipv4_addr = "192.0.2.1".parse::<Address<Any>>()?;
    /// let ipv6_addr = "2001:db8::1".parse::<Address<Any>>()?;
    ///
    /// assert!(!ipv4_addr.is_ipv6());
    /// assert!(ipv6_addr.is_ipv6());
    /// # Ok::<(), ip::Error>(())
    /// ```
    #[must_use]
    pub const fn is_ipv6(&self) -> bool {
        matches!(self, Self::Ipv6(_))
    }

    // TODO: move to `traits::Address`
    /// Convert the address to its canonical representation.
    ///
    /// [`Address::Ipv4`] variants are returned unchanged.
    ///
    /// [`Address::Ipv6`] variants are handled by converting an IPv4-mapped
    /// IPv6 address to an [`Address::Ipv4`], and returning an
    /// [`Address::Ipv6`] otherwise.
    ///
    /// # Examples
    ///
    /// ``` rust
    /// use ip::{Address, Any, Ipv6};
    ///
    /// let ipv4_mapped = "::ffff:192.0.2.1".parse::<Address<Any>>()?;
    ///
    /// assert!(ipv4_mapped.is_ipv6());
    /// assert!(ipv4_mapped.to_canonical().is_ipv4());
    /// # Ok::<(), ip::Error>(())
    /// ```
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_canonical(&self) -> Self {
        match self {
            Self::Ipv4(_) => *self,
            Self::Ipv6(ipv6_addr) => ipv6_addr.to_canonical(),
        }
    }
}

impl traits::Address for Address {
    delegate! {
        fn afi(&self) -> concrete::Afi;
        fn is_broadcast(&self) -> bool;
        fn is_link_local(&self) -> bool;
        fn is_private(&self) -> bool;
        fn is_reserved(&self) -> bool;
        fn is_shared(&self) -> bool;
        fn is_thisnet(&self) -> bool;
        fn is_benchmarking(&self) -> bool;
        fn is_documentation(&self) -> bool;
        fn is_global(&self) -> bool;
        fn is_loopback(&self) -> bool;
        fn is_multicast(&self) -> bool;
        fn is_unicast(&self) -> bool;
        fn is_unspecified(&self) -> bool;
        fn is_unique_local(&self) -> bool;
    }
}

macro_rules! impl_from_address {
    ( $( $af:ident ),* $(,)? ) => {
        $(
            impl From<concrete::Address<$af>> for Address {
                fn from(addr: concrete::Address<$af>) -> Self {
                    Self::$af(addr)
                }
            }
        )*
    }
}
impl_from_address!(Ipv4, Ipv6);

macro_rules! impl_from_primitive {
    ( $( $af:ident ),* $(,)? ) => {
        $(
            impl From<<$af as Afi>::Primitive> for Address {
                fn from(primitive: <$af as Afi>::Primitive) -> Self {
                    concrete::Address::<$af>::new(primitive).into()
                }
            }
        )*
    }
}
impl_from_primitive!(Ipv4, Ipv6);

macro_rules! impl_from_octets {
    ( $( $af:ident ),* $(,)? ) => {
        $(
            impl From<<$af as Afi>::Octets> for Address {
                fn from(octets: <$af as Afi>::Octets) -> Self {
                    <$af as Afi>::Primitive::from_be_bytes(octets).into()
                }
            }
        )*
    }
}
impl_from_octets!(Ipv4, Ipv6);

impl From<[u16; 8]> for Address {
    fn from(segments: [u16; 8]) -> Self {
        <Ipv6 as Afi>::Primitive::from_segments(segments).into()
    }
}

#[cfg(feature = "std")]
impl From<std::net::Ipv4Addr> for Address {
    fn from(addr: std::net::Ipv4Addr) -> Self {
        concrete::Address::from(addr).into()
    }
}

#[cfg(feature = "std")]
impl From<std::net::Ipv6Addr> for Address {
    fn from(addr: std::net::Ipv6Addr) -> Self {
        concrete::Address::from(addr).into()
    }
}

#[cfg(feature = "std")]
impl From<std::net::IpAddr> for Address {
    fn from(addr: std::net::IpAddr) -> Self {
        match addr {
            std::net::IpAddr::V4(addr) => addr.into(),
            std::net::IpAddr::V6(addr) => addr.into(),
        }
    }
}

impl PartialOrd for Address {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Ipv4(addr), Self::Ipv4(other)) => addr.partial_cmp(other),
            (Self::Ipv6(addr), Self::Ipv6(other)) => addr.partial_cmp(other),
            _ => None,
        }
    }
}

macro_rules! impl_partial_cmp {
    ( $( $af:ident ),* $(,)? ) => {
        $(
            impl PartialEq<concrete::Address<$af>> for Address {
                fn eq(&self, other: &concrete::Address<$af>) -> bool {
                    if let Self::$af(addr) = self {
                        addr.eq(other)
                    } else {
                        false
                    }
                }
            }

            impl PartialEq<Address> for concrete::Address<$af> {
                fn eq(&self, other: &Address) -> bool {
                    other.eq(self)
                }
            }

            impl PartialOrd<concrete::Address<$af>> for Address {
                fn partial_cmp(&self, other: &concrete::Address<$af>) -> Option<Ordering> {
                    if let Self::$af(addr) = self {
                        addr.partial_cmp(other)
                    } else {
                        None
                    }
                }
            }

            impl PartialOrd<Address> for concrete::Address<$af> {
                fn partial_cmp(&self, other: &Address) -> Option<Ordering> {
                    other.partial_cmp(self).map(Ordering::reverse)
                }
            }
        )*
    }
}
impl_partial_cmp!(Ipv4, Ipv6);

impl FromStr for Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Ipv4 as Afi>::Primitive::parse_addr(s)
            .map(Self::from)
            .or_else(|_| <Ipv6 as Afi>::Primitive::parse_addr(s).map(Self::from))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4(addr) => addr.fmt(f),
            Self::Ipv6(addr) => addr.fmt(f),
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address<Any>::")?;
        match self {
            Self::Ipv4(addr) => write!(f, "Ipv4({addr})"),
            Self::Ipv6(addr) => write!(f, "Ipv6({addr})"),
        }
    }
}

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any, Arbitrary},
    prop_oneof,
    strategy::{BoxedStrategy, Strategy},
};

#[cfg(any(test, feature = "arbitrary"))]
impl Arbitrary for Address {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        prop_oneof![
            any::<concrete::Address<Ipv4>>().prop_map(Self::Ipv4),
            any::<concrete::Address<Ipv6>>().prop_map(Self::Ipv6),
        ]
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use proptest::proptest;

    use super::*;
    use crate::traits::Address as _;

    #[cfg(feature = "std")]
    proptest! {
        #[test]
        fn parse_any_display(addr in any::<Address>()) {
            use std::string::ToString as _;
            let parsed = addr.to_string().parse::<Address>().unwrap();
            assert_eq!(addr, parsed);
        }
    }

    proptest! {
        #[test]
        fn symmetric_eq((a, b) in any::<(Address, Address)>()) {
            assert_eq!(a.eq(&b), b.eq(&a));
        }

        #[test]
        fn symmetric_eq_ipv4(a in any::<Address>(), b in any::<concrete::Address<Ipv4>>()) {
            assert_eq!(a.eq(&b), b.eq(&a));
        }

        #[test]
        fn symmetric_eq_ipv6(a in any::<Address>(), b in any::<concrete::Address<Ipv6>>()) {
            assert_eq!(a.eq(&b), b.eq(&a));
        }

        #[test]
        fn transitive_eq((a, b, c) in any::<(Address, Address, Address)>()) {
            assert_eq!(a == b && b == c, a == c);
        }

        #[test]
        fn transitive_eq_ipv4(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv4>>(),
        ) {
            assert_eq!(a == b && b == c, a == c);
        }

        #[test]
        fn transitive_eq_ipv6(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv6>>(),
        ) {
            assert_eq!(a == b && b == c, a == c);
        }
    }

    proptest! {
        #[test]
        fn dual_cmp((a, b) in any::<(Address, Address)>()) {
            assert_eq!(a.partial_cmp(&b), b.partial_cmp(&a).map(Ordering::reverse));
        }

        #[test]
        fn dual_cmp_ipv4(a in any::<Address>(), b in any::<concrete::Address<Ipv4>>()) {
            assert_eq!(a.partial_cmp(&b), b.partial_cmp(&a).map(Ordering::reverse));
        }

        #[test]
        fn dual_cmp_ipv6(a in any::<Address>(), b in any::<concrete::Address<Ipv6>>()) {
            assert_eq!(a.partial_cmp(&b), b.partial_cmp(&a).map(Ordering::reverse));
        }

        #[test]
        fn transitive_le((a, b, c) in any::<(Address, Address, Address)>()) {
            if a < b && b < c {
                assert!(a < c);
            }
        }

        #[test]
        fn transitive_le_ipv4(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv4>>(),
        ) {
            if a < b && b < c {
                assert!(a < c);
            }
        }

        #[test]
        fn transitive_le_ipv6(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv6>>(),
        ) {
            if a < b && b < c {
                assert!(a < c);
            }
        }

        #[test]
        fn transitive_ge((a, b, c) in any::<(Address, Address, Address)>()) {
            if a > b && b > c {
                assert!(a > c);
            }
        }

        #[test]
        fn transitive_ge_ipv4(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv4>>(),
        ) {
            if a > b && b > c {
                assert!(a > c);
            }
        }

        #[test]
        fn transitive_ge_ipv6(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv6>>(),
        ) {
            if a > b && b > c {
                assert!(a > c);
            }
        }
    }

    #[test]
    fn ipv4_broadcast_is_broadcast() {
        assert!("255.255.255.255".parse::<Address>().unwrap().is_broadcast());
    }

    #[test]
    fn ipv4_unicast_is_not_broadcast() {
        assert!(!"203.0.113.1".parse::<Address>().unwrap().is_broadcast());
    }

    #[test]
    fn ipv6_all_ones_is_not_broadcast() {
        assert!(!"ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"
            .parse::<Address>()
            .unwrap()
            .is_broadcast());
    }

    #[test]
    fn ipv4_link_local_is_link_local() {
        assert!("169.254.254.1".parse::<Address>().unwrap().is_link_local());
    }

    #[test]
    fn ipv6_link_local_is_link_local() {
        assert!("fe80::1".parse::<Address>().unwrap().is_link_local());
    }

    #[test]
    fn ipv4_unicast_is_not_link_local() {
        assert!(!"203.0.113.1".parse::<Address>().unwrap().is_link_local());
    }

    #[test]
    fn ipv6_localhost_is_not_link_local() {
        assert!(!"::1".parse::<Address>().unwrap().is_link_local());
    }

    #[test]
    fn ipv4_private_is_private() {
        assert!("172.18.0.1".parse::<Address>().unwrap().is_private());
    }

    #[test]
    fn ipv4_unicast_is_not_private() {
        assert!(!"203.0.113.1".parse::<Address>().unwrap().is_private());
    }

    #[test]
    fn ipv6_ula_is_not_private() {
        assert!(!"fc01::1".parse::<Address>().unwrap().is_private());
    }

    #[test]
    fn ipv4_reserved_is_reserved() {
        assert!("240.0.0.1".parse::<Address>().unwrap().is_reserved());
    }

    #[test]
    fn ipv4_broadcast_is_not_reserved() {
        assert!(!"255.255.255.255".parse::<Address>().unwrap().is_reserved());
    }

    #[test]
    fn ipv6_unassigned_is_not_reserved() {
        assert!(!"4::1".parse::<Address>().unwrap().is_reserved());
    }

    #[test]
    fn ipv4_shared_is_shared() {
        assert!("100.72.1.1".parse::<Address>().unwrap().is_shared());
    }

    #[test]
    fn ipv4_unicast_is_not_shared() {
        assert!(!"192.0.2.1".parse::<Address>().unwrap().is_shared());
    }

    #[test]
    fn ipv6_ula_is_not_shared() {
        assert!(!"fc00::1".parse::<Address>().unwrap().is_shared());
    }

    #[test]
    fn ipv4_thisnet_is_thisnet() {
        assert!("0.255.255.255".parse::<Address>().unwrap().is_thisnet());
    }

    #[test]
    fn ipv6_unspecified_is_not_thisnet() {
        assert!(!"::".parse::<Address>().unwrap().is_thisnet());
    }

    #[test]
    fn ipv4_benchmarking_is_benmarking() {
        assert!("198.19.0.1".parse::<Address>().unwrap().is_benchmarking());
    }

    #[test]
    fn ipv6_benchmarking_is_benmarking() {
        assert!("2001:2::1".parse::<Address>().unwrap().is_benchmarking());
    }

    #[test]
    fn ipv4_test_net_2_is_documentation() {
        assert!("198.51.100.1"
            .parse::<Address>()
            .unwrap()
            .is_documentation());
    }

    #[test]
    fn ipv6_documentation_is_documentation() {
        assert!("2001:db8::1".parse::<Address>().unwrap().is_documentation());
    }

    #[test]
    fn ipv4_private_1_is_not_global() {
        assert!(!"10.254.0.0".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_private_2_is_not_global() {
        assert!(!"192.168.10.65".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_private_3_is_not_global() {
        assert!(!"172.16.10.65".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv6_ula_is_not_global() {
        assert!(!"fc00::1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_thisnet_is_not_global() {
        assert!(!"0.1.2.3".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_unspecified_is_not_global() {
        assert!(!"0.0.0.0".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv6_unspecified_is_not_global() {
        assert!(!"::".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_localhost_is_not_global() {
        assert!(!"127.0.0.1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv6_localhost_is_not_global() {
        assert!(!"::1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_link_local_is_not_global() {
        assert!(!"169.254.45.1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv6_link_local_is_not_global() {
        assert!(!"fe80::1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_broadcast_is_not_global() {
        assert!(!"255.255.255.255".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_doc_1_is_not_global() {
        assert!(!"192.0.2.255".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_doc_2_is_not_global() {
        assert!(!"198.51.100.65".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_doc_3_is_not_global() {
        assert!(!"203.0.113.6".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv6_doc_is_not_global() {
        assert!(!"2001:db8::1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_shared_is_not_global() {
        assert!(!"100.100.0.0".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_proto_specific_1_is_not_global() {
        assert!(!"192.0.0.0".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_proto_specific_2_is_not_global() {
        assert!(!"192.0.0.255".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv6_proto_specific_is_not_global() {
        assert!(!"2001:100::1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_reserved_is_not_global() {
        assert!(!"250.10.20.30".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_benchmarking_is_not_global() {
        assert!(!"198.18.0.0".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv6_benchmarking_is_not_global() {
        assert!(!"2001:2::1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_local_multicast_is_not_global() {
        assert!(!"224.0.0.1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_domain_multicast_is_not_global() {
        assert!(!"239.0.0.1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv6_non_global_multicast_is_not_global() {
        assert!(!"ff08::1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_pcp_anycast_is_global() {
        assert!("192.0.0.9".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv6_orchidv2_is_global() {
        assert!("2001:20::1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_global_multicast_is_global() {
        assert!("224.0.1.1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv6_global_multicast_is_global() {
        assert!("ff0e::1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv4_global_unicast_is_global() {
        assert!("1.1.1.1".parse::<Address>().unwrap().is_global());
    }

    #[test]
    fn ipv6_global_unicast_is_global() {
        assert!("2606:4700:4700::1111"
            .parse::<Address>()
            .unwrap()
            .is_global());
    }

    #[test]
    fn ipv4_loopback_is_loopback() {
        assert!("127.0.0.53".parse::<Address>().unwrap().is_loopback());
    }

    #[test]
    fn ipv6_loopback_is_loopback() {
        assert!("::1".parse::<Address>().unwrap().is_loopback());
    }

    #[test]
    fn ipv4_multicast_is_multicast() {
        assert!("224.254.0.0".parse::<Address>().unwrap().is_multicast());
    }

    #[test]
    fn ipv4_unicast_is_not_multicast() {
        assert!(!"172.16.10.65".parse::<Address>().unwrap().is_multicast());
    }

    #[test]
    fn ipv6_multicast_is_multicast() {
        assert!("ff01::1".parse::<Address>().unwrap().is_multicast());
    }

    #[test]
    fn ipv4_unspecified_is_unspecified() {
        assert!("0.0.0.0".parse::<Address>().unwrap().is_unspecified());
    }

    #[test]
    fn ipv6_unspecified_is_unspecified() {
        assert!("::".parse::<Address>().unwrap().is_unspecified());
    }

    #[test]
    fn ipv6_ula_is_unique_local() {
        assert!("fc01::1".parse::<Address>().unwrap().is_unique_local());
    }

    #[test]
    fn ipv6_doc_is_not_unique_local() {
        assert!(!"2001:db8::1".parse::<Address>().unwrap().is_unique_local());
    }

    #[test]
    fn ipv4_private_is_not_unique_local() {
        assert!(!"192.168.1.1".parse::<Address>().unwrap().is_unique_local());
    }

    #[test]
    fn ipv6_unicast_is_unicast() {
        assert!("2001:db8::1".parse::<Address>().unwrap().is_unicast());
    }
    #[test]
    fn ipv4_unicast_is_unicast() {
        assert!("192.168.1.1".parse::<Address>().unwrap().is_unicast());
    }
    #[test]
    fn ipv6_multicast_is_not_unicast() {
        assert!(!"ffaa::1".parse::<Address>().unwrap().is_unicast());
    }
    #[test]
    fn ipv4_multicast_is_not_unicast() {
        assert!(!"239.0.0.1".parse::<Address>().unwrap().is_unicast());
    }
    #[test]
    fn ipv4_broadcast_is_not_unicast() {
        assert!(!"255.255.255.255".parse::<Address>().unwrap().is_unicast());
    }

    #[test]
    fn ipv4_unicast_global_is_unicast_global() {
        assert!("1.1.1.1".parse::<Address>().unwrap().is_unicast_global());
    }
    #[test]
    fn ipv6_unicast_global_is_unicast_global() {
        assert!("2606:4700:4700::1111"
            .parse::<Address>()
            .unwrap()
            .is_unicast_global());
    }
    #[test]
    fn ipv4_unicast_private_is_not_unicast_global() {
        assert!(!"192.168.1.1"
            .parse::<Address>()
            .unwrap()
            .is_unicast_global());
    }
    #[test]
    fn ipv4_multicast_global_is_not_unicast_global() {
        assert!(!"225.0.0.1".parse::<Address>().unwrap().is_unicast_global());
    }
    #[test]
    fn ipv6_unicast_documentation_is_not_unicast_global() {
        assert!(!"2001:db8::1"
            .parse::<Address>()
            .unwrap()
            .is_unicast_global());
    }
    #[test]
    fn ipv6_multicast_global_is_not_unicast_global() {
        assert!(!"ff0e::1".parse::<Address>().unwrap().is_unicast_global());
    }
}
