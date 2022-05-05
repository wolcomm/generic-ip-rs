use core::cmp::Ordering;
use core::fmt;
use core::str::FromStr;

use crate::{
    concrete::{self, Ipv4, Ipv6},
    error::Error,
    traits::{
        self,
        primitive::{Address as _, IntoIpv6Segments as _},
        Afi,
    },
};

use super::delegate;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Address {
    Ipv4(concrete::Address<Ipv4>),
    Ipv6(concrete::Address<Ipv6>),
}

impl Address {
    pub fn is_ipv4(&self) -> bool {
        matches!(self, Self::Ipv4(_))
    }

    pub fn is_ipv6(&self) -> bool {
        matches!(self, Self::Ipv6(_))
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_canonical(&self) -> Self {
        match self {
            Self::Ipv4(_) => *self,
            Self::Ipv6(ipv6_addr) => ipv6_addr.to_canonical(),
        }
    }
}

impl traits::Address for Address {
    delegate! {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Ipv4(addr) => addr.fmt(f),
            Self::Ipv6(addr) => addr.fmt(f),
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Address<Any>::")?;
        match self {
            Self::Ipv4(addr) => write!(f, "Ipv4({})", addr),
            Self::Ipv6(addr) => write!(f, "Ipv6({})", addr),
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

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            any::<concrete::Address<Ipv4>>().prop_map(Self::Ipv4),
            any::<concrete::Address<Ipv6>>().prop_map(Self::Ipv6),
        ]
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::{arbitrary::any, proptest};

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
            assert_eq!(a.eq(&b), b.eq(&a))
        }

        #[test]
        fn symmetric_eq_ipv4(a in any::<Address>(), b in any::<concrete::Address<Ipv4>>()) {
            assert_eq!(a.eq(&b), b.eq(&a))
        }

        #[test]
        fn symmetric_eq_ipv6(a in any::<Address>(), b in any::<concrete::Address<Ipv6>>()) {
            assert_eq!(a.eq(&b), b.eq(&a))
        }

        #[test]
        fn transitive_eq((a, b, c) in any::<(Address, Address, Address)>()) {
            assert_eq!(a == b && b == c, a == c)
        }

        #[test]
        fn transitive_eq_ipv4(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv4>>(),
        ) {
            assert_eq!(a == b && b == c, a == c)
        }

        #[test]
        fn transitive_eq_ipv6(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv6>>(),
        ) {
            assert_eq!(a == b && b == c, a == c)
        }
    }

    proptest! {
        #[test]
        fn dual_cmp((a, b) in any::<(Address, Address)>()) {
            assert_eq!(a.partial_cmp(&b), b.partial_cmp(&a).map(Ordering::reverse))
        }

        #[test]
        fn dual_cmp_ipv4(a in any::<Address>(), b in any::<concrete::Address<Ipv4>>()) {
            assert_eq!(a.partial_cmp(&b), b.partial_cmp(&a).map(Ordering::reverse))
        }

        #[test]
        fn dual_cmp_ipv6(a in any::<Address>(), b in any::<concrete::Address<Ipv6>>()) {
            assert_eq!(a.partial_cmp(&b), b.partial_cmp(&a).map(Ordering::reverse))
        }

        #[test]
        fn transitive_le((a, b, c) in any::<(Address, Address, Address)>()) {
            if a < b && b < c {
                assert!(a < c)
            }
        }

        #[test]
        fn transitive_le_ipv4(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv4>>(),
        ) {
            if a < b && b < c {
                assert!(a < c)
            }
        }

        #[test]
        fn transitive_le_ipv6(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv6>>(),
        ) {
            if a < b && b < c {
                assert!(a < c)
            }
        }

        #[test]
        fn transitive_ge((a, b, c) in any::<(Address, Address, Address)>()) {
            if a > b && b > c {
                assert!(a > c)
            }
        }

        #[test]
        fn transitive_ge_ipv4(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv4>>(),
        ) {
            if a > b && b > c {
                assert!(a > c)
            }
        }

        #[test]
        fn transitive_ge_ipv6(
            (a, c) in any::<(Address, Address)>(),
            b in any::<concrete::Address<Ipv6>>(),
        ) {
            if a > b && b > c {
                assert!(a > c)
            }
        }
    }
}
