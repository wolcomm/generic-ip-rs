use core::fmt;
use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

use super::{impl_try_from_any, Address, PrefixLength};
use crate::{
    any,
    fmt::AddressDisplay,
    traits::{self, primitive::Address as _, Afi},
    Ipv4, Ipv6,
};

mod private;
pub use self::private::Mask;

/// Types of IP address mask.
pub mod types;
use self::types::{Bit, Host, Net, Type};

/// An IP Netmask.
pub type Netmask<A> = Mask<Net, A>;

/// An IP Hostmask.
pub type Hostmask<A> = Mask<Host, A>;

/// An address bit-mask.
pub type Bitmask<A> = Mask<Bit, A>;

impl<A: Afi, T: Type> Mask<T, A> {
    /// The "all-zeros" mask.
    pub const ZEROS: Self = Self::new(A::Primitive::ZERO);

    /// The "all-ones" mask.
    pub const ONES: Self = Self::new(A::Primitive::ONES);
}

impl<T: Type, A: Afi> traits::Mask for Mask<T, A> {}
impl<A: Afi> traits::Netmask for Netmask<A> {}
impl<A: Afi> traits::Hostmask for Hostmask<A> {}
impl<A: Afi> traits::Bitmask for Bitmask<A> {}

impl<A: Afi> From<PrefixLength<A>> for Netmask<A> {
    fn from(len: PrefixLength<A>) -> Self {
        Self::ONES << -len
    }
}

impl<A: Afi> From<PrefixLength<A>> for Hostmask<A> {
    fn from(len: PrefixLength<A>) -> Self {
        Self::ONES >> len
    }
}

impl<A: Afi> From<Address<A>> for Bitmask<A> {
    fn from(addr: Address<A>) -> Self {
        Self::new(addr.into_primitive())
    }
}

impl_try_from_any! {
    any::Netmask {
        any::Netmask::Ipv4 => Netmask<Ipv4>,
        any::Netmask::Ipv6 => Netmask<Ipv6>,
    }
}

impl_try_from_any! {
    any::Hostmask {
        any::Hostmask::Ipv4 => Hostmask<Ipv4>,
        any::Hostmask::Ipv6 => Hostmask<Ipv6>,
    }
}

impl<A: Afi, T: Type> Shl<PrefixLength<A>> for Mask<T, A> {
    type Output = Self;

    fn shl(self, rhs: PrefixLength<A>) -> Self::Output {
        if rhs == PrefixLength::<A>::MAX {
            Self::ZEROS
        } else {
            Self::new(Self::into_primitive(self) << rhs.into_primitive())
        }
    }
}

impl<A: Afi, T: Type> Shr<PrefixLength<A>> for Mask<T, A> {
    type Output = Self;

    fn shr(self, rhs: PrefixLength<A>) -> Self::Output {
        if rhs == PrefixLength::<A>::MAX {
            Self::ZEROS
        } else {
            Self::new(Self::into_primitive(self) >> rhs.into_primitive())
        }
    }
}

impl<A: Afi, T1: Type, T2: Type> BitAnd<Mask<T2, A>> for Mask<T1, A> {
    type Output = Bitmask<A>;
    fn bitand(self, rhs: Mask<T2, A>) -> Self::Output {
        Self::Output::new(self.into_primitive() & rhs.into_primitive())
    }
}

impl<A: Afi, T: Type> BitOr<Mask<T, A>> for Bitmask<A> {
    type Output = Self;
    fn bitor(self, rhs: Mask<T, A>) -> Self {
        Self::new(self.into_primitive() | rhs.into_primitive())
    }
}

impl<A: Afi, T: Type> BitXor<Mask<T, A>> for Bitmask<A> {
    type Output = Self;
    fn bitxor(self, rhs: Mask<T, A>) -> Self {
        Self::new(self.into_primitive() ^ rhs.into_primitive())
    }
}

impl<A: Afi> Not for Bitmask<A> {
    type Output = Self;
    fn not(self) -> Self {
        Self::new(self.into_primitive().not())
    }
}

// TODO: impl FromStr

impl<A: Afi, T: Type> fmt::Display for Mask<T, A>
where
    A::Primitive: AddressDisplay<A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.into_primitive().fmt_addr(f)
    }
}

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any_with, Arbitrary, ParamsFor, StrategyFor},
    strategy::{BoxedStrategy, Strategy},
};

#[cfg(any(test, feature = "arbitrary"))]
impl<A, T> Arbitrary for Mask<T, A>
where
    A: Afi + 'static,
    A::Primitive: 'static,
    T: Type + 'static,
    Self: From<PrefixLength<A>>,
    PrefixLength<A>: Arbitrary,
    StrategyFor<PrefixLength<A>>: 'static,
{
    type Parameters = ParamsFor<PrefixLength<A>>;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
        any_with::<PrefixLength<A>>(params)
            .prop_map(Self::from)
            .boxed()
    }
}
