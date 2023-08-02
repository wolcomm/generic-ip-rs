use core::borrow::Borrow;

use super::Address;
use crate::traits::primitive::{Address as _, IntoIpv6Segments as _};
use crate::{
    concrete::{Ipv4, Ipv6},
    traits::Afi,
};

// TODO:
// These should be `impl<A: Afi> From<A::Primitive> for Address<A>`, but that is
// not allowed by the coherence rules.
macro_rules! impl_from_primitive {
    ( $( $af:ident ),* $(,)? ) => {
        $(
            impl From<<$af as Afi>::Primitive> for Address<$af> {
                fn from(primitive: <$af as Afi>::Primitive) -> Self {
                    Self::new(primitive)
                }
            }
        )*
    };
}
impl_from_primitive! { Ipv4, Ipv6, }

// TODO:
// Ditto
macro_rules! impl_try_from_byte_slice {
    ( $( $af:ident ),* $(,)? ) => {
        $(
            impl<'a> TryFrom<&'a [u8]> for Address<$af> {
                type Error = <<$af as Afi>::Octets as TryFrom<&'a [u8]>>::Error;
                fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
                    value.try_into().map(Self::from_octets)
                }
            }
        )*
    };
}
impl_try_from_byte_slice! { Ipv4, Ipv6, }

impl<A, O> From<O> for Address<A>
where
    A: Afi<Octets = O>,
    O: Borrow<[u8]>,
{
    fn from(octets: O) -> Self {
        Self::new(A::Primitive::from_be_bytes(octets))
    }
}

impl From<[u16; 8]> for Address<Ipv6> {
    fn from(segments: [u16; 8]) -> Self {
        Self::new(<Ipv6 as Afi>::Primitive::from_segments(segments))
    }
}

#[cfg(feature = "std")]
impl From<std::net::Ipv4Addr> for Address<Ipv4> {
    fn from(addr: std::net::Ipv4Addr) -> Self {
        Self::new(addr.into())
    }
}

#[cfg(feature = "std")]
impl From<std::net::Ipv6Addr> for Address<Ipv6> {
    fn from(addr: std::net::Ipv6Addr) -> Self {
        Self::new(addr.into())
    }
}
