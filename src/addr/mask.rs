use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Shl, Shr};

use crate::{af::Afi, prefix::PrefixLength, primitive::AddressPrimitive};

pub trait Type: Copy + Debug {}

#[derive(Clone, Copy, Debug)]
pub enum Net {}
impl Type for Net {}

#[derive(Clone, Copy, Debug)]
pub enum Host {}
impl Type for Host {}

mod private {
    use super::*;

    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub struct Mask<A: Afi, T: Type>(A::Addr, PhantomData<T>);

    impl<A: Afi, T: Type> Mask<A, T> {
        pub const ZEROS: Self = Self(A::Addr::ZERO, PhantomData);
        pub const ONES: Self = Self(A::Addr::ONES, PhantomData);

        pub fn new(bits: A::Addr) -> Self {
            Self(bits, PhantomData)
        }

        pub fn into_primitive(self) -> A::Addr {
            self.0
        }
    }
}

pub use self::private::Mask;

/// An IP Netmask.
pub type Netmask<A> = Mask<A, Net>;

/// An IP Hostmask.
pub type Hostmask<A> = Mask<A, Host>;

impl<A: Afi, T: Type> Shl<PrefixLength<A>> for Mask<A, T> {
    type Output = Self;

    fn shl(self, rhs: PrefixLength<A>) -> Self::Output {
        if rhs == PrefixLength::<A>::MAX {
            Self::ZEROS
        } else {
            Self::new(Self::into_primitive(self) << rhs.into_primitive())
        }
    }
}

impl<A: Afi, T: Type> Shr<PrefixLength<A>> for Mask<A, T> {
    type Output = Self;

    fn shr(self, rhs: PrefixLength<A>) -> Self::Output {
        if rhs == PrefixLength::<A>::MAX {
            Self::ZEROS
        } else {
            Self::new(Self::into_primitive(self) >> rhs.into_primitive())
        }
    }
}

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

mod fmt {
    use super::*;

    use core::fmt;

    use crate::fmt::AddressDisplay;

    impl<A: Afi, T: Type> fmt::Display for Mask<A, T>
    where
        A::Addr: AddressDisplay<A>,
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

    impl<A: Afi, T: Type> Arbitrary for Mask<A, T>
    where
        A: 'static,
        T: 'static,
        Self: From<PrefixLength<A>>,
        PrefixLength<A>: Arbitrary,
        StrategyFor<PrefixLength<A>>: 'static,
    {
        type Parameters = ParamsFor<PrefixLength<A>>;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            any_with::<PrefixLength<A>>(params)
                .prop_map(Mask::from)
                .boxed()
        }
    }
}
