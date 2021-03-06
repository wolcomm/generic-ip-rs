use core::fmt;
use core::str::FromStr;

use crate::{
    error::Error,
    fmt::AddressDisplay,
    traits::{self, primitive::Address as _, Afi},
};

use super::{AddressRange, PrefixLength};

mod private;
pub use self::private::Address;

mod convert;
mod ops;

mod ipv4;
mod ipv6;

impl<A: Afi> Address<A> {
    pub fn from_octets(octets: A::Octets) -> Self {
        Self::new(A::Primitive::from_be_bytes(octets))
    }

    pub fn octets(&self) -> A::Octets {
        self.into_primitive().to_be_bytes()
    }

    #[allow(clippy::missing_panics_doc)]
    /// Compute the common length of `self` and another [`Address<A>`].
    pub fn common_length(self, other: Self) -> PrefixLength<A> {
        // ok to unwrap here as long as primitive width invariants hold
        PrefixLength::<A>::from_primitive((self ^ other).leading_zeros()).unwrap()
    }
}
/// Compute the length, as a [`PrefixLength<A>`], for the common prefixes of
/// two [`Address<A>`].
pub fn common_length<A: Afi>(lhs: Address<A>, rhs: Address<A>) -> PrefixLength<A> {
    lhs.common_length(rhs)
}

impl<A: Afi> traits::Address for Address<A> {
    #[allow(clippy::option_if_let_else)]
    fn is_broadcast(&self) -> bool {
        if let Some(broadcast) = A::Primitive::BROADCAST {
            self.into_primitive() == broadcast
        } else {
            false
        }
    }

    fn is_link_local(&self) -> bool {
        AddressRange::from(&A::Primitive::LINK_LOCAL_RANGE).contains(self)
    }

    #[allow(clippy::option_if_let_else)]
    fn is_private(&self) -> bool {
        if let Some(ranges) = A::Primitive::PRIVATE_RANGES {
            ranges
                .iter()
                .any(|range| AddressRange::from(range).contains(self))
        } else {
            false
        }
    }

    #[allow(clippy::option_if_let_else)]
    fn is_reserved(&self) -> bool {
        if let Some(range) = A::Primitive::RESERVED_RANGE {
            // TODO: this should compare to `Self::BROADCAST`, but that is
            // currently defined only for `Address<Ipv4>`.
            AddressRange::from(&range).contains(self) && self.into_primitive() != A::Primitive::ONES
        } else {
            false
        }
    }

    #[allow(clippy::option_if_let_else)]
    fn is_shared(&self) -> bool {
        if let Some(range) = A::Primitive::SHARED_RANGE {
            AddressRange::from(&range).contains(self)
        } else {
            false
        }
    }

    #[allow(clippy::option_if_let_else)]
    fn is_thisnet(&self) -> bool {
        if let Some(range) = A::Primitive::THISNET_RANGE {
            AddressRange::from(&range).contains(self)
        } else {
            false
        }
    }

    fn is_benchmarking(&self) -> bool {
        AddressRange::from(&A::Primitive::BENCHMARK_RANGE).contains(self)
    }

    fn is_documentation(&self) -> bool {
        A::Primitive::DOCUMENTATION_RANGES
            .iter()
            .any(|range| AddressRange::from(range).contains(self))
    }

    fn is_global(&self) -> bool {
        self.into_primitive().is_global()
    }

    fn is_loopback(&self) -> bool {
        AddressRange::from(&A::Primitive::LOCALHOST_RANGE).contains(self)
    }

    fn is_multicast(&self) -> bool {
        AddressRange::from(&A::Primitive::MULTICAST_RANGE).contains(self)
    }

    fn is_unspecified(&self) -> bool {
        self == &Self::UNSPECIFIED
    }

    #[allow(clippy::option_if_let_else)]
    fn is_unique_local(&self) -> bool {
        if let Some(range) = A::Primitive::ULA_RANGE {
            AddressRange::from(&range).contains(self)
        } else {
            false
        }
    }
}

impl<A: Afi> FromStr for Address<A> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        A::Primitive::parse_addr(s).map(Self::new)
    }
}

impl<A: Afi> fmt::Display for Address<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.into_primitive().fmt_addr(f)
    }
}

impl<A: Afi> fmt::Debug for Address<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address<{:?}>({})", A::as_afi(), self)
    }
}

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any_with, Arbitrary, ParamsFor, StrategyFor},
    strategy::{BoxedStrategy, Strategy},
};

#[cfg(any(test, feature = "arbitrary"))]
impl<A: Afi> Arbitrary for Address<A>
where
    A: 'static,
    A::Primitive: Arbitrary + 'static,
    StrategyFor<A::Primitive>: 'static,
{
    type Parameters = ParamsFor<A::Primitive>;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
        any_with::<A::Primitive>(params).prop_map(Self::new).boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{traits::Address as _, Ipv4, Ipv6};

    #[test]
    fn ipv4_broadcast_is_broadcast() {
        assert!("255.255.255.255"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_broadcast());
    }

    #[test]
    fn ipv4_unicast_is_not_broadcast() {
        assert!(!"203.0.113.1"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_broadcast());
    }

    #[test]
    fn ipv6_all_ones_is_not_broadcast() {
        assert!(!"ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"
            .parse::<Address<Ipv6>>()
            .unwrap()
            .is_broadcast());
    }

    #[test]
    fn ipv4_link_local_is_link_local() {
        assert!("169.254.254.1"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_link_local());
    }

    #[test]
    fn ipv6_link_local_is_link_local() {
        assert!("fe80::1".parse::<Address<Ipv6>>().unwrap().is_link_local());
    }

    #[test]
    fn ipv4_unicast_is_not_link_local() {
        assert!(!"203.0.113.1"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_link_local());
    }

    #[test]
    fn ipv6_localhost_is_not_link_local() {
        assert!(!Address::<Ipv6>::LOCALHOST.is_link_local());
    }

    #[test]
    fn ipv4_private_is_private() {
        assert!("172.18.0.1".parse::<Address<Ipv4>>().unwrap().is_private());
    }

    #[test]
    fn ipv4_unicast_is_not_private() {
        assert!(!"203.0.113.1".parse::<Address<Ipv4>>().unwrap().is_private());
    }

    #[test]
    fn ipv6_ula_is_not_private() {
        assert!(!"fc01::1".parse::<Address<Ipv6>>().unwrap().is_private());
    }

    #[test]
    fn ipv4_reserved_is_reserved() {
        assert!("240.0.0.1".parse::<Address<Ipv4>>().unwrap().is_reserved());
    }

    #[test]
    fn ipv4_broadcast_is_not_reserved() {
        assert!(!Address::<Ipv4>::BROADCAST.is_reserved());
    }

    #[test]
    fn ipv6_unassigned_is_not_reserved() {
        assert!(!"4::1".parse::<Address<Ipv6>>().unwrap().is_reserved());
    }

    #[test]
    fn ipv4_shared_is_shared() {
        assert!("100.72.1.1".parse::<Address<Ipv4>>().unwrap().is_shared());
    }

    #[test]
    fn ipv4_unicast_is_not_shared() {
        assert!(!"192.0.2.1".parse::<Address<Ipv4>>().unwrap().is_shared());
    }

    #[test]
    fn ipv6_ula_is_not_shared() {
        assert!(!"fc00::1".parse::<Address<Ipv6>>().unwrap().is_shared());
    }

    #[test]
    fn ipv4_thisnet_is_thisnet() {
        assert!("0.255.255.255"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_thisnet());
    }

    #[test]
    fn ipv6_unspecified_is_not_thisnet() {
        assert!(!Address::<Ipv6>::UNSPECIFIED.is_thisnet());
    }

    #[test]
    fn ipv4_benchmarking_is_benmarking() {
        assert!("198.19.0.1"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_benchmarking());
    }

    #[test]
    fn ipv6_benchmarking_is_benmarking() {
        assert!("2001:2::1"
            .parse::<Address<Ipv6>>()
            .unwrap()
            .is_benchmarking());
    }

    #[test]
    fn ipv4_test_net_2_is_documentation() {
        assert!("198.51.100.1"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_documentation());
    }

    #[test]
    fn ipv6_documentation_is_documentation() {
        assert!("2001:db8::1"
            .parse::<Address<Ipv6>>()
            .unwrap()
            .is_documentation());
    }

    #[test]
    fn ipv4_private_1_is_not_global() {
        assert!(!"10.254.0.0".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_private_2_is_not_global() {
        assert!(!"192.168.10.65"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_global());
    }

    #[test]
    fn ipv4_private_3_is_not_global() {
        assert!(!"172.16.10.65".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv6_ula_is_not_global() {
        assert!(!"fc00::1".parse::<Address<Ipv6>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_thisnet_is_not_global() {
        assert!(!"0.1.2.3".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_unspecified_is_not_global() {
        assert!(!Address::<Ipv4>::UNSPECIFIED.is_global());
    }

    #[test]
    fn ipv6_unspecified_is_not_global() {
        assert!(!Address::<Ipv6>::UNSPECIFIED.is_global());
    }

    #[test]
    fn ipv4_localhost_is_not_global() {
        assert!(!Address::<Ipv4>::LOCALHOST.is_global());
    }

    #[test]
    fn ipv6_localhost_is_not_global() {
        assert!(!Address::<Ipv6>::LOCALHOST.is_global());
    }

    #[test]
    fn ipv4_link_local_is_not_global() {
        assert!(!"169.254.45.1".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv6_link_local_is_not_global() {
        assert!(!"fe80::1".parse::<Address<Ipv6>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_broadcast_is_not_global() {
        assert!(!Address::<Ipv4>::BROADCAST.is_global());
    }

    #[test]
    fn ipv4_doc_1_is_not_global() {
        assert!(!"192.0.2.255".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_doc_2_is_not_global() {
        assert!(!"198.51.100.65"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_global());
    }

    #[test]
    fn ipv4_doc_3_is_not_global() {
        assert!(!"203.0.113.6".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv6_doc_is_not_global() {
        assert!(!"2001:db8::1".parse::<Address<Ipv6>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_shared_is_not_global() {
        assert!(!"100.100.0.0".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_proto_specific_1_is_not_global() {
        assert!(!"192.0.0.0".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_proto_specific_2_is_not_global() {
        assert!(!"192.0.0.255".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv6_proto_specific_is_not_global() {
        assert!(!"2001:100::1".parse::<Address<Ipv6>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_reserved_is_not_global() {
        assert!(!"250.10.20.30".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_benchmarking_is_not_global() {
        assert!(!"198.18.0.0".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv6_benchmarking_is_not_global() {
        assert!(!"2001:2::1".parse::<Address<Ipv6>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_local_multicast_is_not_global() {
        assert!(!"224.0.0.1".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_domain_multicast_is_not_global() {
        assert!(!"239.0.0.1".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv6_non_global_multicast_is_not_global() {
        assert!(!"ff08::1".parse::<Address<Ipv6>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_pcp_anycast_is_global() {
        assert!("192.0.0.9".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv6_orchidv2_is_global() {
        assert!("2001:20::1".parse::<Address<Ipv6>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_global_multicast_is_global() {
        assert!("224.0.1.1".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv6_global_multicast_is_global() {
        assert!("ff0e::1".parse::<Address<Ipv6>>().unwrap().is_global());
    }

    #[test]
    fn ipv4_global_unicast_is_global() {
        assert!("1.1.1.1".parse::<Address<Ipv4>>().unwrap().is_global());
    }

    #[test]
    fn ipv6_global_unicast_is_global() {
        assert!("2606:4700:4700::1111"
            .parse::<Address<Ipv6>>()
            .unwrap()
            .is_global());
    }

    #[test]
    fn ipv4_loopback_is_loopback() {
        assert!("127.0.0.53".parse::<Address<Ipv4>>().unwrap().is_loopback());
    }

    #[test]
    fn ipv6_loopback_is_loopback() {
        assert!("::1".parse::<Address<Ipv6>>().unwrap().is_loopback());
    }

    #[test]
    fn ipv4_multicast_is_multicast() {
        assert!("224.254.0.0"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_multicast());
    }

    #[test]
    fn ipv4_unicast_is_not_multicast() {
        assert!(!"172.16.10.65"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_multicast());
    }

    #[test]
    fn ipv6_multicast_is_multicast() {
        assert!("ff01::1".parse::<Address<Ipv6>>().unwrap().is_multicast());
    }

    #[test]
    fn ipv4_unspecified_is_unspecified() {
        assert!("0.0.0.0".parse::<Address<Ipv4>>().unwrap().is_unspecified());
    }

    #[test]
    fn ipv6_unspecified_is_unspecified() {
        assert!("::".parse::<Address<Ipv6>>().unwrap().is_unspecified());
    }

    #[test]
    fn ipv6_ula_is_unique_local() {
        assert!("fc01::1"
            .parse::<Address<Ipv6>>()
            .unwrap()
            .is_unique_local());
    }

    #[test]
    fn ipv6_doc_is_not_unique_local() {
        assert!(!"2001:db8::1"
            .parse::<Address<Ipv6>>()
            .unwrap()
            .is_unique_local());
    }

    #[test]
    fn ipv4_private_is_not_unique_local() {
        assert!(!"192.168.1.1"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_unique_local());
    }

    #[test]
    fn ipv6_unicast_is_unicast() {
        assert!("2001:db8::1".parse::<Address<Ipv6>>().unwrap().is_unicast());
    }
    #[test]
    fn ipv4_unicast_is_unicast() {
        assert!("192.168.1.1".parse::<Address<Ipv4>>().unwrap().is_unicast());
    }
    #[test]
    fn ipv6_multicast_is_not_unicast() {
        assert!(!"ffaa::1".parse::<Address<Ipv6>>().unwrap().is_unicast());
    }
    #[test]
    fn ipv4_multicast_is_not_unicast() {
        assert!(!"239.0.0.1".parse::<Address<Ipv4>>().unwrap().is_unicast());
    }
    #[test]
    fn ipv4_broadcast_is_not_unicast() {
        assert!(!"255.255.255.255"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_unicast());
    }

    #[test]
    fn ipv4_unicast_global_is_unicast_global() {
        assert!("1.1.1.1"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_unicast_global());
    }
    #[test]
    fn ipv6_unicast_global_is_unicast_global() {
        assert!("2606:4700:4700::1111"
            .parse::<Address<Ipv6>>()
            .unwrap()
            .is_unicast_global());
    }
    #[test]
    fn ipv4_unicast_private_is_not_unicast_global() {
        assert!(!"192.168.1.1"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_unicast_global());
    }
    #[test]
    fn ipv4_multicast_global_is_not_unicast_global() {
        assert!(!"225.0.0.1"
            .parse::<Address<Ipv4>>()
            .unwrap()
            .is_unicast_global());
    }
    #[test]
    fn ipv6_unicast_documentation_is_not_unicast_global() {
        assert!(!"2001:db8::1"
            .parse::<Address<Ipv6>>()
            .unwrap()
            .is_unicast_global());
    }
    #[test]
    fn ipv6_multicast_global_is_not_unicast_global() {
        assert!(!"ff0e::1"
            .parse::<Address<Ipv6>>()
            .unwrap()
            .is_unicast_global());
    }
}
