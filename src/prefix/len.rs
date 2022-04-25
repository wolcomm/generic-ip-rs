use core::fmt;
use core::ops::Neg;

use crate::{
    af::{Afi, DefaultPrimitive, Ipv4, Ipv6},
    error::{err, Error, ErrorKind},
    primitive::{AddressPrimitive, WidthPrimitive},
};

mod private {
    use super::*;

    /// An IP prefix length guaranteed to be within appropriate bounds for
    /// address family `A`.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ConcretePrefixLength<A: Afi, P: AddressPrimitive<A> = DefaultPrimitive<A>>(P::Width);

    impl<A: Afi, P: AddressPrimitive<A>> ConcretePrefixLength<A, P> {
        /// Maximum valid value of [`PrefixLength<A>`].
        pub const MAX: Self = Self(P::MAX_LENGTH);

        // TODO: use `Self::new()` to construct these (and move out of `mod
        // private`) once const trait bounds are available in stable rustc
        // (1.61+)
        pub(crate) const LOCALHOST_NET: Self = Self(P::LOCALHOST_NET.1);
        pub(crate) const BENCHMARK_NET: Self = Self(P::BENCHMARK_NET.1);
        pub(crate) const MULTICAST_NET: Self = Self(P::MULTICAST_NET.1);

        /// Construct a new [`PrefixLength<A>`] from an integer primitive
        /// appropriate to `A`.
        ///
        /// Fails if `n` is outside of the range [`Afi::MIN_LENGTH`] to
        /// [`Afi::MAX_LENGTH`] inclusive (for `A as Afi`).
        pub fn from_primitive(n: P::Width) -> Result<Self, Error<'static, A, P>> {
            if P::MIN_LENGTH <= n && n <= P::MAX_LENGTH {
                Ok(Self(n))
            } else {
                Err(err!(ErrorKind::PrefixLength(n)))
            }
        }

        /// Get the inner integer val, consuming `self`.
        pub fn into_primitive(self) -> P::Width {
            self.0
        }
    }
}

pub use self::private::ConcretePrefixLength;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub enum AnyPrefixLength<P4 = DefaultPrimitive<Ipv4>, P6 = DefaultPrimitive<Ipv6>>
where
    P4: AddressPrimitive<Ipv4>,
    P6: AddressPrimitive<Ipv6>,
{
    Ipv4(ConcretePrefixLength<Ipv4, P4>),
    Ipv6(ConcretePrefixLength<Ipv6, P6>),
}

pub trait PrefixLengthI {}
impl<A: Afi, P: AddressPrimitive<A>> PrefixLengthI for ConcretePrefixLength<A, P> {}
impl<P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>> PrefixLengthI
    for AnyPrefixLength<P4, P6>
{
}

impl<P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>> From<ConcretePrefixLength<Ipv4, P4>>
    for AnyPrefixLength<P4, P6>
{
    fn from(length: ConcretePrefixLength<Ipv4, P4>) -> Self {
        Self::Ipv4(length)
    }
}

impl<P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>> From<ConcretePrefixLength<Ipv6, P6>>
    for AnyPrefixLength<P4, P6>
{
    fn from(length: ConcretePrefixLength<Ipv6, P6>) -> Self {
        Self::Ipv6(length)
    }
}

impl<A: Afi, P: AddressPrimitive<A>> ConcretePrefixLength<A, P> {
    pub fn decrement(self) -> Result<Self, Error<'static, A, P>> {
        Self::from_primitive(self.into_primitive() - P::Width::ONE)
    }
}

impl<A: Afi, P: AddressPrimitive<A>> fmt::Display for ConcretePrefixLength<A, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.into_primitive().fmt(f)
    }
}

impl<A: Afi, P: AddressPrimitive<A>> Neg for ConcretePrefixLength<A, P> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        // ok to unwrap since 0 <= self.0 <= A::MAX_LENGTH
        Self::from_primitive(P::MAX_LENGTH - self.into_primitive()).unwrap()
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

    impl<A: Afi, P: AddressPrimitive<A>> Arbitrary for ConcretePrefixLength<A, P>
    where
        P::Width: 'static,
        RangeInclusive<P::Width>: Strategy<Value = P::Width>,
    {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (P::MIN_LENGTH..=P::MAX_LENGTH)
                .prop_map(|l| Self::from_primitive(l).unwrap())
                .boxed()
        }
    }
}
