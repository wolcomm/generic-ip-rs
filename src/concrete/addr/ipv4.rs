use crate::{
    concrete::{Ipv4, Ipv6},
    traits::Afi,
};

use super::Address;

// TODO: make methods `const fn`
impl Address<Ipv4> {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_ipv6_compatible(&self) -> Address<Ipv6> {
        Address::new(<Ipv6 as Afi>::Primitive::from_be_bytes(
            self.to_ipv6_lo_octets(),
        ))
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_ipv6_mapped(&self) -> Address<Ipv6> {
        let mut octets = self.to_ipv6_lo_octets();
        octets[10..12].copy_from_slice(&[0xffu8, 0xffu8]);
        Address::new(<Ipv6 as Afi>::Primitive::from_be_bytes(octets))
    }

    fn to_ipv6_lo_octets(self) -> <Ipv6 as Afi>::Octets {
        let mut octets = <Ipv6 as Afi>::Octets::default();
        octets[12..].copy_from_slice(&self.octets());
        octets
    }
}
