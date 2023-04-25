use core::fmt::Debug;
use core::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Sub};

use num_traits::{One, Zero};

use super::{Prefix, Range};

/// Address-family independent interface for IP prefix-sets
///
/// Methods on `PrefixSet` types that are well defined for all address-families
/// are implemented via this trait.
///
/// See also [`concrete::PrefixSet<A>`][crate::concrete::PrefixSet] and
/// [`any::PrefixSet`][crate::any::PrefixSet] for address-family specific items.
#[allow(clippy::trait_duplication_in_bounds)]
pub trait Set<'a>:
    Debug
    + Clone
    + Default
    + Extend<Self::Prefix>
    + FromIterator<Self::Prefix>
    + Extend<Self::Range>
    + FromIterator<Self::Range>
    + One
    + Zero
    + PartialEq
    + Eq
    + PartialOrd
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Not<Output = Self>
    + Add<Output = Self>
    + Mul<Output = Self>
    + Sub<Output = Self>
{
    /// The type of IP prefix over which `Self` represents a set.
    type Prefix: Prefix;

    /// The type of IP prefix-range over which `Self` represents a set.
    type Range: Range<Prefix = Self::Prefix>;

    /// The iterator returned by [`Self::prefixes`].
    type Prefixes: Iterator<Item = Self::Prefix>;

    /// The iterator returned by [`Self::ranges`].
    type Ranges: Iterator<Item = Self::Range>;

    /// Test whether `prefix` is contained in `self`.
    ///
    /// ```
    /// # use ip::{traits::PrefixSet as _, Error, Ipv4, Prefix, PrefixRange, PrefixSet};
    /// let set: PrefixSet<Ipv4> = ["192.0.2.0/24,26,26".parse::<PrefixRange<Ipv4>>()?]
    ///     .into_iter()
    ///     .collect();
    /// assert!(set.contains("192.0.2.128/26".parse()?));
    /// # Ok::<_, Error>(())
    /// ```
    fn contains(&self, prefix: Self::Prefix) -> bool;

    /// Get an iterator over the [`Self::Prefix`]s contained in `self`.
    ///
    /// ```
    /// # use core::str::FromStr;
    /// # use ip::{traits::PrefixSet as _, Any, Error, Prefix, PrefixSet};
    /// let set: PrefixSet<Any> = ["192.0.2.0/25", "192.0.2.128/25"]
    ///     .into_iter()
    ///     .map(Prefix::<Any>::from_str)
    ///     .collect::<Result<_, _>>()?;
    /// let mut prefixes = set.prefixes();
    /// assert_eq!(prefixes.next(), Some("192.0.2.0/25".parse()?));
    /// assert_eq!(prefixes.next(), Some("192.0.2.128/25".parse()?));
    /// assert_eq!(prefixes.next(), None);
    /// # Ok::<_, Error>(())
    /// ```
    fn prefixes(&'a self) -> Self::Prefixes;

    /// Get an iterator over the [`Self::Range`]s contained in `self`.
    ///
    /// ```
    /// # use core::str::FromStr;
    /// # use ip::{traits::PrefixSet as _, Any, Error, Prefix, PrefixSet};
    /// let set: PrefixSet<Any> = ["192.0.2.0/25", "192.0.2.128/25"]
    ///     .into_iter()
    ///     .map(Prefix::<Any>::from_str)
    ///     .collect::<Result<_, _>>()?;
    /// let mut ranges = set.ranges();
    /// assert_eq!(ranges.next(), Some("192.0.2.0/24,25,25".parse()?));
    /// assert_eq!(ranges.next(), None);
    /// # Ok::<_, Error>(())
    /// ```
    fn ranges(&'a self) -> Self::Ranges;

    /// Construct a prefix-set consisting of all prefixes.
    ///
    /// ```
    /// # use core::str::FromStr;
    /// # use ip::{traits::PrefixSet as _, Any, Error, Prefix, PrefixSet};
    /// let set = PrefixSet::<Any>::any();
    /// let mut ranges = set.ranges();
    /// assert_eq!(ranges.next(), Some("0.0.0.0/0,0,32".parse()?));
    /// assert_eq!(ranges.next(), Some("::/0,0,128".parse()?));
    /// assert_eq!(ranges.next(), None);
    /// # Ok::<_, Error>(())
    /// ```
    #[must_use]
    fn any() -> Self {
        Self::one()
    }

    /// Get the number of prefixes in `self`.
    ///
    /// ```
    /// # use ip::{PrefixSet, traits::PrefixSet as _, Error, Ipv4, PrefixRange};
    /// let set: PrefixSet<Ipv4> = ["192.0.2.0/24,26,26".parse::<PrefixRange<Ipv4>>()?]
    ///     .into_iter()
    ///     .collect();
    /// assert_eq!(set.len(), 4);
    /// # Ok::<_, Error>(())
    /// ```
    #[must_use]
    fn len(&'a self) -> usize {
        self.prefixes().count()
    }

    /// Test whether `self` is empty.
    ///
    /// ``` rust
    /// # use ip::{traits::PrefixSet as _, Error, Ipv6, PrefixSet};
    /// assert!(PrefixSet::<Ipv6>::default().is_empty());
    /// # Ok::<_, Error>(())
    /// ```
    #[must_use]
    fn is_empty(&'a self) -> bool {
        self.ranges().count() == 0
    }
}
