use core::marker::PhantomData;

use crate::{af::Afi, traits::primitive::Address as _};

use super::Type;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Mask<T: Type, A: Afi>(A::Primitive, PhantomData<T>);

impl<A: Afi, T: Type> Mask<T, A> {
    pub const ZEROS: Self = Self(A::Primitive::ZERO, PhantomData);
    pub const ONES: Self = Self(A::Primitive::ONES, PhantomData);

    pub fn new(bits: A::Primitive) -> Self {
        Self(bits, PhantomData)
    }

    pub fn into_primitive(self) -> A::Primitive {
        self.0
    }
}
