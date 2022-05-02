use core::borrow::Borrow;

use crate::{
    concrete::{Ipv4, Ipv6},
    traits::Afi,
};

use crate::traits::primitive::{Address as _, IntoIpv6Segments as _};

use super::Address;

// TODO:
// These should be `impl<A: Afi> From<A::Primitive> for Address<A>`, but that is not
// allowed by the coherence rules.
macro_rules! impl_from_primitive {
    ( $( $af:ident ),* $(,)? ) => {
        $(
            impl From<<$af as Afi>::Primitive> for Address<$af> {
                fn from(primitive: <$af as Afi>::Primitive) -> Self {
                    Self::new(primitive)
                }
            }

            impl From<Address<$af>> for <$af as Afi>::Primitive {
                fn from(addr: Address<$af>) -> <$af as Afi>::Primitive {
                    addr.into_primitive()
                }
            }
        )*
    };
}
impl_from_primitive! { Ipv4, Ipv6, }

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
mod stdlib {
    use super::*;

    use std::net::{Ipv4Addr, Ipv6Addr};

    use crate::concrete::{Ipv4, Ipv6};

    impl From<Ipv4Addr> for Address<Ipv4> {
        fn from(addr: Ipv4Addr) -> Self {
            Self::new(addr.into())
        }
    }

    impl From<Ipv6Addr> for Address<Ipv6> {
        fn from(addr: Ipv6Addr) -> Self {
            Self::new(addr.into())
        }
    }
}
