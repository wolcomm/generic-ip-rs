use core::cmp::Ord;
use core::fmt::Debug;
use core::hash::Hash;

use crate::primitive::{AddressPrimitive, WidthOf, WidthPrimitive};
use crate::{error::Error, parser};

mod macros;

use self::macros::afi_definitions;

/// Provides an interface for describing a class of IP address families.
pub trait AfiClass: Copy + Debug + Hash + Ord {}

/// Provides an interface for describing an IP address family.
pub trait Afi: AfiClass + Copy + Debug + Hash + Ord {
    /// Underlying primitive type used to store IP address integers for this
    /// address family.
    type Addr: AddressPrimitive;

    /// Minimum valid value of the underlying primitive value used to store
    /// prefix-lengths for this address-family.
    const MIN_LENGTH: WidthOf<Self::Addr> = <WidthOf<Self::Addr>>::ZERO;

    /// Maximum valid value of the underlying primitive value used to store
    /// prefix-lengths for this address-family.
    const MAX_LENGTH: WidthOf<Self::Addr> = Self::Addr::MAX_WIDTH;

    /// Get the [`AfiEnum`] variant associated with `Self`.
    fn as_enum() -> AfiEnum;

    /// Parse an `impl AsRef<str>` into a [`Self::Addr`].
    ///
    /// This method is primarily intended for use via the
    /// [`FromStr`][core::str::FromStr] implementation for
    /// [`Address<A>`][crate::addr::Address].
    fn parse_addr<S>(s: &S) -> Result<Self::Addr, Error<'static, Self>>
    where
        S: AsRef<str> + ?Sized;

    /// Parse an `impl AsRef<str>` into a `(Self::Addr, WidthOf<Self::Addr>)`
    /// pair.
    ///
    /// This method is primarily intended for use via the
    /// [`FromStr`][core::str::FromStr] implementation for
    /// [`Prefix<A>`][crate::prefix::Prefix].
    fn parse_prefix<S>(s: &S) -> Result<(Self::Addr, WidthOf<Self::Addr>), Error<'static, Self>>
    where
        S: AsRef<str> + ?Sized;
}

impl<A: Afi> AfiClass for A {}

afi_definitions! {
    /// IPv4 address family marker type.
    pub afi Ipv4 {
        type Addr = u32;
        fn parse_addr = parser::ipv4::parse_addr;
        fn parse_prefix = parser::ipv4::parse_prefix;
    }
    /// IPv6 address family marker type.
    pub afi Ipv6 {
        type Addr = u128;
        fn parse_addr = parser::ipv6::parse_addr;
        fn parse_prefix = parser::ipv6::parse_prefix;
    }
}
