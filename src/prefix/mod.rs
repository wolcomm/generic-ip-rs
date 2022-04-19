use core::cmp::min;

use crate::{
    addr::{common_length, Address, Netmask},
    af::Afi,
};

mod len;
mod ord;

pub use self::len::PrefixLength;
pub use self::ord::PrefixOrdering as Ordering;

mod private {
    use super::*;

    /// An IP prefix, consisting of a network address and prefix length.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub struct Prefix<A: Afi> {
        prefix: Address<A>,
        length: PrefixLength<A>,
    }

    impl<A: Afi> Prefix<A> {
        /// Construct a new [`Prefix<A>`] from an address and prefix length.
        ///
        /// The host bits of `prefix` will be automatically set to zero.
        pub fn new(mut prefix: Address<A>, length: PrefixLength<A>) -> Self {
            prefix &= Netmask::from(length);
            Self { prefix, length }
        }

        /// Get the network address of this prefix.
        pub fn prefix(&self) -> Address<A> {
            self.prefix
        }

        /// Get the length of this prefix.
        pub fn length(&self) -> PrefixLength<A> {
            self.length
        }
    }
}

pub use self::private::Prefix;

impl<A: Afi> Prefix<A> {
    fn common_with(self, other: Self) -> Self {
        let min_length = min(self.length(), other.length());
        let common_length = common_length(self.prefix(), other.prefix());
        let length = min(min_length, common_length);
        Self::new(self.prefix(), length)
    }
}

mod parse {
    use super::*;

    use core::str::FromStr;

    use crate::error::Error;

    impl<A: Afi> FromStr for Prefix<A> {
        type Err = Error<'static, A>;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            A::parse_prefix(s).and_then(|(addr, len)| {
                Ok(Self::new(
                    Address::new(addr),
                    PrefixLength::from_primitive(len)?,
                ))
            })
        }
    }
}

mod fmt {
    use super::*;

    use core::fmt;

    use crate::fmt::AddressDisplay;

    impl<A: Afi> fmt::Display for Prefix<A>
    where
        A::Addr: AddressDisplay<A>,
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}/{}", self.prefix(), self.length())
        }
    }
}

#[cfg(feature = "ipnet")]
mod convert {
    use super::*;

    use crate::af::{Ipv4, Ipv6};

    use ipnet::{Ipv4Net, Ipv6Net};

    impl From<Ipv4Net> for Prefix<Ipv4> {
        fn from(net: Ipv4Net) -> Self {
            let prefix = net.network().into();
            let length = PrefixLength::from_primitive(net.prefix_len())
                .expect("we trusted `ipnet` to enforce length bounds");
            Self::new(prefix, length)
        }
    }

    impl From<Ipv6Net> for Prefix<Ipv6> {
        fn from(net: Ipv6Net) -> Self {
            let prefix = net.network().into();
            let length = PrefixLength::from_primitive(net.prefix_len())
                .expect("we trusted `ipnet` to enforce length bounds");
            Self::new(prefix, length)
        }
    }
}

#[cfg(any(test, feature = "arbitrary"))]
mod arbitrary {
    use super::*;

    use proptest::{
        arbitrary::{any_with, Arbitrary, ParamsFor, StrategyFor},
        strategy::{BoxedStrategy, Strategy},
    };

    impl<A: Afi> Arbitrary for Prefix<A>
    where
        Address<A>: Arbitrary,
        StrategyFor<Address<A>>: 'static,
        PrefixLength<A>: Arbitrary,
        StrategyFor<PrefixLength<A>>: 'static,
    {
        type Parameters = ParamsFor<(Address<A>, PrefixLength<A>)>;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            (
                any_with::<Address<A>>(params.0),
                any_with::<PrefixLength<A>>(params.1),
            )
                .prop_map(|(prefix, length)| Self::new(prefix, length))
                .boxed()
        }
    }
}
