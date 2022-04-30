use crate::{
    af::{Ipv4, Ipv6},
    concrete::{
        self,
        mask_types::{Host, Net, Type},
    },
    traits,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Mask<T: Type> {
    Ipv4(concrete::Mask<T, Ipv4>),
    Ipv6(concrete::Mask<T, Ipv6>),
}

pub type Netmask = Mask<Net>;

pub type Hostmask = Mask<Host>;

impl<T: Type> traits::Mask for Mask<T> {}

impl<T: Type> From<concrete::Mask<T, Ipv4>> for Mask<T> {
    fn from(mask: concrete::Mask<T, Ipv4>) -> Self {
        Self::Ipv4(mask)
    }
}

impl<T: Type> From<concrete::Mask<T, Ipv6>> for Mask<T> {
    fn from(mask: concrete::Mask<T, Ipv6>) -> Self {
        Self::Ipv6(mask)
    }
}

// TODO: impl Display
// TODO: impl Arbitrary
