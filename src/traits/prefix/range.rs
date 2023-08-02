use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::ops::RangeInclusive;
use core::str::FromStr;

use super::{Length, Prefix};
use crate::{concrete, error::Error};

/// Address-family independent interface for IP prefix ranges.
///
/// See also [`concrete::PrefixRange<A>`][crate::concrete::PrefixRange] and
/// [`any::PrefixRange`][crate::any::PrefixRange] for address-family specific
/// items.
pub trait Range:
    Clone
    + Debug
    + Display
    + From<Self::Prefix>
    + FromStr<Err = Error>
    + Hash
    + IntoIterator<Item = Self::Prefix>
    + PartialEq
    + Eq
    + PartialOrd
{
    /// The type of IP prefix over which `Self` represents a range.
    type Prefix: Prefix<Length = Self::Length>;

    /// The type used to represent lengths for this IP prefix type.
    type Length: Length;

    /// Return the covering super-prefix of `self`.
    fn prefix(&self) -> Self::Prefix;

    /// Return the lower bound [`Self::Length`] of `self`.
    fn lower(&self) -> Self::Length;

    /// Return the upper bound [`Self::Length`] of `self`.
    fn upper(&self) -> Self::Length;

    /// Construct a new IP prefix-range from the intersection of `self` and
    /// `len_range`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::PrefixRange as _, Ipv6, Prefix, PrefixLength, PrefixRange};
    ///
    /// let lower = PrefixLength::<Ipv6>::from_primitive(52)?;
    /// let upper = PrefixLength::<Ipv6>::from_primitive(56)?;
    ///
    /// let x: PrefixRange<Ipv6> = "2001:db8::/48".parse::<Prefix<Ipv6>>()?.into();
    /// assert_eq!(x.with_intersection(lower..=upper).into_iter().count(), 0,);
    ///
    /// let y: PrefixRange<Ipv6> = "2001:db8::/54".parse::<Prefix<Ipv6>>()?.into();
    /// assert_eq!(y.with_intersection(lower..=upper).into_iter().count(), 1,);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn with_intersection(self, len_range: RangeInclusive<Self::Length>) -> Option<Self>;

    /// Construct a new IP prefix-range consisting of all the more specific
    /// sub-prefixes of `self` with prefix-lengths within `len_range`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::cmp::max;
    ///
    /// use ip::{traits::PrefixRange as _, Address, Ipv4, Prefix, PrefixLength, PrefixRange};
    ///
    /// let addr = "192.0.2.0".parse::<Address<Ipv4>>()?;
    ///
    /// let [l, m, n, p, q]: &[PrefixLength<Ipv4>] = &[24u8, 26, 28, 30, 32]
    ///     .into_iter()
    ///     .map(PrefixLength::<Ipv4>::from_primitive)
    ///     .collect::<Result<Vec<PrefixLength<Ipv4>>, _>>()?
    /// else {
    ///     panic!()
    /// };
    ///
    /// let prefix = Prefix::<Ipv4>::new(addr, *l);
    ///
    /// assert_eq!(
    ///     PrefixRange::<Ipv4>::new(prefix, *m..=*n)?
    ///         .with_length_range(*p..=*q)
    ///         .unwrap(),
    ///     PrefixRange::<Ipv4>::new(prefix, max(*m, *p)..=*q)?,
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn with_length_range(self, len_range: RangeInclusive<Self::Length>) -> Option<Self>;

    /// Construct a new IP prefix-range consisting of all the more-specific
    /// sub-prefixes of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::PrefixRange as _, Any, Prefix, PrefixRange};
    ///
    /// let range: PrefixRange<Any> = "2001:db8::/126".parse::<Prefix<Any>>()?.into();
    ///
    /// assert_eq!(range.or_longer().into_iter().count(), 7,);
    /// # Ok::<(), ip::Error>(())
    /// ```
    #[must_use]
    fn or_longer(self) -> Self {
        let lower = self.lower();
        let upper = self.prefix().max_prefix_len();
        // OK to unwrap here as we can guarantee that `len_range` in non-empty.
        self.with_length_range(lower..=upper).unwrap()
    }

    /// Construct a new IP prefix-range consisting of all the *strictly*
    /// more-specific sub-prefixes of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::PrefixRange as _, Any, Prefix, PrefixRange};
    ///
    /// let x: PrefixRange<Any> = "192.0.2.0/24,25,27".parse()?;
    /// let y: PrefixRange<Any> = "192.0.2.0/24,26,32".parse()?;
    ///
    /// assert_eq!(x.or_longer_excl().unwrap(), y);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn or_longer_excl(self) -> Option<Self> {
        let lower = self.lower().increment().ok()?;
        let upper = self.prefix().max_prefix_len();
        self.with_length_range(lower..=upper)
    }

    /// Construct a new IP prefix-range consisting of all the more-specific
    /// sub-prefixes of `self` of length `len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::PrefixRange as _, Any, Ipv6, Prefix, PrefixLength, PrefixRange};
    ///
    /// let range: PrefixRange<Any> = "2001:db8::/32".parse::<Prefix<Any>>()?.into();
    /// let len = PrefixLength::<Ipv6>::from_primitive(48)?.into();
    ///
    /// assert_eq!(range.with_length(len).unwrap().into_iter().count(), 1 << 16,);
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn with_length(self, len: Self::Length) -> Option<Self> {
        self.with_length_range(len..=len)
    }

    /// Returns the address-family associated with this IP prefix-range.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::PrefixRange as _, Any, PrefixRange};
    ///
    /// let range: PrefixRange<Any> = "2001:db8::/32,48,64".parse()?;
    ///
    /// assert_eq!(range.afi().to_string(), "ipv6");
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn afi(&self) -> concrete::Afi {
        self.prefix().afi()
    }

    /// Try to construct a new [`Self::Length`] for the address-family
    /// associated with this IP prefix-range.
    ///
    /// # Errors
    ///
    /// Fails when `length` is outside of the bounds of prefix-lengths of the
    /// address-family.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip::{traits::PrefixRange as _, Any, Ipv4, PrefixLength, PrefixRange};
    ///
    /// let range: PrefixRange<Any> = "192.0.2.0/24,26,28".parse()?;
    ///
    /// assert_eq!(
    ///     range.new_prefix_length(30)?,
    ///     PrefixLength::<Ipv4>::from_primitive(30)?.into(),
    /// );
    /// # Ok::<(), ip::Error>(())
    /// ```
    fn new_prefix_length(&self, length: u8) -> Result<Self::Length, Error> {
        self.prefix().new_prefix_length(length)
    }
}
