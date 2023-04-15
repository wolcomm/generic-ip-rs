use core::ops::{Add, BitAnd, BitAndAssign, BitOr, BitXor};

use num_traits::CheckedAdd;

use super::{
    super::{mask_types::Type, Mask},
    Address,
};
use crate::traits::Afi;

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

impl<A: Afi, T: Type> Add<Mask<T, A>> for Address<A> {
    type Output = Option<Self>;

    fn add(self, mask: Mask<T, A>) -> Self::Output {
        self.into_primitive()
            .checked_add(&mask.into_primitive())
            .map(Self::new)
    }
}

impl<A: Afi> BitXor<Self> for Address<A> {
    type Output = A::Primitive;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.into_primitive() ^ rhs.into_primitive()
    }
}
