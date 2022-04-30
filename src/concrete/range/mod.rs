use core::ops::RangeInclusive;

use crate::af::Afi;

use super::Address;

#[derive(Clone, Debug)]
pub struct AddressRange<A: Afi>(RangeInclusive<Address<A>>);

impl<A: Afi> AddressRange<A> {
    pub fn new(start: Address<A>, end: Address<A>) -> Self {
        Self(RangeInclusive::new(start, end))
    }

    pub fn contains(&self, addr: &Address<A>) -> bool {
        self.0.contains(addr)
    }
}

impl<A: Afi> From<&RangeInclusive<A::Primitive>> for AddressRange<A> {
    fn from(range: &RangeInclusive<A::Primitive>) -> Self {
        Self::new(Address::new(*range.start()), Address::new(*range.end()))
    }
}
