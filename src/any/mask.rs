use super::PrefixLength;
use crate::{
    concrete::{
        self,
        mask_types::{Host, Net, Type},
        Ipv4, Ipv6,
    },
    traits,
};

/// Either an IPv4 or IPv6 address mask.
///
/// # Memory Use
///
/// Rust enums are sized to accomodate their largest variant, with smaller
/// variants being padded to fill up any unused space.
///
/// As a result, users should avoid using this type in a context where only
/// [`Mask::Ipv4`] variants are expected.
#[allow(variant_size_differences)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Mask<T: Type> {
    /// IPv4 address mask variant.
    Ipv4(concrete::Mask<T, Ipv4>),
    /// IPv6 address mask variant.
    Ipv6(concrete::Mask<T, Ipv6>),
}

/// Either an IPv4 or IPv6 netmask.
pub type Netmask = Mask<Net>;

/// Either an IPv4 or IPv6 netmask.
pub type Hostmask = Mask<Host>;

impl<T: Type> traits::Mask for Mask<T> {}
impl traits::Netmask for Netmask {}
impl traits::Hostmask for Hostmask {}

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

impl From<PrefixLength> for Netmask {
    fn from(len: PrefixLength) -> Self {
        match len {
            PrefixLength::Ipv4(len) => Self::Ipv4(len.into()),
            PrefixLength::Ipv6(len) => Self::Ipv6(len.into()),
        }
    }
}

impl From<PrefixLength> for Hostmask {
    fn from(len: PrefixLength) -> Self {
        match len {
            PrefixLength::Ipv4(len) => Self::Ipv4(len.into()),
            PrefixLength::Ipv6(len) => Self::Ipv6(len.into()),
        }
    }
}

// TODO: impl FromStr
// TODO: impl Display
// TODO: impl Arbitrary
