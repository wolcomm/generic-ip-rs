use core::fmt;
use core::ops::Neg;

use crate::{
    af::{Afi, Ipv4, Ipv6},
    error::{err, Error, ErrorKind},
    primitive::{AddressPrimitive, WidthPrimitive},
};

mod private {
    use super::*;

    /// An IP prefix length guaranteed to be within appropriate bounds for
    /// address family `A`.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ConcretePrefixLength<A: Afi>(<A::AddressPrimitive as AddressPrimitive<A>>::Width);

    impl<A: Afi> ConcretePrefixLength<A> {
        /// Maximum valid value of [`PrefixLength<A>`].
        pub const MAX: Self = Self(A::AddressPrimitive::MAX_LENGTH);

        /// Construct a new [`PrefixLength<A>`] from an integer primitive
        /// appropriate to `A`.
        ///
        /// Fails if `n` is outside of the range [`Afi::MIN_LENGTH`] to
        /// [`Afi::MAX_LENGTH`] inclusive (for `A as Afi`).
        pub fn from_primitive(
            n: <A::AddressPrimitive as AddressPrimitive<A>>::Width,
        ) -> Result<Self, Error<'static, A>> {
            if A::AddressPrimitive::MIN_LENGTH <= n && n <= A::AddressPrimitive::MAX_LENGTH {
                Ok(Self(n))
            } else {
                Err(err!(ErrorKind::PrefixLength(n)))
            }
        }

        /// Get the inner integer val, consuming `self`.
        pub fn into_primitive(self) -> <A::AddressPrimitive as AddressPrimitive<A>>::Width {
            self.0
        }
    }
}

pub use self::private::ConcretePrefixLength;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub enum AnyPrefixLength {
    Ipv4(ConcretePrefixLength<Ipv4>),
    Ipv6(ConcretePrefixLength<Ipv6>),
}

pub trait PrefixLengthI {}
impl<A: Afi> PrefixLengthI for ConcretePrefixLength<A> {}
impl PrefixLengthI for AnyPrefixLength {}

impl From<ConcretePrefixLength<Ipv4>> for AnyPrefixLength {
    fn from(length: ConcretePrefixLength<Ipv4>) -> Self {
        Self::Ipv4(length)
    }
}

impl From<ConcretePrefixLength<Ipv6>> for AnyPrefixLength {
    fn from(length: ConcretePrefixLength<Ipv6>) -> Self {
        Self::Ipv6(length)
    }
}

impl<A: Afi> ConcretePrefixLength<A> {
    pub fn decrement(self) -> Result<Self, Error<'static, A>> {
        Self::from_primitive(
            self.into_primitive() - <A::AddressPrimitive as AddressPrimitive<A>>::Width::ONE,
        )
    }
}

impl<A: Afi> fmt::Display for ConcretePrefixLength<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.into_primitive().fmt(f)
    }
}

impl<A: Afi> Neg for ConcretePrefixLength<A> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        // ok to unwrap since 0 <= self.0 <= A::MAX_LENGTH
        Self::from_primitive(A::AddressPrimitive::MAX_LENGTH - self.into_primitive()).unwrap()
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

    impl<A: Afi> Arbitrary for ConcretePrefixLength<A>
    where
        <A::AddressPrimitive as AddressPrimitive<A>>::Width: 'static,
        RangeInclusive<<A::AddressPrimitive as AddressPrimitive<A>>::Width>:
            Strategy<Value = <A::AddressPrimitive as AddressPrimitive<A>>::Width>,
    {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (A::AddressPrimitive::MIN_LENGTH..=A::AddressPrimitive::MAX_LENGTH)
                .prop_map(|l| Self::from_primitive(l).unwrap())
                .boxed()
        }
    }
}
