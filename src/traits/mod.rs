mod af;
pub use self::af::{Afi, AfiClass};

mod addr;
pub use self::addr::Address;

mod mask;
pub use self::mask::{Hostmask, Mask, Netmask};

mod prefix;
pub use self::prefix::{Length as PrefixLength, Prefix};

/// Traits for defining underlying integer primitives used in IP objects.
pub mod primitive;
