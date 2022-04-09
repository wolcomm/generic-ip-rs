use core::fmt;
use core::ops::Neg;

use crate::{
    af::Afi,
    error::{err, Error, ErrorKind},
    primitive::WidthOf,
};

mod private {
    use super::*;

    /// An IP prefix length guaranteed to be within appropriate bounds for
    /// address family `A`.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct PrefixLength<A: Afi>(WidthOf<A::Addr>);

    impl<A: Afi> PrefixLength<A> {
        /// Maximum valid value of [`PrefixLength<A>`].
        pub const MAX: Self = Self(A::MAX_LENGTH);

        /// Construct a new [`PrefixLength<A>`] from an integer primitive
        /// appropriate to `A`.
        ///
        /// Fails if `n` is outside of the range [`Afi::MIN_LENGTH`] to
        /// [`Afi::MAX_LENGTH`] inclusive (for `A as Afi`).
        pub fn from_primitive(n: WidthOf<A::Addr>) -> Result<Self, Error<'static, A>> {
            if A::MIN_LENGTH <= n && n <= A::MAX_LENGTH {
                Ok(Self(n))
            } else {
                Err(err!(ErrorKind::PrefixLength(n)))
            }
        }

        /// Get the inner integer val, consuming `self`.
        pub fn into_primitive(self) -> WidthOf<A::Addr> {
            self.0
        }
    }
}

pub use self::private::PrefixLength;

impl<A: Afi> fmt::Display for PrefixLength<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.into_primitive().fmt(f)
    }
}

impl<A: Afi> Neg for PrefixLength<A> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        // ok to unwrap since 0 <= self.0 <= A::MAX_LENGTH
        Self::from_primitive(A::MAX_LENGTH - self.into_primitive()).unwrap()
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
        WidthOf<A::Addr>: 'static,
        RangeInclusive<WidthOf<A::Addr>>: Strategy<Value = WidthOf<A::Addr>>,
    {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (A::MIN_LENGTH..=A::MAX_LENGTH)
                .prop_map(|l| Self::from_primitive(l).unwrap())
                .boxed()
        }
    }
}
