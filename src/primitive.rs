use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::ops::{BitAnd, BitXor, Shl, Shr, Sub};

/// Underlying integer-like type used to respresent an IP address.
pub trait AddressPrimitive:
    Copy
    + Debug
    + Default
    + Hash
    + Ord
    + BitAnd<Self, Output = Self>
    + BitXor<Self, Output = Self>
    + Shl<Self::Width, Output = Self>
    + Shr<Self::Width, Output = Self>
{
    /// Underlying primitive type used to store bit-widths of `Self`.
    type Width: WidthPrimitive;

    /// Maximum valid bit-width of `Self`.
    const MAX_WIDTH: Self::Width;

    /// "All-zeros" IP address representation.
    const ZERO: Self;
    /// "All-ones" IP address representation.
    const ONES: Self;

    /// Get the number of leading zeros in the binary representation of `self`.
    fn leading_zeros(self) -> Self::Width;
}

impl AddressPrimitive for u32 {
    type Width = u8;
    const MAX_WIDTH: Self::Width = 32;

    const ZERO: Self = 0x0000_0000;
    const ONES: Self = 0xffff_ffff;

    fn leading_zeros(self) -> Self::Width {
        self.leading_zeros() as Self::Width
    }
}

impl AddressPrimitive for u128 {
    type Width = u8;
    const MAX_WIDTH: Self::Width = 128;

    const ZERO: Self = 0x0000_0000_0000_0000_0000_0000_0000_0000;
    const ONES: Self = 0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff;

    fn leading_zeros(self) -> Self::Width {
        self.leading_zeros() as Self::Width
    }
}

/// Convenience alias for [`AddressPrimitive::Width`].
pub type WidthOf<Addr> = <Addr as AddressPrimitive>::Width;

/// Underlying integer-like type used to respresent an IP prefix-length.
pub trait WidthPrimitive: Copy + Clone + Debug + Display + Hash + Ord + Sub<Output = Self> {
    /// Additive identity value.
    const ZERO: Self;
}

impl WidthPrimitive for u8 {
    const ZERO: Self = 0;
}
