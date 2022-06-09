use core::str::FromStr;

use crate::{
    concrete::{self, Ipv4, Ipv6},
    error::Error,
    traits,
};

use super::{delegate, Address, Hostmask, Netmask};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub enum Prefix {
    Ipv4(concrete::Prefix<Ipv4>),
    Ipv6(concrete::Prefix<Ipv6>),
}

impl traits::Prefix for Prefix {
    type Address = Address;
    type PrefixLength = PrefixLength;
    type Hostmask = Hostmask;
    type Netmask = Netmask;

    delegate! {
        fn network(&self) -> Self::Address;
        fn addr(&self) -> Self::Address;
        fn trunc(&self) -> Self;
        fn hostmask(&self) -> Self::Hostmask;
        fn netmask(&self) -> Self::Netmask;
        fn max_prefix_len(&self) -> Self::PrefixLength;
        fn prefix_len(&self) -> Self::PrefixLength;
        fn broadcast(&self) -> Self::Address;
    }

    fn supernet(&self) -> Option<Self> {
        match self {
            Self::Ipv4(prefix) => prefix.supernet().map(Self::Ipv4),
            Self::Ipv6(prefix) => prefix.supernet().map(Self::Ipv6),
        }
    }

    fn is_sibling(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ipv4(prefix), Self::Ipv4(other)) => prefix.is_sibling(other),
            (Self::Ipv6(prefix), Self::Ipv6(other)) => prefix.is_sibling(other),
            _ => false,
        }
    }
}

impl From<concrete::Prefix<Ipv4>> for Prefix {
    fn from(prefix: concrete::Prefix<Ipv4>) -> Self {
        Self::Ipv4(prefix)
    }
}

impl From<concrete::Prefix<Ipv6>> for Prefix {
    fn from(prefix: concrete::Prefix<Ipv6>) -> Self {
        Self::Ipv6(prefix)
    }
}

impl FromStr for Prefix {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        concrete::Prefix::<Ipv4>::from_str(s)
            .map(Self::from)
            .or_else(|_| concrete::Prefix::<Ipv6>::from_str(s).map(Self::from))
    }
}

// TODO: impl Display for Prefix
// TODO: impl Arbitrary for Prefix

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd)]
pub enum PrefixLength {
    Ipv4(concrete::PrefixLength<Ipv4>),
    Ipv6(concrete::PrefixLength<Ipv6>),
}

impl traits::PrefixLength for PrefixLength {}

impl From<concrete::PrefixLength<Ipv4>> for PrefixLength {
    fn from(length: concrete::PrefixLength<Ipv4>) -> Self {
        Self::Ipv4(length)
    }
}

impl From<concrete::PrefixLength<Ipv6>> for PrefixLength {
    fn from(length: concrete::PrefixLength<Ipv6>) -> Self {
        Self::Ipv6(length)
    }
}

// TODO: impl Display for PrefixLength
// TODO: impl FromStr for PrefixLength
// TODO: impl Arbitrary for PrefixLength
