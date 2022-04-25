use core::fmt::Debug;
use core::ops::{Shl, Shr};

use crate::{
    af::{Afi, DefaultPrimitive, Ipv4, Ipv6},
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
    pub struct ConcreteMask<T: Type, A: Afi, P: AddressPrimitive<A> = DefaultPrimitive<A>>(
        P,
        PhantomData<A>,
        PhantomData<T>,
    );

    impl<A: Afi, P: AddressPrimitive<A>, T: Type> ConcreteMask<T, A, P> {
        pub const ZEROS: Self = Self(P::ZERO, PhantomData, PhantomData);
        pub const ONES: Self = Self(P::ONES, PhantomData, PhantomData);

        pub fn new(bits: P) -> Self {
            Self(bits, PhantomData, PhantomData)
        }

        pub fn into_primitive(self) -> P {
            self.0
        }
    }
}

pub use self::private::ConcreteMask;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum AnyMask<T: Type, P4 = DefaultPrimitive<Ipv4>, P6 = DefaultPrimitive<Ipv6>>
where
    P4: AddressPrimitive<Ipv4>,
    P6: AddressPrimitive<Ipv6>,
{
    Ipv4(ConcreteMask<T, Ipv4, P4>),
    Ipv6(ConcreteMask<T, Ipv6, P6>),
}

pub trait MaskI {}
impl<T: Type, A: Afi, P: AddressPrimitive<A>> MaskI for ConcreteMask<T, A, P> {}
impl<T: Type, P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>> MaskI for AnyMask<T, P4, P6> {}

/// An IP Netmask.
pub type ConcreteNetmask<A, P = DefaultPrimitive<A>> = ConcreteMask<Net, A, P>;
pub type AnyNetmask<P4 = DefaultPrimitive<Ipv4>, P6 = DefaultPrimitive<Ipv6>> =
    AnyMask<Net, P4, P6>;

/// An IP Hostmask.
pub type ConcreteHostmask<A, P = DefaultPrimitive<A>> = ConcreteMask<Host, A, P>;
pub type AnyHostmask<P4 = DefaultPrimitive<Ipv4>, P6 = DefaultPrimitive<Ipv6>> =
    AnyMask<Host, P4, P6>;

impl<T: Type, P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>>
    From<ConcreteMask<T, Ipv4, P4>> for AnyMask<T, P4, P6>
{
    fn from(mask: ConcreteMask<T, Ipv4, P4>) -> Self {
        Self::Ipv4(mask)
    }
}

impl<T: Type, P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>>
    From<ConcreteMask<T, Ipv6, P6>> for AnyMask<T, P4, P6>
{
    fn from(mask: ConcreteMask<T, Ipv6, P6>) -> Self {
        Self::Ipv6(mask)
    }
}

impl<A: Afi, P: AddressPrimitive<A>, T: Type> Shl<ConcretePrefixLength<A, P>>
    for ConcreteMask<T, A, P>
{
    type Output = Self;

    fn shl(self, rhs: ConcretePrefixLength<A, P>) -> Self::Output {
        if rhs == ConcretePrefixLength::<A, P>::MAX {
            Self::ZEROS
        } else {
            Self::new(Self::into_primitive(self) << rhs.into_primitive())
        }
    }
}

impl<A: Afi, P: AddressPrimitive<A>, T: Type> Shr<ConcretePrefixLength<A, P>>
    for ConcreteMask<T, A, P>
{
    type Output = Self;

    fn shr(self, rhs: ConcretePrefixLength<A, P>) -> Self::Output {
        if rhs == ConcretePrefixLength::<A, P>::MAX {
            Self::ZEROS
        } else {
            Self::new(Self::into_primitive(self) >> rhs.into_primitive())
        }
    }
}

impl<A: Afi, P: AddressPrimitive<A>> From<ConcretePrefixLength<A, P>> for ConcreteNetmask<A, P> {
    fn from(len: ConcretePrefixLength<A, P>) -> Self {
        Self::ONES << -len
    }
}

impl<A: Afi, P: AddressPrimitive<A>> From<ConcretePrefixLength<A, P>> for ConcreteHostmask<A, P> {
    fn from(len: ConcretePrefixLength<A, P>) -> Self {
        Self::ONES >> len
    }
}

mod fmt {
    use super::*;

    use core::fmt;

    use crate::fmt::AddressDisplay;

    impl<A: Afi, P: AddressPrimitive<A>, T: Type> fmt::Display for ConcreteMask<T, A, P>
    where
        P: AddressDisplay<A>,
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

    impl<A: Afi, P: AddressPrimitive<A>, T: Type> Arbitrary for ConcreteMask<T, A, P>
    where
        A: 'static,
        P: 'static,
        T: 'static,
        Self: From<ConcretePrefixLength<A, P>>,
        ConcretePrefixLength<A, P>: Arbitrary,
        StrategyFor<ConcretePrefixLength<A, P>>: 'static,
    {
        type Parameters = ParamsFor<ConcretePrefixLength<A, P>>;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            any_with::<ConcretePrefixLength<A, P>>(params)
                .prop_map(ConcreteMask::from)
                .boxed()
        }
    }
}
