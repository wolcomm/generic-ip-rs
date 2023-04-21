mod af;
pub use self::af::{Afi, Ipv4, Ipv6};

mod addr;
pub use self::addr::{common_length, Address, Ipv6MulticastScope, Range as AddressRange};

mod mask;
pub use self::mask::{types as mask_types, Bitmask, Hostmask, Mask, Netmask};

mod interface;
pub use self::interface::Interface;

mod prefix;
#[cfg(feature = "std")]
pub use self::prefix::Set as PrefixSet;
pub use self::prefix::{Prefix, PrefixLength, PrefixOrdering, Range as PrefixRange, Subprefixes};

macro_rules! impl_try_from_any {
    ( $any_ty:ty {
        $( $variant:path => $concrete_ty:ty ),* $(,)?
    } ) => {
        $(
            impl TryFrom<$any_ty> for $concrete_ty {
                type Error = $crate::error::Error;

                fn try_from(from: $any_ty) -> Result<Self, Self::Error> {
                    if let $variant(inner) = from {
                        Ok(inner)
                    } else {
                        Err($crate::error::err!($crate::error::Kind::AfiMismatch))
                    }
                }
            }
        )*
    }
}
pub(self) use impl_try_from_any;
