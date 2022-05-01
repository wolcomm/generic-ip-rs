use crate::{
    concrete::{self, Ipv4, Ipv6},
    traits,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Address {
    Ipv4(concrete::Address<Ipv4>),
    Ipv6(concrete::Address<Ipv6>),
}

impl Address {
    pub fn is_ipv4(&self) -> bool {
        matches!(self, Self::Ipv4(_))
    }

    pub fn is_ipv6(&self) -> bool {
        matches!(self, Self::Ipv6(_))
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_canonical(&self) -> Self {
        match self {
            Self::Ipv4(_) => *self,
            Self::Ipv6(ipv6_addr) => ipv6_addr.to_canonical(),
        }
    }
}

// TODO: deduplicate
macro_rules! delegate {
    ( $( fn $fn:ident(&self) -> $ret_ty:ty; )* ) => {
        $(
            fn $fn(&self) -> $ret_ty {
                match self {
                    Self::Ipv4(addr) => addr.$fn(),
                    Self::Ipv6(addr) => addr.$fn(),
                }
            }
        )*
    }
}

impl traits::Address for Address {
    delegate! {
        fn is_broadcast(&self) -> bool;
        fn is_link_local(&self) -> bool;
        fn is_private(&self) -> bool;
        fn is_reserved(&self) -> bool;
        fn is_shared(&self) -> bool;
        fn is_thisnet(&self) -> bool;
        fn is_benchmarking(&self) -> bool;
        fn is_documentation(&self) -> bool;
        fn is_global(&self) -> bool;
        fn is_loopback(&self) -> bool;
        fn is_multicast(&self) -> bool;
        fn is_unicast(&self) -> bool;
        fn is_unspecified(&self) -> bool;
        fn is_unique_local(&self) -> bool;
    }
}

impl From<concrete::Address<Ipv4>> for Address {
    fn from(addr: concrete::Address<Ipv4>) -> Self {
        Self::Ipv4(addr)
    }
}

impl From<concrete::Address<Ipv6>> for Address {
    fn from(addr: concrete::Address<Ipv6>) -> Self {
        Self::Ipv6(addr)
    }
}
