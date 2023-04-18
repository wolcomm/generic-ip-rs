use core::cmp::{max, min, Ordering};
use core::fmt;
use core::ops::RangeInclusive;
use core::str::FromStr;

#[cfg(any(test, feature = "arbitrary"))]
use proptest::{
    arbitrary::{any_with, Arbitrary, ParamsFor},
    strategy::{BoxedStrategy, Just, Strategy},
};

use super::{impl_try_from_any, Address, Ipv4, Ipv6, Prefix, PrefixLength, Subprefixes};
#[cfg(any(test, feature = "arbitrary"))]
use crate::traits::primitive;
use crate::{
    any,
    error::{err, Error, Kind},
    traits::{self, primitive::Address as _, Afi, Prefix as _, PrefixLength as _},
};

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
        /// The range containing all prefixes of address family `A`.
        pub const ALL: Self = Self {
            prefix: Prefix::DEFAULT,
            len_range: PrefixLength::MIN..=PrefixLength::MAX,
        };

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

        /// Return the covering super-prefix of `self`.
        pub const fn prefix(&self) -> Prefix<A> {
            self.prefix
        }

        /// Return the lower bound [`PrefixLength`] of `self`.
        pub const fn lower(&self) -> PrefixLength<A> {
            *self.len_range.start()
        }

        /// Return the upper bound [`PrefixLength`] of `self`.
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

    fn with_intersection(self, len_range: RangeInclusive<Self::Length>) -> Option<Self> {
        let lower = max(self.lower(), *len_range.start());
        let upper = min(self.upper(), *len_range.end());
        Self::new(self.prefix(), lower..=upper).ok()
    }

    fn with_length_range(self, len_range: RangeInclusive<Self::Length>) -> Option<Self> {
        let lower = max(self.lower(), *len_range.start());
        let upper = *len_range.end();
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

impl<A: Afi> FromStr for Range<A> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        A::Primitive::parse_range(s).and_then(|(addr, len, l, u)| {
            let (lower, upper) = (
                PrefixLength::from_primitive(l)?,
                PrefixLength::from_primitive(u)?,
            );
            Self::new(
                Prefix::new(Address::new(addr), PrefixLength::from_primitive(len)?),
                lower..=upper,
            )
        })
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

impl<A: Afi> IntoIterator for Range<A> {
    type Item = Prefix<A>;
    type IntoIter = IntoIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        // Safe to unwrap here since we are passing self.lower() which is
        // guaranteed to be within the bounds check.
        let current_iter = Some(self.prefix().subprefixes(self.lower()).unwrap());
        Self::IntoIter {
            base: self.prefix(),
            current_length: self.lower(),
            upper_length: self.upper(),
            current_iter,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntoIter<A: Afi> {
    base: Prefix<A>,
    current_length: PrefixLength<A>,
    upper_length: PrefixLength<A>,
    current_iter: Option<Subprefixes<A>>,
}

impl<A: Afi> Iterator for IntoIter<A> {
    type Item = Prefix<A>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(mut subprefixes) = self.current_iter.take() {
                if let Some(prefix) = subprefixes.next() {
                    self.current_iter = Some(subprefixes);
                    break Some(prefix);
                }
                self.current_length = self
                    .current_length
                    .increment()
                    .ok()
                    .filter(|length| length <= &self.upper_length)?;
                self.current_iter = self.base.subprefixes(self.current_length).ok();
            } else {
                break None;
            }
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
