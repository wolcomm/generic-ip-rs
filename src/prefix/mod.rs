use core::cmp::min;

use crate::{
    addr::{common_length, AddressI, AnyAddress, ConcreteAddress},
    af::{Afi, Ipv4, Ipv6},
    mask::{AnyHostmask, AnyNetmask, ConcreteHostmask, ConcreteNetmask, MaskI},
    primitive::AddressPrimitive,
};

mod len;
mod ord;

pub use self::len::{AnyPrefixLength, ConcretePrefixLength, PrefixLengthI};
pub use self::ord::PrefixOrdering as Ordering;

mod private {
    use super::*;

    /// An IP prefix, consisting of a network address and prefix length.
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub struct ConcretePrefix<A: Afi> {
        prefix: ConcreteAddress<A>,
        length: ConcretePrefixLength<A>,
    }

    impl<A: Afi> ConcretePrefix<A> {
        /// Construct a new [`Prefix<A>`] from an address and prefix length.
        ///
        /// The host bits of `prefix` will be automatically set to zero.
        pub fn new(mut prefix: ConcreteAddress<A>, length: ConcretePrefixLength<A>) -> Self {
            prefix &= ConcreteNetmask::from(length);
            Self { prefix, length }
        }

        /// Get the network address of this prefix.
        pub fn prefix(&self) -> ConcreteAddress<A> {
            self.prefix
        }

        /// Get the length of this prefix.
        pub fn length(&self) -> ConcretePrefixLength<A> {
            self.length
        }
    }
}

pub use self::private::ConcretePrefix;

impl ConcretePrefix<Ipv4> {}
impl ConcretePrefix<Ipv6> {}
impl<A: Afi> ConcretePrefix<A> {}

pub enum AnyPrefix {
    Ipv4(ConcretePrefix<Ipv4>),
    Ipv6(ConcretePrefix<Ipv6>),
}

pub trait PrefixI: Sized {
    type Address: AddressI;
    type PrefixLength: PrefixLengthI;
    type Hostmask: MaskI;
    type Netmask: MaskI;
    // TODO:
    // type Hosts: Iterator<Item = Self::Address>;
    // type Subnets: Iterator<Item = Self>;

    fn network(&self) -> Self::Address;
    // TODO: remove from Prefix, add to `Interface`
    fn addr(&self) -> Self::Address;
    // TODO: remove from Prefix, add to `Interface`
    fn trunc(&self) -> Self;
    fn hostmask(&self) -> Self::Hostmask;
    fn netmask(&self) -> Self::Netmask;
    fn max_prefix_len(&self) -> Self::PrefixLength;
    fn prefix_len(&self) -> Self::PrefixLength;
    fn broadcast(&self) -> Self::Address;
    fn supernet(&self) -> Option<Self>;
    fn is_sibling(&self, other: &Self) -> bool;

    fn contains<T>(&self, other: T) -> bool
    where
        Self: PartialOrd<T>,
    {
        self.ge(&other)
    }

    // TODO:
    // #[cfg(feature = "std")]
    // fn aggregate(networks: &std::vec::Vec<Self>) -> std::vec::Vec<Self>;
    // fn hosts(&self) -> Self::Hosts;
    // fn subnets(
    //     &self,
    //     new_prefix_len: Self::PrefixLength,
    // ) -> Result<Self::Subnets, Error<'static, A, P>>;
}

impl<A: Afi> PrefixI for ConcretePrefix<A> {
    type Address = ConcreteAddress<A>;
    type PrefixLength = ConcretePrefixLength<A>;
    type Hostmask = ConcreteHostmask<A>;
    type Netmask = ConcreteNetmask<A>;

    fn network(&self) -> Self::Address {
        self.prefix()
    }

    fn addr(&self) -> Self::Address {
        self.prefix()
    }

    fn trunc(&self) -> Self {
        *self
    }

    fn hostmask(&self) -> Self::Hostmask {
        self.length().into()
    }

    fn netmask(&self) -> Self::Netmask {
        self.length().into()
    }

    fn max_prefix_len(&self) -> Self::PrefixLength {
        Self::PrefixLength::MAX
    }

    fn prefix_len(&self) -> Self::PrefixLength {
        self.length()
    }

    fn broadcast(&self) -> Self::Address {
        self.network() | self.hostmask()
    }

    fn supernet(&self) -> Option<Self> {
        self.length()
            .decrement()
            .map(|len| Self::new(self.prefix(), len))
            .ok()
    }

    fn is_sibling(&self, other: &Self) -> bool {
        self.supernet() == other.supernet()
    }
}

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
impl PrefixI for AnyPrefix {
    type Address = AnyAddress;
    type PrefixLength = AnyPrefixLength;
    type Hostmask = AnyHostmask;
    type Netmask = AnyNetmask;

    delegate! {
        fn network(&self) -> Self::Address;
        fn addr(&self) -> Self::Address;
        fn trunc(&self) -> Self;
        fn hostmask(&self) -> Self::Hostmask;
        fn netmask(&self) -> Self::Netmask;
        fn max_prefix_len(&self) -> Self::PrefixLength;
        fn prefix_len(&self) -> Self::PrefixLength;
        fn broadcast(&self) -> Self::Address;
    }

    fn supernet(&self) -> Option<Self> {
        match self {
            Self::Ipv4(prefix) => prefix.supernet().map(Self::Ipv4),
            Self::Ipv6(prefix) => prefix.supernet().map(Self::Ipv6),
        }
    }

    fn is_sibling(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ipv4(prefix), Self::Ipv4(other)) => prefix.is_sibling(other),
            (Self::Ipv6(prefix), Self::Ipv6(other)) => prefix.is_sibling(other),
            _ => false,
        }
    }
}

impl From<ConcretePrefix<Ipv4>> for AnyPrefix {
    fn from(prefix: ConcretePrefix<Ipv4>) -> Self {
        Self::Ipv4(prefix)
    }
}

impl From<ConcretePrefix<Ipv6>> for AnyPrefix {
    fn from(prefix: ConcretePrefix<Ipv6>) -> Self {
        Self::Ipv6(prefix)
    }
}

impl<A: Afi> ConcretePrefix<A> {
    fn common_with(self, other: Self) -> Self {
        let min_length = min(self.length(), other.length());
        let common_length = common_length(self.prefix(), other.prefix());
        let length = min(min_length, common_length);
        Self::new(self.prefix(), length)
    }
}

impl<A: Afi> From<ConcreteAddress<A>> for ConcretePrefix<A> {
    fn from(addr: ConcreteAddress<A>) -> Self {
        Self::new(addr, ConcretePrefixLength::MAX)
    }
}

mod parse {
    use super::*;

    use core::str::FromStr;

    use crate::error::Error;

    impl<A: Afi> FromStr for ConcretePrefix<A> {
        type Err = Error<'static, A>;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            A::AddressPrimitive::parse_prefix(s).and_then(|(addr, len)| {
                Ok(Self::new(
                    ConcreteAddress::new(addr),
                    ConcretePrefixLength::from_primitive(len)?,
                ))
            })
        }
    }
}

mod fmt {
    use super::*;

    use core::fmt;

    use crate::fmt::AddressDisplay;

    impl<A: Afi> fmt::Display for ConcretePrefix<A>
    where
        A::AddressPrimitive: AddressDisplay<A>,
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

    impl From<Ipv4Net> for ConcretePrefix<Ipv4> {
        fn from(net: Ipv4Net) -> Self {
            let prefix = net.network().into();
            let length = ConcretePrefixLength::from_primitive(net.prefix_len())
                .expect("we trusted `ipnet` to enforce length bounds");
            Self::new(prefix, length)
        }
    }

    impl From<Ipv6Net> for ConcretePrefix<Ipv6> {
        fn from(net: Ipv6Net) -> Self {
            let prefix = net.network().into();
            let length = ConcretePrefixLength::from_primitive(net.prefix_len())
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

    impl<A: Afi> Arbitrary for ConcretePrefix<A>
    where
        ConcreteAddress<A>: Arbitrary,
        StrategyFor<ConcreteAddress<A>>: 'static,
        ConcretePrefixLength<A>: Arbitrary,
        StrategyFor<ConcretePrefixLength<A>>: 'static,
    {
        type Parameters = ParamsFor<(ConcreteAddress<A>, ConcretePrefixLength<A>)>;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            (
                any_with::<ConcreteAddress<A>>(params.0),
                any_with::<ConcretePrefixLength<A>>(params.1),
            )
                .prop_map(|(prefix, length)| Self::new(prefix, length))
                .boxed()
        }
    }
}
