use core::ops::{BitAnd, BitAndAssign, BitOr, BitXor};

use crate::traits::Afi;

use super::{
    super::{mask_types::Type, Mask},
    Address,
};

impl<A: Afi, T: Type> BitAnd<Mask<T, A>> for Address<A> {
    type Output = Self;

    fn bitand(self, mask: Mask<T, A>) -> Self::Output {
        Self::new(self.into_primitive().bitand(mask.into_primitive()))
    }
}

impl<A: Afi, T> BitAndAssign<T> for Address<A>
where
    Self: BitAnd<T, Output = Self>,
{
    fn bitand_assign(&mut self, rhs: T) {
        *self = self.bitand(rhs);
    }
}

impl<A: Afi, T: Type> BitOr<Mask<T, A>> for Address<A> {
    type Output = Self;

    fn bitor(self, mask: Mask<T, A>) -> Self::Output {
        Self::new(self.into_primitive().bitor(mask.into_primitive()))
    }
}

impl<A: Afi> BitXor<Self> for Address<A> {
    type Output = A::Primitive;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.into_primitive() ^ rhs.into_primitive()
    }
}
