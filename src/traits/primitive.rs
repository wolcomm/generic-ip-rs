use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::mem;
use core::ops::{Add, BitAnd, BitOr, BitXor, Not, RangeInclusive, Shl, Shr, Sub};

use super::Afi;
use crate::{
    concrete::{Ipv4, Ipv6},
    error::Error,
    parser,
};

/// Underlying integer-like type used to respresent an IP address.
pub trait Address<A: Afi>:
    Copy
    + Debug
    + Default
    + Hash
    + Ord
    + BitAnd<Self, Output = Self>
    + BitOr<Self, Output = Self>
    + BitXor<Self, Output = Self>
    + Not<Output = Self>
    + Shl<Self::Length, Output = Self>
    + Shr<Self::Length, Output = Self>
    + 'static
{
    /// Underlying primitive type used to store bit-widths of `Self`.
    type Length: Length;

    /// Minimum valid value of the underlying primitive value used to store
    /// prefix-lengths for this address-family.
    const MIN_LENGTH: Self::Length = Self::Length::ZERO;

    /// Maximum valid value of the underlying primitive value used to store
    /// prefix-lengths for this address-family.
    const MAX_LENGTH: Self::Length;

    /// "All-zeros" IP address representation.
    const ZERO: Self;
    /// "All-ones" IP address representation.
    const ONES: Self;

    /// The primitive value of the subnet-local broadcast address, if it is
    /// defined for the address family.
    const BROADCAST: Option<Self>;

    /// The primitive value of the `localhost` address for this address family.
    const LOCALHOST: Self;

    /// The primitive value of the "unspecified" address for this address
    /// family.
    const UNSPECIFIED: Self;

    /// The range of primitive address values defined for "loopback" use for
    /// this address family.
    const LOOPBACK_RANGE: RangeInclusive<Self>;

    /// The range of primitive address values defined for "benchmarking" use for
    /// this address family.
    const BENCHMARK_RANGE: RangeInclusive<Self>;

    /// The range of primitive address values defined for "multicast" use for
    /// this address family.
    const MULTICAST_RANGE: RangeInclusive<Self>;

    /// The range of primitive address values defined for "link-local" use for
    /// this address family.
    const LINK_LOCAL_RANGE: RangeInclusive<Self>;

    /// The range of primitive address values reserved for IETF protocol
    /// assignments for this address family.
    const PROTOCOL_ASSIGNMENTS_RANGE: RangeInclusive<Self>;

    /// The range of primitive address values defined for "documentation" use
    /// for this address family.
    const DOCUMENTATION_RANGES: &'static [RangeInclusive<Self>];

    /// The range of primitive address values defined for "private" use, if
    /// that is defined for this address family.
    const PRIVATE_RANGES: Option<&'static [RangeInclusive<Self>]>;

    /// The range of primitive address values reserved for future use, if that
    /// is defined for this address family.
    const RESERVED_RANGE: Option<RangeInclusive<Self>>;

    /// The range of primitive address values reserved for "shared" use, if that
    /// is defined for this address family.
    const SHARED_RANGE: Option<RangeInclusive<Self>>;

    /// The range of primitive address values having "this network" semantics,
    /// if that is defined for this address family.
    const THISNET_RANGE: Option<RangeInclusive<Self>>;

    /// The range of primitive address values defined as "unique local
    /// addresses", if that is defined for this address family.
    const ULA_RANGE: Option<RangeInclusive<Self>>;

    /// Get the number of leading zeros in the binary representation of `self`.
    fn leading_zeros(self) -> Self::Length;

    /// Convert `self` to big-endian [`A::Octets`][Afi::Octets].
    fn to_be_bytes(self) -> A::Octets;

    /// Construct `Self` from big-endian [`A::Octets`][Afi::Octets].
    fn from_be_bytes(bytes: A::Octets) -> Self;

    // TODO: This really is a horrible hack. Will do better.
    /// Returns [`true`] if this primitive value respresents a "globally
    /// routable" address, according to the address family semantics.
    fn is_global(&self) -> bool;

    /// Parse an `impl AsRef<str>` into a [`Self::Addr`].
    ///
    /// This method is primarily intended for use via the
    /// [`FromStr`][core::str::FromStr] implementation for
    /// [`Address<A>`][crate::addr::Address].
    ///
    /// # Errors
    ///
    /// Fails if the string does not conform to the textual address
    /// representation rules for `A`.
    fn parse_addr<S>(s: &S) -> Result<Self, Error>
    where
        S: AsRef<str> + ?Sized;

    /// Parse an `impl AsRef<str>` into a `(Self::Addr, WidthOf<Self::Addr>)`
    /// pair.
    ///
    /// This method is primarily intended for use via the
    /// [`FromStr`][core::str::FromStr] implementation for
    /// [`Prefix<A>`][crate::prefix::Prefix].
    ///
    /// # Errors
    ///
    /// Fails if the string does not conform to the textual address
    /// representation rules for `A`.
    fn parse_prefix<S>(s: &S) -> Result<(Self, Self::Length), Error>
    where
        S: AsRef<str> + ?Sized;
}

macro_rules! ipv4 {
    ($a:literal, $b:literal, $c:literal, $d:literal) => {
        u32::from_be_bytes([$a, $b, $c, $d])
    };
}

impl Address<Ipv4> for u32 {
    type Length = u8;

    const MAX_LENGTH: Self::Length = 32;
    const ZERO: Self = ipv4!(0, 0, 0, 0);
    const ONES: Self = ipv4!(255, 255, 255, 255);

    const BROADCAST: Option<Self> = Some(ipv4!(255, 255, 255, 255));
    // const LOCALHOST: Self = 0x7f00_0001;
    const LOCALHOST: Self = ipv4!(127, 0, 0, 1);
    const UNSPECIFIED: Self = ipv4!(0, 0, 0, 0);

    const LOOPBACK_RANGE: RangeInclusive<Self> = ipv4!(127, 0, 0, 0)..=ipv4!(127, 255, 255, 255);
    const BENCHMARK_RANGE: RangeInclusive<Self> = ipv4!(198, 18, 0, 0)..=ipv4!(198, 19, 255, 255);
    const MULTICAST_RANGE: RangeInclusive<Self> = ipv4!(224, 0, 0, 0)..=ipv4!(239, 255, 255, 255);
    const LINK_LOCAL_RANGE: RangeInclusive<Self> =
        ipv4!(169, 254, 0, 0)..=ipv4!(169, 254, 255, 255);
    const PROTOCOL_ASSIGNMENTS_RANGE: RangeInclusive<Self> =
        ipv4!(192, 0, 0, 0)..=ipv4!(192, 0, 0, 255);
    const DOCUMENTATION_RANGES: &'static [RangeInclusive<Self>] = &[
        ipv4!(192, 0, 2, 0)..=ipv4!(192, 0, 2, 255),
        ipv4!(198, 51, 100, 0)..=ipv4!(198, 51, 100, 255),
        ipv4!(203, 0, 113, 0)..=ipv4!(203, 0, 113, 255),
    ];
    const PRIVATE_RANGES: Option<&'static [RangeInclusive<Self>]> = Some(&[
        ipv4!(10, 0, 0, 0)..=ipv4!(10, 255, 255, 255),
        ipv4!(172, 16, 0, 0)..=ipv4!(172, 31, 255, 255),
        ipv4!(192, 168, 0, 0)..=ipv4!(192, 168, 255, 255),
    ]);
    const RESERVED_RANGE: Option<RangeInclusive<Self>> =
        Some(ipv4!(240, 0, 0, 0)..=ipv4!(255, 255, 255, 255));
    const SHARED_RANGE: Option<RangeInclusive<Self>> =
        Some(ipv4!(100, 64, 0, 0)..=ipv4!(100, 127, 255, 255));
    const THISNET_RANGE: Option<RangeInclusive<Self>> =
        Some(ipv4!(0, 0, 0, 0)..=ipv4!(0, 255, 255, 255));
    const ULA_RANGE: Option<RangeInclusive<Self>> = None;

    #[allow(clippy::cast_possible_truncation)]
    fn leading_zeros(self) -> Self::Length {
        self.leading_zeros() as Self::Length
    }

    fn to_be_bytes(self) -> <Ipv4 as Afi>::Octets {
        self.to_be_bytes()
    }

    // TODO: remove `allow` once https://github.com/rust-lang/rust-clippy/pull/8804
    // is merged
    #[allow(clippy::only_used_in_recursion)]
    fn from_be_bytes(bytes: <Ipv4 as Afi>::Octets) -> Self {
        Self::from_be_bytes(bytes)
    }

    fn is_global(&self) -> bool {
        if Self::LOOPBACK_RANGE.contains(self)
            || Self::LINK_LOCAL_RANGE.contains(self)
            || Self::BENCHMARK_RANGE.contains(self)
            || Self::DOCUMENTATION_RANGES
                .iter()
                .any(|range| range.contains(self))
        {
            return false;
        }
        if let Some(ref broadcast) = Self::BROADCAST {
            if broadcast == self {
                return false;
            }
        }
        if let Some(ranges) = Self::PRIVATE_RANGES {
            if ranges.iter().any(|range| range.contains(self)) {
                return false;
            }
        }
        if let Some(range) = Self::SHARED_RANGE {
            if range.contains(self) {
                return false;
            }
        }
        if let Some(range) = Self::RESERVED_RANGE {
            if range.contains(self) {
                return false;
            }
        }
        if let Some(range) = Self::THISNET_RANGE {
            if range.contains(self) {
                return false;
            }
        }
        // non-global multicast
        if Self::MULTICAST_RANGE.contains(self)
            && !(ipv4!(224, 0, 1, 0)..=ipv4!(238, 255, 255, 255)).contains(self)
        {
            return false;
        }
        if Self::PROTOCOL_ASSIGNMENTS_RANGE.contains(self)
            // Port Control Protocol Anycast
            && self != &ipv4!(192, 0, 0, 9)
            // TURN Anycast
            && self != &ipv4!(192, 0, 0, 10)
        {
            return false;
        }
        true
    }

    fn parse_addr<S>(s: &S) -> Result<Self, Error>
    where
        S: AsRef<str> + ?Sized,
    {
        parser::ipv4::parse_addr(s.as_ref())
    }

    fn parse_prefix<S>(s: &S) -> Result<(Self, Self::Length), Error>
    where
        S: AsRef<str> + ?Sized,
    {
        parser::ipv4::parse_prefix(s.as_ref())
    }
}

impl Address<Ipv6> for u128 {
    type Length = u8;

    const MAX_LENGTH: Self::Length = 128;
    const ZERO: Self = 0x0000_0000_0000_0000_0000_0000_0000_0000;
    const ONES: Self = 0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff;

    const BROADCAST: Option<Self> = None;
    const LOCALHOST: Self = 0x0000_0000_0000_0000_0000_0000_0000_0001;
    const UNSPECIFIED: Self = Self::ZERO;

    const LOOPBACK_RANGE: RangeInclusive<Self> = 0x1..=0x1;
    const BENCHMARK_RANGE: RangeInclusive<Self> =
        0x2001_0002_0000_0000_0000_0000_0000_0000..=0x2001_0002_0000_ffff_ffff_ffff_ffff_ffff;
    const MULTICAST_RANGE: RangeInclusive<Self> =
        0xff00_0000_0000_0000_0000_0000_0000_0000..=0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff;
    const LINK_LOCAL_RANGE: RangeInclusive<Self> =
        0xfe80_0000_0000_0000_0000_0000_0000_0000..=0xfebf_ffff_ffff_ffff_ffff_ffff_ffff_ffff;
    const PROTOCOL_ASSIGNMENTS_RANGE: RangeInclusive<Self> =
        0x2001_0000_0000_0000_0000_0000_0000_0000..=0x2001_01ff_ffff_ffff_ffff_ffff_ffff_ffff;
    const DOCUMENTATION_RANGES: &'static [RangeInclusive<Self>] =
        &[(0x2001_0db8_0000_0000_0000_0000_0000_0000..=0x2001_0db8_ffff_ffff_ffff_ffff_ffff_ffff)];
    const PRIVATE_RANGES: Option<&'static [RangeInclusive<Self>]> = None;
    const RESERVED_RANGE: Option<RangeInclusive<Self>> = None;
    const SHARED_RANGE: Option<RangeInclusive<Self>> = None;
    const THISNET_RANGE: Option<RangeInclusive<Self>> = None;
    const ULA_RANGE: Option<RangeInclusive<Self>> =
        Some(0xfc00_0000_0000_0000_0000_0000_0000_0000..=0xfd00_0000_0000_0000_0000_0000_0000_0000);

    #[allow(clippy::cast_possible_truncation)]
    fn leading_zeros(self) -> Self::Length {
        self.leading_zeros() as Self::Length
    }

    fn to_be_bytes(self) -> <Ipv6 as Afi>::Octets {
        self.to_be_bytes()
    }

    fn from_be_bytes(bytes: <Ipv6 as Afi>::Octets) -> Self {
        Self::from_be_bytes(bytes)
    }

    fn is_global(&self) -> bool {
        if Self::LOOPBACK_RANGE.contains(self)
            || Self::LINK_LOCAL_RANGE.contains(self)
            || self == &Self::UNSPECIFIED
            || Self::BENCHMARK_RANGE.contains(self)
            || Self::DOCUMENTATION_RANGES
                .iter()
                .any(|range| range.contains(self))
        {
            return false;
        }
        if let Some(range) = Self::ULA_RANGE {
            if range.contains(self) {
                return false;
            }
        }
        // non-global multicast
        if Self::MULTICAST_RANGE.contains(self)
            && self & 0x000f_0000_0000_0000_0000_0000_0000_0000
                != 0x000e_0000_0000_0000_0000_0000_0000_0000
        {
            return false;
        }
        if Self::PROTOCOL_ASSIGNMENTS_RANGE.contains(self)
            // TEREDO
            && !(0x2001_0000_0000_0000_0000_0000_0000_0000..=0x2001_0000_ffff_ffff_ffff_ffff_ffff_ffff).contains(self)
            // Port Control Protocol Anycast
            && self != &0x2001_0001_0000_0000_0000_0000_0000_0001
            // TURN Anycast
            && self != &0x2001_0001_0000_0000_0000_0000_0000_0002
            // AMT
            && !(0x2001_0003_0000_0000_0000_0000_0000_0000..=0x2001_0003_ffff_ffff_ffff_ffff_ffff_ffff).contains(self)
            // AS112
            && !(0x2001_0004_0112_0000_0000_0000_0000_0000..=0x2001_0004_0112_ffff_ffff_ffff_ffff_ffff).contains(self)
            // ORCHIDv2
            && !(0x2001_0020_0000_0000_0000_0000_0000_0000..=0x2001_002f_ffff_ffff_ffff_ffff_ffff_ffff).contains(self)
        {
            return false;
        }
        true
    }

    fn parse_addr<S>(s: &S) -> Result<Self, Error>
    where
        S: AsRef<str> + ?Sized,
    {
        parser::ipv6::parse_addr(s.as_ref())
    }

    fn parse_prefix<S>(s: &S) -> Result<(Self, Self::Length), Error>
    where
        S: AsRef<str> + ?Sized,
    {
        parser::ipv6::parse_prefix(s.as_ref())
    }
}

pub(crate) trait IntoIpv6Segments: Address<Ipv6> {
    // TODO:
    // const UNSPECIFIED_SEGMENTS: [u16; 8] = Self::UNSPECIFIED.into_segments();
    // const LOCALHOST_SEGMENTS: [u16; 8] = Self::LOCALHOST.into_segments();

    #[allow(unsafe_code)]
    fn into_segments(self) -> [u16; 8] {
        // Safety:
        // it is always safe to transmute `[u8; 16]` to `[u16; 8]`
        let [a, b, c, d, e, f, g, h]: [u16; 8] = unsafe { mem::transmute(self.to_be_bytes()) };
        [
            u16::from_be(a),
            u16::from_be(b),
            u16::from_be(c),
            u16::from_be(d),
            u16::from_be(e),
            u16::from_be(f),
            u16::from_be(g),
            u16::from_be(h),
        ]
    }

    #[allow(unsafe_code)]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn from_segments(segments: [u16; 8]) -> Self {
        // Safety:
        // it is always safe to transmute `[u16; 8]` to `[u8; 16]`
        let octets = unsafe {
            core::mem::transmute([
                segments[0].to_be(),
                segments[1].to_be(),
                segments[2].to_be(),
                segments[3].to_be(),
                segments[4].to_be(),
                segments[5].to_be(),
                segments[6].to_be(),
                segments[7].to_be(),
            ])
        };
        Self::from_be_bytes(octets)
    }
}
impl<P: Address<Ipv6>> IntoIpv6Segments for P {}

/// Underlying integer-like type used to respresent an IP prefix-length.
pub trait Length:
    Copy + Clone + Debug + Display + Hash + Ord + Add<Output = Self> + Sub<Output = Self>
{
    /// Additive identity value.
    const ZERO: Self;
    /// Additive unit value.
    const ONE: Self;
}

impl Length for u8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
