use core::cmp::{max, min, Ordering};
use core::fmt;
use core::ops::RangeInclusive;

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any_with, Arbitrary, ParamsFor},
    strategy::{BoxedStrategy, Just, Strategy},
};

use crate::{
    any,
    error::{err, Error, Kind},
    traits::{self, Afi},
};

#[cfg(any(test, feature = "arbitrary"))]
use crate::traits::primitive;

use super::{impl_try_from_any, Ipv4, Ipv6, Prefix, PrefixLength};

mod private {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    /// A set of [`Prefix<A>`] covered by a common super-prefix, each having a
    /// pref-length within a contigious range.
    #[derive(Clone, Debug, Hash, PartialEq, Eq)]
    pub struct Range<A: Afi> {
        prefix: Prefix<A>,
        len_range: RangeInclusive<PrefixLength<A>>,
    }

    impl<A: Afi> Range<A> {
        /// Construct a new [`Self`] from a convering [`Prefix<A>`] and an
        /// inclusive range of [`PrefixLength`]..
        ///
        /// # Errors
        ///
        /// Construction will fail if either:
        ///
        /// - `prefix.length() > len_range.start()`; or
        /// - `len_range.start() > len_range.end()`
        pub fn new(
            prefix: Prefix<A>,
            len_range: RangeInclusive<PrefixLength<A>>,
        ) -> Result<Self, Error> {
            if &prefix.length() <= len_range.start() && len_range.start() <= len_range.end() {
                Ok(Self { prefix, len_range })
            } else {
                Err(err!(Kind::PrefixLengthRange))
            }
        }

        pub const fn prefix(&self) -> Prefix<A> {
            self.prefix
        }

        pub const fn lower(&self) -> PrefixLength<A> {
            *self.len_range.start()
        }

        pub const fn upper(&self) -> PrefixLength<A> {
            *self.len_range.end()
        }
    }
}

pub use self::private::Range;

impl<A: Afi> traits::PrefixRange for Range<A> {
    type Prefix = Prefix<A>;
    type Length = PrefixLength<A>;

    fn prefix(&self) -> Self::Prefix {
        self.prefix()
    }

    fn lower(&self) -> Self::Length {
        self.lower()
    }

    fn upper(&self) -> Self::Length {
        self.upper()
    }

    fn with_length_range(self, len_range: RangeInclusive<Self::Length>) -> Option<Self> {
        let lower = max(self.lower(), *len_range.start());
        let upper = min(self.upper(), *len_range.end());
        Self::new(self.prefix(), lower..=upper).ok()
    }
}

#[allow(clippy::fallible_impl_from)]
impl<A: Afi> From<Prefix<A>> for Range<A> {
    fn from(prefix: Prefix<A>) -> Self {
        // OK to unwrap here as we can guarantee the checks in `new()` will
        // pass.
        Self::new(prefix, prefix.length()..=prefix.length()).unwrap()
    }
}

impl_try_from_any! {
    any::PrefixRange {
        any::PrefixRange::Ipv4 => Range<Ipv4>,
        any::PrefixRange::Ipv6 => Range<Ipv6>,
    }
}

impl<A: Afi> fmt::Display for Range<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}^{}-{}", self.prefix(), self.lower(), self.upper())
    }
}

impl<A: Afi> PartialOrd for Range<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.prefix().partial_cmp(&other.prefix()) {
            _ if self == other => Some(Ordering::Equal),
            Some(Ordering::Less | Ordering::Equal)
                if other.lower() <= self.lower() && self.upper() <= other.upper() =>
            {
                Some(Ordering::Less)
            }
            Some(Ordering::Greater | Ordering::Equal)
                if self.lower() <= other.lower() && other.upper() <= self.upper() =>
            {
                Some(Ordering::Greater)
            }
            _ => None,
        }
    }
}

#[cfg(any(test, feature = "arbitrary"))]
impl<A> Arbitrary for Range<A>
where
    A: Afi + 'static,
    A::Primitive: Arbitrary,
    RangeInclusive<<A::Primitive as primitive::Address<A>>::Length>:
        Strategy<Value = <A::Primitive as primitive::Address<A>>::Length>,
{
    type Parameters = ParamsFor<Prefix<A>>;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
        any_with::<Prefix<A>>(params)
            .prop_flat_map(|prefix| {
                (
                    Just(prefix),
                    (prefix.length().into_primitive()
                        ..=<A::Primitive as primitive::Address<A>>::MAX_LENGTH)
                        .prop_flat_map(|lower| {
                            (
                                Just(lower),
                                lower..=<A::Primitive as primitive::Address<A>>::MAX_LENGTH,
                            )
                        })
                        .prop_map(|(lower, upper)| {
                            <A as traits::AfiClass>::PrefixLength::from_primitive(lower).unwrap()
                                ..=<A as traits::AfiClass>::PrefixLength::from_primitive(upper)
                                    .unwrap()
                        }),
                )
            })
            .prop_map(|(prefix, len_range)| Self::new(prefix, len_range).unwrap())
            .boxed()
    }
}
