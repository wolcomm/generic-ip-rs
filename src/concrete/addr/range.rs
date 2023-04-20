use core::ops::RangeInclusive;

use super::Address;
use crate::traits::Afi;

/// An inclusive range of [`Address<A>`].
///
/// # Example
///
/// ``` rust
/// use ip::{concrete::AddressRange, Ipv4};
///
/// let range = AddressRange::<Ipv4>::new("10.250.0.0".parse()?, "10.252.255.255".parse()?);
///
/// let mid = "10.251.127.1".parse()?;
///
/// assert!(range.contains(&mid));
/// # Ok::<(), ip::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Range<A: Afi>(RangeInclusive<Address<A>>);

impl<A: Afi> Range<A> {
    /// Construct a new [`AddressRange<A>`][Self] from `start` and `end` bounds.
    pub const fn new(start: Address<A>, end: Address<A>) -> Self {
        Self(start..=end)
    }

    /// Returns [`true`] if `addr` is contained in the range.
    pub fn contains(&self, addr: &Address<A>) -> bool {
        self.0.contains(addr)
    }
}

impl<A: Afi> From<&RangeInclusive<A::Primitive>> for Range<A> {
    fn from(range: &RangeInclusive<A::Primitive>) -> Self {
        Self::new(Address::new(*range.start()), Address::new(*range.end()))
    }
}
