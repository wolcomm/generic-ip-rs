use core::fmt;
use core::ops::Neg;

use crate::{
    any,
    error::{err, Error, Kind},
    traits::{
        self,
        primitive::{self, Address as _, Length as _},
        Afi,
    },
    Ipv4, Ipv6,
};

use super::impl_try_from_any;

#[allow(clippy::wildcard_imports)]
mod private {
    use super::*;

    /// An IP prefix length guaranteed to be within appropriate bounds for
    /// address family `A`.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct PrefixLength<A: Afi>(<A::Primitive as primitive::Address<A>>::Length);

    impl<A: Afi> PrefixLength<A> {
        /// Maximum valid value of [`PrefixLength<A>`].
        pub const MAX: Self = Self(A::Primitive::MAX_LENGTH);

        /// Construct a new [`PrefixLength<A>`] from an integer primitive
        /// appropriate to `A`.
        ///
        /// # Errors
        ///
        /// Fails if `n` is outside of the range [`Afi::MIN_LENGTH`] to
        /// [`Afi::MAX_LENGTH`] inclusive (for `A as Afi`).
        pub fn from_primitive(
            n: <A::Primitive as primitive::Address<A>>::Length,
        ) -> Result<Self, Error> {
            if A::Primitive::MIN_LENGTH <= n && n <= A::Primitive::MAX_LENGTH {
                Ok(Self(n))
            } else {
                Err(err!(Kind::PrefixLength))
            }
        }

        /// Get the inner integer val, consuming `self`.
        pub const fn into_primitive(self) -> <A::Primitive as primitive::Address<A>>::Length {
            self.0
        }
    }
}

pub use self::private::PrefixLength;

impl<A: Afi> PrefixLength<A> {
    /// Returns a new [`PrefixLength<A>`] that is one less than `self` unless
    /// `self` is already the minimum possible value.
    ///
    /// # Errors
    ///
    /// An [`Error`] of kind [`error::Kind::PrefixLength`] is returned if self
    /// is equal to zero.
    pub fn decrement(self) -> Result<Self, Error> {
        let l = self.into_primitive();
        if l > <A::Primitive as primitive::Address<A>>::Length::ZERO {
            Self::from_primitive(l - <A::Primitive as primitive::Address<A>>::Length::ONE)
        } else {
            Err(err!(Kind::PrefixLength))
        }
    }
}

impl<A: Afi> traits::PrefixLength for PrefixLength<A> {}

impl_try_from_any! {
    any::PrefixLength {
        any::PrefixLength::Ipv4 => PrefixLength<Ipv4>,
        any::PrefixLength::Ipv6 => PrefixLength<Ipv6>,
    }
}

impl<A: Afi> fmt::Display for PrefixLength<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.into_primitive().fmt(f)
    }
}

impl<A: Afi> Neg for PrefixLength<A> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        // ok to unwrap since 0 <= self.0 <= A::MAX_LENGTH
        Self::from_primitive(A::Primitive::MAX_LENGTH - self.into_primitive()).unwrap()
    }
}

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::Arbitrary,
    strategy::{BoxedStrategy, Strategy},
};

#[cfg(any(test, feature = "arbitrary"))]
impl<A: Afi> Arbitrary for PrefixLength<A>
where
    <A::Primitive as primitive::Address<A>>::Length: 'static,
    core::ops::RangeInclusive<<A::Primitive as primitive::Address<A>>::Length>:
        Strategy<Value = <A::Primitive as primitive::Address<A>>::Length>,
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (A::Primitive::MIN_LENGTH..=A::Primitive::MAX_LENGTH)
            .prop_map(|l| Self::from_primitive(l).unwrap())
            .boxed()
    }
}
