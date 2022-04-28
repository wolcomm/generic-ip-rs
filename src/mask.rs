use core::fmt::Debug;
use core::ops::{Shl, Shr};

use crate::{
    af::{Afi, Ipv4, Ipv6},
    prefix::ConcretePrefixLength,
    primitive::AddressPrimitive,
};

pub trait Type: Copy + Debug {}

#[derive(Clone, Copy, Debug)]
pub enum Net {}
impl Type for Net {}

#[derive(Clone, Copy, Debug)]
pub enum Host {}
impl Type for Host {}

mod private {
    use super::*;

    use core::marker::PhantomData;

    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub struct ConcreteMask<T: Type, A: Afi>(A::AddressPrimitive, PhantomData<T>);

    impl<A: Afi, T: Type> ConcreteMask<T, A> {
        pub const ZEROS: Self = Self(A::AddressPrimitive::ZERO, PhantomData);
        pub const ONES: Self = Self(A::AddressPrimitive::ONES, PhantomData);

        pub fn new(bits: A::AddressPrimitive) -> Self {
            Self(bits, PhantomData)
        }

        pub fn into_primitive(self) -> A::AddressPrimitive {
            self.0
        }
    }
}

pub use self::private::ConcreteMask;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum AnyMask<T: Type> {
    Ipv4(ConcreteMask<T, Ipv4>),
    Ipv6(ConcreteMask<T, Ipv6>),
}

pub trait MaskI {}
impl<T: Type, A: Afi> MaskI for ConcreteMask<T, A> {}
impl<T: Type> MaskI for AnyMask<T> {}

/// An IP Netmask.
pub type ConcreteNetmask<A> = ConcreteMask<Net, A>;
pub type AnyNetmask = AnyMask<Net>;

/// An IP Hostmask.
pub type ConcreteHostmask<A> = ConcreteMask<Host, A>;
pub type AnyHostmask = AnyMask<Host>;

impl<T: Type> From<ConcreteMask<T, Ipv4>> for AnyMask<T> {
    fn from(mask: ConcreteMask<T, Ipv4>) -> Self {
        Self::Ipv4(mask)
    }
}

impl<T: Type> From<ConcreteMask<T, Ipv6>> for AnyMask<T> {
    fn from(mask: ConcreteMask<T, Ipv6>) -> Self {
        Self::Ipv6(mask)
    }
}

impl<A: Afi, T: Type> Shl<ConcretePrefixLength<A>> for ConcreteMask<T, A> {
    type Output = Self;

    fn shl(self, rhs: ConcretePrefixLength<A>) -> Self::Output {
        if rhs == ConcretePrefixLength::<A>::MAX {
            Self::ZEROS
        } else {
            Self::new(Self::into_primitive(self) << rhs.into_primitive())
        }
    }
}

impl<A: Afi, T: Type> Shr<ConcretePrefixLength<A>> for ConcreteMask<T, A> {
    type Output = Self;

    fn shr(self, rhs: ConcretePrefixLength<A>) -> Self::Output {
        if rhs == ConcretePrefixLength::<A>::MAX {
            Self::ZEROS
        } else {
            Self::new(Self::into_primitive(self) >> rhs.into_primitive())
        }
    }
}

impl<A: Afi> From<ConcretePrefixLength<A>> for ConcreteNetmask<A> {
    fn from(len: ConcretePrefixLength<A>) -> Self {
        Self::ONES << -len
    }
}

impl<A: Afi> From<ConcretePrefixLength<A>> for ConcreteHostmask<A> {
    fn from(len: ConcretePrefixLength<A>) -> Self {
        Self::ONES >> len
    }
}

mod fmt {
    use super::*;

    use core::fmt;

    use crate::fmt::AddressDisplay;

    impl<A: Afi, T: Type> fmt::Display for ConcreteMask<T, A>
    where
        A::AddressPrimitive: AddressDisplay<A>,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.into_primitive().fmt_addr(f)
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

    impl<A: Afi, T: Type> Arbitrary for ConcreteMask<T, A>
    where
        A: 'static,
        A::AddressPrimitive: 'static,
        T: 'static,
        Self: From<ConcretePrefixLength<A>>,
        ConcretePrefixLength<A>: Arbitrary,
        StrategyFor<ConcretePrefixLength<A>>: 'static,
    {
        type Parameters = ParamsFor<ConcretePrefixLength<A>>;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            any_with::<ConcretePrefixLength<A>>(params)
                .prop_map(ConcreteMask::from)
                .boxed()
        }
    }
}
