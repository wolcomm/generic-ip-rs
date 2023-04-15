use super::Prefix;
use crate::concrete::{self, Ipv4, Ipv6};

/// Iterator returned by [`Prefix::subprefixes`].
#[derive(Debug, Copy, Clone)]
pub enum Subprefixes {
    /// IPv4 variant.
    Ipv4(concrete::Subprefixes<Ipv4>),
    /// IPv6 variant.
    Ipv6(concrete::Subprefixes<Ipv6>),
}

impl Iterator for Subprefixes {
    type Item = Prefix;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Ipv4(iter) => iter.next().map(Self::Item::Ipv4),
            Self::Ipv6(iter) => iter.next().map(Self::Item::Ipv6),
        }
    }
}
