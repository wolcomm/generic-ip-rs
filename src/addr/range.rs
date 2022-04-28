use core::ops::RangeInclusive;

use crate::af::Afi;

use super::ConcreteAddress;

#[derive(Clone, Debug)]
pub struct AddressRange<A: Afi>(RangeInclusive<ConcreteAddress<A>>);

impl<A: Afi> AddressRange<A> {
    pub fn new(start: ConcreteAddress<A>, end: ConcreteAddress<A>) -> Self {
        Self(RangeInclusive::new(start, end))
    }

    pub fn contains(&self, addr: &ConcreteAddress<A>) -> bool {
        self.0.contains(addr)
    }
}

impl<A: Afi> From<&RangeInclusive<A::AddressPrimitive>> for AddressRange<A> {
    fn from(range: &RangeInclusive<A::AddressPrimitive>) -> Self {
        Self::new(
            ConcreteAddress::new(*range.start()),
            ConcreteAddress::new(*range.end()),
        )
    }
}
