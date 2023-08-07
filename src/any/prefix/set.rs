use core::cmp::Ordering;
use core::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Sub};

use num_traits::{One, Zero};

use super::{Prefix, Range};
use crate::{
    concrete::{self, Ipv4, Ipv6},
    traits,
};

/// A collection of mixed IPv4 and IPv6 prefixes.
///
/// See also [`traits::PrefixSet`] and [`concrete::PrefixSet`].
///
/// # Examples
///
/// ```
/// # use core::str::FromStr;
/// # use ip::{Any, Error, Prefix, PrefixRange, PrefixSet, traits::PrefixSet as _};
/// let set: PrefixSet<Any> = ["192.0.2.0/24,25,26", "2001:db8::/48,52,52"]
///     .into_iter()
///     .map(PrefixRange::<Any>::from_str)
///     .collect::<Result<_, _>>()?;
/// assert_eq!(set.len(), 6 + 16);
/// # Ok::<_, Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Set {
    ipv4: concrete::PrefixSet<Ipv4>,
    ipv6: concrete::PrefixSet<Ipv6>,
}

impl Set {
    fn aggregate(&mut self) -> &mut Self {
        _ = self.ipv4.aggregate();
        _ = self.ipv6.aggregate();
        self
    }

    /// Partition the prefix set by address family.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::str::FromStr;
    /// # use ip::{Any, Error, Prefix, PrefixSet, traits::PrefixSet as _};
    /// let set: PrefixSet<Any> = ["192.0.2.0/24", "2001:db8::/32"]
    ///     .into_iter()
    ///     .map(Prefix::<Any>::from_str)
    ///     .collect::<Result<_, _>>()?;
    /// let (_, ipv6) = set.partition();
    /// let mut ipv6_prefixes = ipv6.prefixes();
    /// assert_eq!(
    ///     ipv6_prefixes.next().map(|p| p.to_string()),
    ///     Some("2001:db8::/32".to_string())
    /// );
    /// assert_eq!(ipv6_prefixes.next(), None);
    /// # Ok::<_, Error>(())
    /// ```
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn partition(self) -> (concrete::PrefixSet<Ipv4>, concrete::PrefixSet<Ipv6>) {
        (self.ipv4, self.ipv6)
    }
}

#[derive(Debug)]
pub struct Prefixes<'a> {
    ipv4: <concrete::PrefixSet<Ipv4> as traits::PrefixSet<'a>>::Prefixes,
    ipv6: <concrete::PrefixSet<Ipv6> as traits::PrefixSet<'a>>::Prefixes,
}

impl Iterator for Prefixes<'_> {
    type Item = Prefix;

    fn next(&mut self) -> Option<Self::Item> {
        self.ipv4
            .next()
            .map(Prefix::Ipv4)
            .or_else(|| self.ipv6.next().map(Prefix::Ipv6))
    }
}

#[derive(Debug)]
pub struct Ranges<'a> {
    ipv4: <concrete::PrefixSet<Ipv4> as traits::PrefixSet<'a>>::Ranges,
    ipv6: <concrete::PrefixSet<Ipv6> as traits::PrefixSet<'a>>::Ranges,
}

impl Iterator for Ranges<'_> {
    type Item = Range;

    fn next(&mut self) -> Option<Self::Item> {
        self.ipv4
            .next()
            .map(Self::Item::Ipv4)
            .or_else(|| self.ipv6.next().map(Self::Item::Ipv6))
    }
}

impl<'a> traits::PrefixSet<'a> for Set {
    type Prefix = Prefix;
    type Range = Range;
    type Prefixes = Prefixes<'a>;
    type Ranges = Ranges<'a>;

    fn prefixes(&'a self) -> Self::Prefixes {
        Self::Prefixes {
            ipv4: self.ipv4.prefixes(),
            ipv6: self.ipv6.prefixes(),
        }
    }

    fn ranges(&'a self) -> Self::Ranges {
        Self::Ranges {
            ipv4: self.ipv4.ranges(),
            ipv6: self.ipv6.ranges(),
        }
    }

    fn contains(&self, prefix: Self::Prefix) -> bool {
        match prefix {
            Self::Prefix::Ipv4(prefix) => self.ipv4.contains(prefix),
            Self::Prefix::Ipv6(prefix) => self.ipv6.contains(prefix),
        }
    }
}

impl From<concrete::PrefixSet<Ipv4>> for Set {
    fn from(value: concrete::PrefixSet<Ipv4>) -> Self {
        Self {
            ipv4: value,
            ..Default::default()
        }
    }
}

impl From<concrete::PrefixSet<Ipv6>> for Set {
    fn from(value: concrete::PrefixSet<Ipv6>) -> Self {
        Self {
            ipv6: value,
            ..Default::default()
        }
    }
}

impl Extend<Prefix> for Set {
    #[allow(unused_results)]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Prefix>,
    {
        iter.into_iter().for_each(|prefix| match prefix {
            Prefix::Ipv4(prefix) => {
                self.ipv4.insert_only(prefix);
            }
            Prefix::Ipv6(prefix) => {
                self.ipv6.insert_only(prefix);
            }
        });
        self.aggregate();
    }
}

impl Extend<Range> for Set {
    #[allow(unused_results)]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Range>,
    {
        iter.into_iter().for_each(|range| match range {
            Range::Ipv4(range) => {
                self.ipv4.insert_only(range);
            }
            Range::Ipv6(range) => {
                self.ipv6.insert_only(range);
            }
        });
        self.aggregate();
    }
}

impl<T> FromIterator<T> for Set
where
    Self: Extend<T>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut this = Self::default();
        this.extend(iter);
        this
    }
}

impl One for Set {
    fn one() -> Self {
        Self {
            ipv4: concrete::PrefixSet::one(),
            ipv6: concrete::PrefixSet::one(),
        }
    }
}

impl Zero for Set {
    fn zero() -> Self {
        Self {
            ipv4: concrete::PrefixSet::zero(),
            ipv6: concrete::PrefixSet::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

impl BitAnd for Set {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            ipv4: self.ipv4 & rhs.ipv4,
            ipv6: self.ipv6 & rhs.ipv6,
        }
    }
}

impl BitOr for Set {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            ipv4: self.ipv4 | rhs.ipv4,
            ipv6: self.ipv6 | rhs.ipv6,
        }
    }
}

impl BitXor for Set {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            ipv4: self.ipv4 ^ rhs.ipv4,
            ipv6: self.ipv6 ^ rhs.ipv6,
        }
    }
}

impl Not for Set {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            ipv4: !self.ipv4,
            ipv6: !self.ipv6,
        }
    }
}

impl Add for Set {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            ipv4: self.ipv4 + rhs.ipv4,
            ipv6: self.ipv6 + rhs.ipv6,
        }
    }
}

impl Mul for Set {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            ipv4: self.ipv4 * rhs.ipv4,
            ipv6: self.ipv6 * rhs.ipv6,
        }
    }
}

impl Sub for Set {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            ipv4: self.ipv4 - rhs.ipv4,
            ipv6: self.ipv6 - rhs.ipv6,
        }
    }
}

impl PartialOrd for Set {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::{Equal, Greater, Less};
        match (
            self.ipv4.partial_cmp(&other.ipv4),
            self.ipv6.partial_cmp(&other.ipv6),
        ) {
            (Some(ord4), Some(ord6)) => match (ord4, ord6) {
                (Equal, Equal) => Some(Equal),
                (Less | Equal, Less | Equal) => Some(Less),
                (Greater | Equal, Greater | Equal) => Some(Greater),
                _ => None,
            },
            _ => None,
        }
    }
}
