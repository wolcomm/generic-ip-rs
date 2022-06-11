use crate::traits::Afi;

/// An IP address of address family `A`.
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address<A: Afi>(A::Primitive);

impl<A: Afi> Address<A> {
    /// Construct a new [`Address<A>`] from an integer primitive
    /// appropriate to `A`.
    pub const fn new(inner: A::Primitive) -> Self {
        Self(inner)
    }

    /// Get the primitive integer value, consuming `self`.
    pub const fn into_primitive(self) -> A::Primitive {
        self.0
    }
}
