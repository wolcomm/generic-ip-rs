use crate::{
    concrete::Ipv4,
    traits::{primitive::Address as _, Afi},
};

/// An IP address.
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address<A: Afi>(A::Primitive);

impl<A: Afi> Address<A> {
    // TODO: use `Self::new()` to construct these (and move out of `mod
    // private`) once const trait bounds are available in stable rustc
    // (1.61+)
    pub const LOCALHOST: Self = Self(A::Primitive::LOCALHOST);
    pub const UNSPECIFIED: Self = Self(A::Primitive::UNSPECIFIED);

    /// Construct a new [`Address<A>`] from an integer primitive
    /// appropriate to `A`.
    pub fn new(inner: A::Primitive) -> Self {
        Self(inner)
    }

    /// Get the inner integer val, consuming `self`.
    pub fn into_primitive(self) -> A::Primitive {
        self.0
    }
}

impl Address<Ipv4> {
    // TODO: use `Self::new()` to contruct these once const trait bounds are
    // available in stable rustc (1.61+)
    pub const BROADCAST: Self = {
        if let Some(inner) = <Ipv4 as Afi>::Primitive::BROADCAST {
            Self(inner)
        } else {
            panic!("failed to get BROADCAST address value")
        }
    };
}
