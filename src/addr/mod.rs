use core::mem;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitXor};

use crate::{
    af::{Afi, DefaultPrimitive, Ipv4, Ipv6},
    prefix::{ConcretePrefix, ConcretePrefixLength, PrefixI},
    primitive::AddressPrimitive,
};

mod mask;

pub use self::mask::{
    AnyHostmask, AnyMask, AnyNetmask, ConcreteHostmask, ConcreteMask, ConcreteNetmask, MaskI,
};

mod private {
    use super::*;

    use core::marker::PhantomData;

    /// An IP address.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ConcreteAddress<A, P = DefaultPrimitive<A>>
    where
        A: Afi,
        P: AddressPrimitive<A>,
    {
        inner: P,
        _marker: PhantomData<A>,
    }

    impl<A: Afi, P: AddressPrimitive<A>> ConcreteAddress<A, P> {
        // TODO: use `Self::new()` to construct these (and move out of `mod
        // private`) once const trait bounds are available in stable rustc
        // (1.61+)
        pub const LOCALHOST: Self = Self {
            inner: P::LOCALHOST,
            _marker: PhantomData,
        };
        pub const UNSPECIFIED: Self = Self {
            inner: P::UNSPECIFIED,
            _marker: PhantomData,
        };

        pub(crate) const LOCALHOST_NET: Self = Self {
            inner: P::LOCALHOST_NET.0,
            _marker: PhantomData,
        };

        pub(crate) const BENCHMARK_NET: Self = Self {
            inner: P::BENCHMARK_NET.0,
            _marker: PhantomData,
        };

        pub(crate) const MULTICAST_NET: Self = Self {
            inner: P::MULTICAST_NET.0,
            _marker: PhantomData,
        };

        /// Construct a new [`Address<A>`] from an integer primitive
        /// appropriate to `A`.
        pub fn new(inner: P) -> Self {
            Self {
                inner,
                _marker: PhantomData,
            }
        }

        /// Get the inner integer val, consuming `self`.
        pub fn into_primitive(self) -> P {
            self.inner
        }
    }

    impl<P: AddressPrimitive<Ipv4>> ConcreteAddress<Ipv4, P> {
        // TODO: use `Self::new()` to contruct these once const trait bounds are
        // available in stable rustc (1.61+)
        // TODO: figure out how to deal with "optional" primitive consts
        // const BROADCAST: Self = Self{inner: P::BROADCAST.unwrap(), _marker: PhantomData};
        // pub const BROADCAST: Self = Self {
        //     inner: P::from_be_bytes([255, 255, 255, 255]),
        //     _marker: PhantomData,
        // };
        pub const BROADCAST: Self = {
            if let Some(inner) = P::BROADCAST {
                Self {
                    inner,
                    _marker: PhantomData,
                }
            } else {
                panic!("failed to get BROADCAST address value")
            }
        };
    }
}

pub use self::private::ConcreteAddress;

// TODO: make methods `const fn`
impl<P: AddressPrimitive<Ipv4>> ConcreteAddress<Ipv4, P> {
    pub fn is_broadcast(&self) -> bool {
        // self == &Self::BROADCAST
        self.octets() == [255, 255, 255, 255]
    }
    pub fn is_link_local(&self) -> bool {
        matches!(self.octets(), [169, 254, ..])
    }
    pub fn is_private(&self) -> bool {
        matches!(
            self.octets(),
            [192, 168, ..] | [172, 16..=31, ..] | [10, ..]
        )
    }
    pub fn is_reserved(&self) -> bool {
        matches!(self.octets(), [240..=255, ..])
    }
    pub fn is_shared(&self) -> bool {
        matches!(self.octets(), [100, 64..=127, ..])
    }
    pub fn is_thisnet(&self) -> bool {
        matches!(self.octets(), [0, ..])
    }
    pub fn to_ipv6_compatible<P6>(&self) -> ConcreteAddress<Ipv6, P6>
    where
        P6: AddressPrimitive<Ipv6>,
    {
        ConcreteAddress::new(P6::from_be_bytes(self.to_ipv6_lo_octets()))
    }
    pub fn to_ipv6_mapped<P6>(&self) -> ConcreteAddress<Ipv6, P6>
    where
        P6: AddressPrimitive<Ipv6>,
    {
        let mut octets = self.to_ipv6_lo_octets();
        octets[10..12].copy_from_slice(&[0xffu8, 0xffu8]);
        ConcreteAddress::new(P6::from_be_bytes(octets))
    }
    fn to_ipv6_lo_octets(self) -> <Ipv6 as Afi>::Octets {
        let mut octets = <Ipv6 as Afi>::Octets::default();
        octets[12..].copy_from_slice(&self.octets());
        octets
    }
    fn is_global(&self) -> bool {
        if self.is_private()
            || self.is_loopback()
            || self.is_link_local()
            || self.is_broadcast()
            || self.is_documentation()
            || self.is_shared()
            || self.is_reserved()
            || self.is_benchmarking()
            || self.is_thisnet()
        {
            false
        } else {
            // TODO: handle 192.0.0.0/24
            unimplemented!()
        }
    }
    fn is_documentation(&self) -> bool {
        matches!(
            self.octets(),
            [192, 0, 2, _] | [198, 51, 100, _] | [203, 0, 113, _]
        )
    }
}

// TODO: make methods `const fn`
impl<P: AddressPrimitive<Ipv6>> ConcreteAddress<Ipv6, P> {
    pub fn is_unicast(&self) -> bool {
        !self.is_multicast()
    }
    pub fn is_unicast_global(&self) -> bool {
        !(self.is_loopback()
            || self.is_unicast_link_local()
            || self.is_unique_local()
            || self.is_unspecified()
            || self.is_documentation())
    }
    pub fn is_unicast_link_local(&self) -> bool {
        matches!(self.octets(), [0xfe, 0x80..=0xbf, ..])
    }
    pub fn is_unique_local(&self) -> bool {
        matches!(self.octets(), [0xfc..=0xfd, ..])
    }
    pub fn multicast_scope(&self) -> Option<MulticastScope> {
        if self.is_multicast() {
            match self.octets()[1] {
                0x00 => Some(MulticastScope::Reserved),
                0x01 => Some(MulticastScope::InterfaceLocal),
                0x02 => Some(MulticastScope::LinkLocal),
                0x03 => Some(MulticastScope::RealmLocal),
                0x04 => Some(MulticastScope::AdminLocal),
                0x05 => Some(MulticastScope::SiteLocal),
                0x06..=0x07 => Some(MulticastScope::Unassigned),
                0x08 => Some(MulticastScope::OrganizationLocal),
                0x09..=0x0d => Some(MulticastScope::Unassigned),
                0x0e => Some(MulticastScope::Global),
                0x0f => Some(MulticastScope::Reserved),
                0x10.. => unreachable!(),
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
    pub fn to_canonical<P4>(&self) -> AnyAddress<P4, P>
    where
        P4: AddressPrimitive<Ipv4>,
    {
        if let Some(ipv4_addr) = self.to_ipv4_mapped() {
            AnyAddress::Ipv4(ipv4_addr)
        } else {
            AnyAddress::Ipv6(*self)
        }
    }
    pub fn to_ipv4<P4>(&self) -> Option<ConcreteAddress<Ipv4, P4>>
    where
        P4: AddressPrimitive<Ipv4>,
    {
        self.to_ipv4_mapped().or_else(|| match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, octets @ ..] => {
                Some(ConcreteAddress::new(P4::from_be_bytes(octets)))
            }
            _ => None,
        })
    }
    pub fn to_ipv4_mapped<P4>(&self) -> Option<ConcreteAddress<Ipv4, P4>>
    where
        P4: AddressPrimitive<Ipv4>,
    {
        match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, octets @ ..] => {
                Some(ConcreteAddress::new(P4::from_be_bytes(octets)))
            }
            _ => None,
        }
    }
    fn is_global(&self) -> bool {
        self.is_unicast_global() || matches!(self.multicast_scope(), Some(MulticastScope::Global))
    }
    fn is_documentation(&self) -> bool {
        matches!(self.octets(), [0x20, 0x01, 0x0d, 0xb8, ..])
    }
}

impl<A: Afi, P: AddressPrimitive<A>> ConcreteAddress<A, P> {
    pub fn octets(&self) -> A::Octets {
        self.into_primitive().to_be_bytes()
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnyAddress<P4 = DefaultPrimitive<Ipv4>, P6 = DefaultPrimitive<Ipv6>>
where
    P4: AddressPrimitive<Ipv4>,
    P6: AddressPrimitive<Ipv6>,
{
    Ipv4(ConcreteAddress<Ipv4, P4>),
    Ipv6(ConcreteAddress<Ipv6, P6>),
}

impl<P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>> AnyAddress<P4, P6> {
    pub fn is_ipv4(&self) -> bool {
        matches!(self, Self::Ipv4(_))
    }

    pub fn is_ipv6(&self) -> bool {
        matches!(self, Self::Ipv6(_))
    }

    pub fn to_canonical(&self) -> Self {
        match self {
            Self::Ipv4(_) => *self,
            Self::Ipv6(ipv6_addr) => ipv6_addr.to_canonical(),
        }
    }
}

pub trait AddressI {
    fn is_benchmarking(&self) -> bool;
    fn is_documentation(&self) -> bool;
    fn is_global(&self) -> bool;
    fn is_loopback(&self) -> bool;
    fn is_multicast(&self) -> bool;
    fn is_unspecified(&self) -> bool;
}

impl<A: Afi, P: AddressPrimitive<A>> AddressI for ConcreteAddress<A, P> {
    fn is_benchmarking(&self) -> bool {
        ConcretePrefix::BENCHMARK.contains(*self)
    }

    fn is_documentation(&self) -> bool {
        self.is_documentation()
    }

    fn is_global(&self) -> bool {
        self.is_global()
    }

    fn is_loopback(&self) -> bool {
        ConcretePrefix::LOCALHOST.contains(*self)
    }

    fn is_multicast(&self) -> bool {
        ConcretePrefix::MULTICAST.contains(*self)
    }

    fn is_unspecified(&self) -> bool {
        self == &Self::UNSPECIFIED
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

impl<P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>> AddressI for AnyAddress<P4, P6> {
    delegate! {
        fn is_benchmarking(&self) -> bool;
        fn is_documentation(&self) -> bool;
        fn is_global(&self) -> bool;
        fn is_loopback(&self) -> bool;
        fn is_multicast(&self) -> bool;
        fn is_unspecified(&self) -> bool;
    }
}

impl<P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>> From<ConcreteAddress<Ipv4, P4>>
    for AnyAddress<P4, P6>
{
    fn from(addr: ConcreteAddress<Ipv4, P4>) -> Self {
        Self::Ipv4(addr)
    }
}

impl<P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>> From<ConcreteAddress<Ipv6, P6>>
    for AnyAddress<P4, P6>
{
    fn from(addr: ConcreteAddress<Ipv6, P6>) -> Self {
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
pub fn common_length<A, P>(
    lhs: ConcreteAddress<A, P>,
    rhs: ConcreteAddress<A, P>,
) -> ConcretePrefixLength<A, P>
where
    A: Afi,
    P: AddressPrimitive<A>,
{
    lhs.common_length(rhs)
}

impl<A: Afi, P: AddressPrimitive<A>> ConcreteAddress<A, P> {
    /// Compute the common length of `self` and another [`Address<A>`].
    pub fn common_length(self, other: Self) -> ConcretePrefixLength<A, P> {
        // ok to unwrap here as long as primitive width invariants hold
        ConcretePrefixLength::<A, P>::from_primitive((self ^ other).leading_zeros()).unwrap()
    }
}

impl<A: Afi, P: AddressPrimitive<A>, T: mask::Type> BitAnd<ConcreteMask<T, A, P>>
    for ConcreteAddress<A, P>
{
    type Output = Self;

    fn bitand(self, mask: ConcreteMask<T, A, P>) -> Self::Output {
        Self::new(self.into_primitive().bitand(mask.into_primitive()))
    }
}

impl<A: Afi, P: AddressPrimitive<A>, T> BitAndAssign<T> for ConcreteAddress<A, P>
where
    Self: BitAnd<T, Output = Self>,
{
    fn bitand_assign(&mut self, rhs: T) {
        *self = self.bitand(rhs);
    }
}

impl<A: Afi, P: AddressPrimitive<A>, T: mask::Type> BitOr<ConcreteMask<T, A, P>>
    for ConcreteAddress<A, P>
{
    type Output = Self;

    fn bitor(self, mask: ConcreteMask<T, A, P>) -> Self::Output {
        Self::new(self.into_primitive().bitor(mask.into_primitive()))
    }
}

impl<A: Afi, P: AddressPrimitive<A>> BitXor<Self> for ConcreteAddress<A, P> {
    type Output = P;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.into_primitive() ^ rhs.into_primitive()
    }
}

mod parse {
    use super::*;

    use core::str::FromStr;

    use crate::error::Error;

    impl<A: Afi, P: AddressPrimitive<A>> FromStr for ConcreteAddress<A, P> {
        type Err = Error<'static, A, P>;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            P::parse_addr(s).map(Self::new)
        }
    }
}

mod fmt {
    use super::*;

    use core::fmt;

    use crate::fmt::AddressDisplay;

    impl<A: Afi, P: AddressPrimitive<A>> fmt::Display for ConcreteAddress<A, P>
    where
        P: AddressDisplay<A>,
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

    impl<P: AddressPrimitive<Ipv4>> From<Ipv4Addr> for ConcreteAddress<Ipv4, P>
    where
        P: From<Ipv4Addr>,
    {
        fn from(addr: Ipv4Addr) -> Self {
            Self::new(addr.into())
        }
    }

    impl<P: AddressPrimitive<Ipv6>> From<Ipv6Addr> for ConcreteAddress<Ipv6, P>
    where
        P: From<Ipv6Addr>,
    {
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

    impl<A: Afi, P: AddressPrimitive<A>> Arbitrary for ConcreteAddress<A, P>
    where
        A: 'static,
        P: Arbitrary + 'static,
        StrategyFor<P>: 'static,
    {
        type Parameters = ParamsFor<P>;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            any_with::<P>(params).prop_map(Self::new).boxed()
        }
    }
}
