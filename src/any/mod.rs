mod af;
pub use self::af::{AfiClass, Any};

mod addr;
pub use self::addr::Address;

mod mask;
pub use self::mask::{Bitmask, Hostmask, Mask, Netmask};

mod interface;
pub use self::interface::Interface;

mod prefix;
#[cfg(feature = "std")]
pub use self::prefix::Set as PrefixSet;
pub use self::prefix::{Length as PrefixLength, Prefix, Range as PrefixRange, Subprefixes};

macro_rules! delegate {
    ( $( fn $fn:ident(&self) -> $ret_ty:ty; )* ) => {
        $(
            fn $fn(&self) -> $ret_ty {
                match self {
                    Self::Ipv4(inner) => inner.$fn().into(),
                    Self::Ipv6(inner) => inner.$fn().into(),
                }
            }
        )*
    };
}
use delegate;
