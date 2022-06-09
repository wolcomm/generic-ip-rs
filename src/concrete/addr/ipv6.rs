use crate::{
    any,
    concrete::{Ipv4, Ipv6},
    traits::{primitive::IntoIpv6Segments as _, Address as _, Afi},
};

use super::Address;

// TODO: make methods `const fn`
impl Address<Ipv6> {
    #[must_use]
    pub fn is_unicast_link_local(&self) -> bool {
        self.is_link_local()
    }

    #[allow(clippy::match_same_arms)]
    #[must_use]
    pub fn multicast_scope(&self) -> Option<MulticastScope> {
        if self.is_multicast() {
            match self.octets()[1] & 0x0f {
                0x0 => Some(MulticastScope::Reserved),
                0x1 => Some(MulticastScope::InterfaceLocal),
                0x2 => Some(MulticastScope::LinkLocal),
                0x3 => Some(MulticastScope::RealmLocal),
                0x4 => Some(MulticastScope::AdminLocal),
                0x5 => Some(MulticastScope::SiteLocal),
                0x6..=0x7 => Some(MulticastScope::Unassigned),
                0x8 => Some(MulticastScope::OrganizationLocal),
                0x9..=0xd => Some(MulticastScope::Unassigned),
                0xe => Some(MulticastScope::Global),
                0xf => Some(MulticastScope::Reserved),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    #[must_use]
    pub fn segments(&self) -> [u16; 8] {
        self.into_primitive().into_segments()
    }

    #[allow(clippy::wrong_self_convention)]
    #[allow(clippy::option_if_let_else)]
    #[must_use]
    pub fn to_canonical(&self) -> any::Address {
        if let Some(ipv4_addr) = self.to_ipv4_mapped() {
            any::Address::Ipv4(ipv4_addr)
        } else {
            any::Address::Ipv6(*self)
        }
    }

    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_ipv4(&self) -> Option<Address<Ipv4>> {
        self.to_ipv4_mapped().or_else(|| match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, octets @ ..] => Some(Address::new(
                <Ipv4 as Afi>::Primitive::from_be_bytes(octets),
            )),
            _ => None,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_ipv4_mapped(&self) -> Option<Address<Ipv4>> {
        match self.octets() {
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff, octets @ ..] => Some(Address::new(
                <Ipv4 as Afi>::Primitive::from_be_bytes(octets),
            )),
            _ => None,
        }
    }
}

// TODO: document omission of `non_exhaustive`
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum MulticastScope {
    Reserved,
    Unassigned,
    InterfaceLocal,
    LinkLocal,
    RealmLocal,
    AdminLocal,
    SiteLocal,
    OrganizationLocal,
    Global,
}
