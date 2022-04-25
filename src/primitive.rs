use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::ops::{BitAnd, BitOr, BitXor, Shl, Shr, Sub};

use crate::{af::Afi, error::Error};

/// Underlying integer-like type used to respresent an IP address.
pub trait AddressPrimitive<A: Afi>:
    Copy
    + Debug
    + Default
    + Hash
    + Ord
    + BitAnd<Self, Output = Self>
    + BitOr<Self, Output = Self>
    + BitXor<Self, Output = Self>
    + Shl<Self::Width, Output = Self>
    + Shr<Self::Width, Output = Self>
{
    /// Underlying primitive type used to store bit-widths of `Self`.
    type Width: WidthPrimitive;

    /// Minimum valid value of the underlying primitive value used to store
    /// prefix-lengths for this address-family.
    const MIN_LENGTH: Self::Width = Self::Width::ZERO;

    /// Maximum valid value of the underlying primitive value used to store
    /// prefix-lengths for this address-family.
    const MAX_LENGTH: Self::Width;

    /// "All-zeros" IP address representation.
    const ZERO: Self;
    /// "All-ones" IP address representation.
    const ONES: Self;

    const BROADCAST: Option<Self>;
    const LOCALHOST: Self;
    const UNSPECIFIED: Self;

    const LOCALHOST_NET: (Self, Self::Width);
    const BENCHMARK_NET: (Self, Self::Width);
    const MULTICAST_NET: (Self, Self::Width);
    // TODO:
    // const DOCUMENTATION_NETS: &'static [(Self, Self::Width)];

    /// Get the number of leading zeros in the binary representation of `self`.
    fn leading_zeros(self) -> Self::Width;

    fn to_be_bytes(self) -> A::Octets;

    fn from_be_bytes(bytes: A::Octets) -> Self;

    /// Parse an `impl AsRef<str>` into a [`Self::Addr`].
    ///
    /// This method is primarily intended for use via the
    /// [`FromStr`][core::str::FromStr] implementation for
    /// [`Address<A>`][crate::addr::Address].
    fn parse_addr<S>(s: &S) -> Result<Self, Error<'static, A, Self>>
    where
        S: AsRef<str> + ?Sized;

    /// Parse an `impl AsRef<str>` into a `(Self::Addr, WidthOf<Self::Addr>)`
    /// pair.
    ///
    /// This method is primarily intended for use via the
    /// [`FromStr`][core::str::FromStr] implementation for
    /// [`Prefix<A>`][crate::prefix::Prefix].
    fn parse_prefix<S>(s: &S) -> Result<(Self, Self::Width), Error<'static, A, Self>>
    where
        S: AsRef<str> + ?Sized;
}

/// Underlying integer-like type used to respresent an IP prefix-length.
pub trait WidthPrimitive: Copy + Clone + Debug + Display + Hash + Ord + Sub<Output = Self> {
    /// Additive identity value.
    const ZERO: Self;
    const ONE: Self;
}

impl WidthPrimitive for u8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
