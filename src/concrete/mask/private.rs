use core::marker::PhantomData;

use crate::traits::Afi;

use super::Type;

/// An IP mask of address family `A`.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Mask<T: Type, A: Afi>(A::Primitive, PhantomData<T>);

impl<A: Afi, T: Type> Mask<T, A> {
    /// Construct a new [`Mask<T, A>`] from an integer primitive appropriate to `A`.
    pub const fn new(bits: A::Primitive) -> Self {
        Self(bits, PhantomData)
    }

    /// Get the primitive integer value, consuming `self`.
    pub const fn into_primitive(self) -> A::Primitive {
        self.0
    }
}
