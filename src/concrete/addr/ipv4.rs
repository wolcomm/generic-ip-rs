use crate::{
    concrete::{Ipv4, Ipv6},
    traits::{primitive::Address as _, Afi},
};

use super::Address;

// TODO: make methods `const fn`
impl Address<Ipv4> {
    pub const BROADCAST: Self = {
        if let Some(inner) = <Ipv4 as Afi>::Primitive::BROADCAST {
            Self::new(inner)
        } else {
            panic!("failed to get BROADCAST address value")
        }
    };

    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_ipv6_compatible(&self) -> Address<Ipv6> {
        Address::from_octets(self.to_ipv6_lo_octets())
    }

    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_ipv6_mapped(&self) -> Address<Ipv6> {
        let mut octets = self.to_ipv6_lo_octets();
        octets[10..12].copy_from_slice(&[0xffu8, 0xffu8]);
        Address::from_octets(octets)
    }

    fn to_ipv6_lo_octets(self) -> <Ipv6 as Afi>::Octets {
        let mut octets = <Ipv6 as Afi>::Octets::default();
        octets[12..].copy_from_slice(&self.octets());
        octets
    }
}
