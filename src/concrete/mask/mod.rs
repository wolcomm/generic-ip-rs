use core::fmt::{self};
use core::ops::{Shl, Shr};

use super::{impl_try_from_any, PrefixLength};
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
use self::types::{Host, Net, Type};

/// An IP Netmask.
pub type Netmask<A> = Mask<Net, A>;

/// An IP Hostmask.
pub type Hostmask<A> = Mask<Host, A>;

impl<A: Afi, T: Type> Mask<T, A> {
    /// The "all-zeros" mask.
    pub const ZEROS: Self = Self::new(A::Primitive::ZERO);

    /// The "all-ones" mask.
    pub const ONES: Self = Self::new(A::Primitive::ONES);
}

impl<T: Type, A: Afi> traits::Mask for Mask<T, A> {}
impl<A: Afi> traits::Netmask for Netmask<A> {}
impl<A: Afi> traits::Hostmask for Hostmask<A> {}

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
impl<A: Afi, T: Type> Arbitrary for Mask<T, A>
where
    A: 'static,
    A::Primitive: 'static,
    T: 'static,
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
