use core::fmt;
use core::ops::Neg;

use crate::{
    error::{err, Error, ErrorKind},
    traits::{
        self,
        primitive::{self, Address as _, Length as _},
        Afi,
    },
};

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
        /// Fails if `n` is outside of the range [`Afi::MIN_LENGTH`] to
        /// [`Afi::MAX_LENGTH`] inclusive (for `A as Afi`).
        pub fn from_primitive(
            n: <A::Primitive as primitive::Address<A>>::Length,
        ) -> Result<Self, Error<'static, A>> {
            if A::Primitive::MIN_LENGTH <= n && n <= A::Primitive::MAX_LENGTH {
                Ok(Self(n))
            } else {
                Err(err!(ErrorKind::PrefixLength(n)))
            }
        }

        /// Get the inner integer val, consuming `self`.
        pub fn into_primitive(self) -> <A::Primitive as primitive::Address<A>>::Length {
            self.0
        }
    }
}

pub use self::private::PrefixLength;

impl<A: Afi> PrefixLength<A> {
    pub fn decrement(self) -> Result<Self, Error<'static, A>> {
        Self::from_primitive(
            self.into_primitive() - <A::Primitive as primitive::Address<A>>::Length::ONE,
        )
    }
}

impl<A: Afi> traits::PrefixLength for PrefixLength<A> {}

impl<A: Afi> fmt::Display for PrefixLength<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
mod arbitrary {
    use super::*;

    use core::ops::RangeInclusive;

    use proptest::{
        arbitrary::Arbitrary,
        strategy::{BoxedStrategy, Strategy},
    };

    impl<A: Afi> Arbitrary for PrefixLength<A>
    where
        <A::Primitive as primitive::Address<A>>::Length: 'static,
        RangeInclusive<<A::Primitive as primitive::Address<A>>::Length>:
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
}
