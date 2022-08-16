mod af;
pub use self::af::{AfiClass, Any};

mod addr;
pub use self::addr::Address;

mod mask;
pub use self::mask::{Hostmask, Mask, Netmask};

mod interface;
pub use self::interface::Interface;

mod prefix;
pub use self::prefix::{Prefix, PrefixLength};

macro_rules! delegate {
    ( $( fn $fn:ident(&self) -> $ret_ty:ty; )* ) => {
        $(
            fn $fn(&self) -> $ret_ty {
                match self {
                    Self::Ipv4(prefix) => prefix.$fn().into(),
                    Self::Ipv6(prefix) => prefix.$fn().into(),
                }
            }
        )*
    }
}
pub(self) use delegate;
