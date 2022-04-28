use core::mem;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitXor};

use crate::{
    af::{Afi, Ipv4, Ipv6},
    mask::{self, ConcreteMask},
    prefix::ConcretePrefixLength,
    primitive::AddressPrimitive,
};

mod range;
use self::range::AddressRange;

mod private {
    use super::*;

    /// An IP address.
    #[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ConcreteAddress<A: Afi>(A::AddressPrimitive);

    impl<A: Afi> ConcreteAddress<A> {
        // TODO: use `Self::new()` to construct these (and move out of `mod
        // private`) once const trait bounds are available in stable rustc
        // (1.61+)
        pub const LOCALHOST: Self = Self(A::AddressPrimitive::LOCALHOST);
        pub const UNSPECIFIED: Self = Self(A::AddressPrimitive::UNSPECIFIED);

        /// Construct a new [`Address<A>`] from an integer primitive
        /// appropriate to `A`.
        pub fn new(inner: A::AddressPrimitive) -> Self {
            Self(inner)
        }

        /// Get the inner integer val, consuming `self`.
        pub fn into_primitive(self) -> A::AddressPrimitive {
            self.0
        }
    }

    impl ConcreteAddress<Ipv4> {
        // TODO: use `Self::new()` to contruct these once const trait bounds are
        // available in stable rustc (1.61+)
        pub const BROADCAST: Self = {
            if let Some(inner) = <Ipv4 as Afi>::AddressPrimitive::BROADCAST {
                Self(inner)
            } else {
                panic!("failed to get BROADCAST address value")
            }
        };
    }
}

pub use self::private::ConcreteAddress;

// TODO: make methods `const fn`
impl ConcreteAddress<Ipv4> {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_ipv6_compatible(&self) -> ConcreteAddress<Ipv6> {
        ConcreteAddress::new(<Ipv6 as Afi>::AddressPrimitive::from_be_bytes(
            self.to_ipv6_lo_octets(),
        ))
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_ipv6_mapped(&self) -> ConcreteAddress<Ipv6> {
        let mut octets = self.to_ipv6_lo_octets();
        octets[10..12].copy_from_slice(&[0xffu8, 0xffu8]);
        ConcreteAddress::new(<Ipv6 as Afi>::AddressPrimitive::from_be_bytes(octets))
    }

    fn to_ipv6_lo_octets(self) -> <Ipv6 as Afi>::Octets {
        let mut octets = <Ipv6 as Afi>::Octets::default();
        octets[12..].copy_from_slice(&self.octets());
        octets
    }
}

// TODO: make methods `const fn`
impl ConcreteAddress<Ipv6> {
    pub fn is_unicast_link_local(&self) -> bool {
        self.is_link_local()
    }

    pub fn multicast_scope(&self) -> Option<MulticastScope> {
        if self.is_multicast() {
            match self.octets()[1] & 0x0f {
                0x0 => Some(MulticastScope::Reserved),
                0x1 => Some(MulticastScope::InterfaceLocal),
                0x2 => Some(MulticastScope::LinkLocal),
                0x3 => Some(MulticastScope::RealmLocal),
                0x4 => Some(MulticastScope::AdminLocal),
                0x5 => Some(MulticastScope::SiteLocal),
                0x6..=0x07 => Some(MulticastScope::Unassigned),
                0x8 => Some(MulticastScope::OrganizationLocal),
                0x9..=0x0d => Some(MulticastScope::Unassigned),
                0xe => Some(MulticastScope::Global),
                0xf => Some(MulticastScope::Reserved),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    pub fn segments(&self) -> [u16; 8] {
        // SAFTEY: [u8; 16] is always safe to transmute to [u16; 8]
        let [a, b, c, d, e, f, g, h] = unsafe { mem::transmute::<_, [u16; 8]>(self.octets()) };
        [
            u16::from_be(a),
            u16::from_be(b),
            u16::from_be(c),
            u16::from_be(d),
            u16::from_be(e),
            u16::from_be(f),
            u16::from_be(g),
            u16::from_be(h),
        ]
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_canonical(&self) -> AnyAddress {
        if let Some(ipv4_addr) = self.to_ipv4_mapped() {
            AnyAddress::Ipv4(ipv4_addr)
        } else {
            AnyAddress::Ipv6(*self)
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_ipv4(&self) -> Option<ConcreteAddress<Ipv4>> {
        self.to_ipv4_mapped().or_else(|| match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, octets @ ..] => Some(ConcreteAddress::new(
                <Ipv4 as Afi>::AddressPrimitive::from_be_bytes(octets),
            )),
            _ => None,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_ipv4_mapped(&self) -> Option<ConcreteAddress<Ipv4>> {
        match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, octets @ ..] => Some(ConcreteAddress::new(
                <Ipv4 as Afi>::AddressPrimitive::from_be_bytes(octets),
            )),
            _ => None,
        }
    }
}

impl<A: Afi> ConcreteAddress<A> {
    pub fn octets(&self) -> A::Octets {
        self.into_primitive().to_be_bytes()
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnyAddress {
    Ipv4(ConcreteAddress<Ipv4>),
    Ipv6(ConcreteAddress<Ipv6>),
}

impl AnyAddress {
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

pub trait AddressI {
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
    fn is_unspecified(&self) -> bool;
    fn is_unique_local(&self) -> bool;
    fn is_unicast(&self) -> bool {
        !self.is_multicast()
    }
    fn is_unicast_global(&self) -> bool {
        self.is_unicast() && self.is_global()
    }
}

impl<A: Afi> AddressI for ConcreteAddress<A> {
    fn is_broadcast(&self) -> bool {
        if let Some(broadcast) = A::AddressPrimitive::BROADCAST {
            self.into_primitive() == broadcast
        } else {
            false
        }
    }
    fn is_link_local(&self) -> bool {
        AddressRange::from(&A::AddressPrimitive::LINK_LOCAL_RANGE).contains(self)
    }

    fn is_private(&self) -> bool {
        if let Some(ranges) = A::AddressPrimitive::PRIVATE_RANGES {
            ranges
                .iter()
                .any(|range| AddressRange::from(range).contains(self))
        } else {
            false
        }
    }

    fn is_reserved(&self) -> bool {
        if let Some(range) = A::AddressPrimitive::RESERVED_RANGE {
            AddressRange::from(&range).contains(self)
        } else {
            false
        }
    }

    fn is_shared(&self) -> bool {
        if let Some(range) = A::AddressPrimitive::SHARED_RANGE {
            AddressRange::from(&range).contains(self)
        } else {
            false
        }
    }

    fn is_thisnet(&self) -> bool {
        if let Some(range) = A::AddressPrimitive::THISNET_RANGE {
            AddressRange::from(&range).contains(self)
        } else {
            false
        }
    }

    fn is_benchmarking(&self) -> bool {
        AddressRange::from(&A::AddressPrimitive::BENCHMARK_RANGE).contains(self)
    }

    fn is_documentation(&self) -> bool {
        A::AddressPrimitive::DOCUMENTATION_RANGES
            .iter()
            .any(|range| AddressRange::from(range).contains(self))
    }

    fn is_global(&self) -> bool {
        self.into_primitive().is_global()
    }

    fn is_loopback(&self) -> bool {
        AddressRange::from(&A::AddressPrimitive::LOCALHOST_RANGE).contains(self)
    }

    fn is_multicast(&self) -> bool {
        AddressRange::from(&A::AddressPrimitive::MULTICAST_RANGE).contains(self)
    }

    fn is_unspecified(&self) -> bool {
        self == &Self::UNSPECIFIED
    }

    fn is_unique_local(&self) -> bool {
        if let Some(range) = A::AddressPrimitive::ULA_RANGE {
            AddressRange::from(&range).contains(self)
        } else {
            false
        }
    }
}

macro_rules! delegate {
    ( $( fn $fn:ident(&self) -> $ret_ty:ty; )* ) => {
        $(
            fn $fn(&self) -> $ret_ty {
                match self {
                    Self::Ipv4(addr) => addr.$fn(),
                    Self::Ipv6(addr) => addr.$fn(),
                }
            }
        )*
    }
}

impl AddressI for AnyAddress {
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

impl From<ConcreteAddress<Ipv4>> for AnyAddress {
    fn from(addr: ConcreteAddress<Ipv4>) -> Self {
        Self::Ipv4(addr)
    }
}

impl From<ConcreteAddress<Ipv6>> for AnyAddress {
    fn from(addr: ConcreteAddress<Ipv6>) -> Self {
        Self::Ipv6(addr)
    }
}

// TODO: document omission of `non_exhaustive`
pub enum MulticastScope {
    Reserved,
    Unassigned,
    InterfaceLocal,
    LinkLocal,
    RealmLocal,
    AdminLocal,
    SiteLocal,
    OrganizationLocal,
    Global,
}

/// Compute the length, as a [`PrefixLength<A>`], for the common prefixes of
/// two [`Address<A>`].
pub fn common_length<A: Afi>(
    lhs: ConcreteAddress<A>,
    rhs: ConcreteAddress<A>,
) -> ConcretePrefixLength<A> {
    lhs.common_length(rhs)
}

impl<A: Afi> ConcreteAddress<A> {
    /// Compute the common length of `self` and another [`Address<A>`].
    pub fn common_length(self, other: Self) -> ConcretePrefixLength<A> {
        // ok to unwrap here as long as primitive width invariants hold
        ConcretePrefixLength::<A>::from_primitive((self ^ other).leading_zeros()).unwrap()
    }
}

impl<A: Afi, T: mask::Type> BitAnd<ConcreteMask<T, A>> for ConcreteAddress<A> {
    type Output = Self;

    fn bitand(self, mask: ConcreteMask<T, A>) -> Self::Output {
        Self::new(self.into_primitive().bitand(mask.into_primitive()))
    }
}

impl<A: Afi, T> BitAndAssign<T> for ConcreteAddress<A>
where
    Self: BitAnd<T, Output = Self>,
{
    fn bitand_assign(&mut self, rhs: T) {
        *self = self.bitand(rhs);
    }
}

impl<A: Afi, T: mask::Type> BitOr<ConcreteMask<T, A>> for ConcreteAddress<A> {
    type Output = Self;

    fn bitor(self, mask: ConcreteMask<T, A>) -> Self::Output {
        Self::new(self.into_primitive().bitor(mask.into_primitive()))
    }
}

impl<A: Afi> BitXor<Self> for ConcreteAddress<A> {
    type Output = A::AddressPrimitive;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.into_primitive() ^ rhs.into_primitive()
    }
}

mod parse {
    use super::*;

    use core::str::FromStr;

    use crate::error::Error;

    impl<A: Afi> FromStr for ConcreteAddress<A> {
        type Err = Error<'static, A>;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            A::AddressPrimitive::parse_addr(s).map(Self::new)
        }
    }
}

mod fmt {
    use super::*;

    use core::fmt;

    use crate::fmt::AddressDisplay;

    impl<A: Afi> fmt::Display for ConcreteAddress<A>
    where
        A::AddressPrimitive: AddressDisplay<A>,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.into_primitive().fmt_addr(f)
        }
    }
}

#[cfg(feature = "std")]
mod convert {
    use super::*;

    use std::net::{Ipv4Addr, Ipv6Addr};

    use crate::af::{Ipv4, Ipv6};

    impl From<Ipv4Addr> for ConcreteAddress<Ipv4> {
        fn from(addr: Ipv4Addr) -> Self {
            Self::new(addr.into())
        }
    }

    impl From<Ipv6Addr> for ConcreteAddress<Ipv6> {
        fn from(addr: Ipv6Addr) -> Self {
            Self::new(addr.into())
        }
    }
}

#[cfg(any(test, feature = "arbitrary"))]
mod arbitrary {
    use super::*;

    use proptest::{
        arbitrary::{any_with, Arbitrary, ParamsFor, StrategyFor},
        strategy::{BoxedStrategy, Strategy},
    };

    impl<A: Afi> Arbitrary for ConcreteAddress<A>
    where
        A: 'static,
        A::AddressPrimitive: Arbitrary + 'static,
        StrategyFor<A::AddressPrimitive>: 'static,
    {
        type Parameters = ParamsFor<A::AddressPrimitive>;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            any_with::<A::AddressPrimitive>(params)
                .prop_map(Self::new)
                .boxed()
        }
    }
}
