use core::cmp::min;

use crate::{
    addr::{
        common_length, AddressI, AnyAddress, AnyHostmask, AnyNetmask, ConcreteAddress,
        ConcreteHostmask, ConcreteNetmask, MaskI,
    },
    af::{Afi, DefaultPrimitive, Ipv4, Ipv6},
    error::Error,
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
    pub struct ConcretePrefix<A: Afi, P: AddressPrimitive<A> = DefaultPrimitive<A>> {
        prefix: ConcreteAddress<A, P>,
        length: ConcretePrefixLength<A, P>,
    }

    impl<A: Afi, P: AddressPrimitive<A>> ConcretePrefix<A, P> {
        // TODO: use `Self::new()` to construct these (and move out of `mod
        // private`) once const trait bounds are available in stable rustc
        // (1.61+)
        pub const LOCALHOST: Self = Self {
            prefix: ConcreteAddress::LOCALHOST_NET,
            length: ConcretePrefixLength::LOCALHOST_NET,
        };
        pub const BENCHMARK: Self = Self {
            prefix: ConcreteAddress::BENCHMARK_NET,
            length: ConcretePrefixLength::BENCHMARK_NET,
        };
        pub const MULTICAST: Self = Self {
            prefix: ConcreteAddress::MULTICAST_NET,
            length: ConcretePrefixLength::MULTICAST_NET,
        };

        /// Construct a new [`Prefix<A>`] from an address and prefix length.
        ///
        /// The host bits of `prefix` will be automatically set to zero.
        pub fn new(mut prefix: ConcreteAddress<A, P>, length: ConcretePrefixLength<A, P>) -> Self {
            prefix &= ConcreteNetmask::from(length);
            Self { prefix, length }
        }

        /// Get the network address of this prefix.
        pub fn prefix(&self) -> ConcreteAddress<A, P> {
            self.prefix
        }

        /// Get the length of this prefix.
        pub fn length(&self) -> ConcretePrefixLength<A, P> {
            self.length
        }
    }
}

pub use self::private::ConcretePrefix;

impl<P: AddressPrimitive<Ipv4>> ConcretePrefix<Ipv4, P> {}
impl<P: AddressPrimitive<Ipv6>> ConcretePrefix<Ipv6, P> {}
impl<A: Afi, P: AddressPrimitive<A>> ConcretePrefix<A, P> {}

pub enum AnyPrefix<P4 = DefaultPrimitive<Ipv4>, P6 = DefaultPrimitive<Ipv6>>
where
    P4: AddressPrimitive<Ipv4>,
    P6: AddressPrimitive<Ipv6>,
{
    Ipv4(ConcretePrefix<Ipv4, P4>),
    Ipv6(ConcretePrefix<Ipv6, P6>),
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

impl<A: Afi, P: AddressPrimitive<A>> PrefixI for ConcretePrefix<A, P> {
    type Address = ConcreteAddress<A, P>;
    type PrefixLength = ConcretePrefixLength<A, P>;
    type Hostmask = ConcreteHostmask<A, P>;
    type Netmask = ConcreteNetmask<A, P>;

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
impl<P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>> PrefixI for AnyPrefix<P4, P6> {
    type Address = AnyAddress<P4, P6>;
    type PrefixLength = AnyPrefixLength<P4, P6>;
    type Hostmask = AnyHostmask<P4, P6>;
    type Netmask = AnyNetmask<P4, P6>;

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

impl<P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>> From<ConcretePrefix<Ipv4, P4>>
    for AnyPrefix<P4, P6>
{
    fn from(prefix: ConcretePrefix<Ipv4, P4>) -> Self {
        Self::Ipv4(prefix)
    }
}

impl<P4: AddressPrimitive<Ipv4>, P6: AddressPrimitive<Ipv6>> From<ConcretePrefix<Ipv6, P6>>
    for AnyPrefix<P4, P6>
{
    fn from(prefix: ConcretePrefix<Ipv6, P6>) -> Self {
        Self::Ipv6(prefix)
    }
}

impl<A: Afi, P: AddressPrimitive<A>> ConcretePrefix<A, P> {
    fn common_with(self, other: Self) -> Self {
        let min_length = min(self.length(), other.length());
        let common_length = common_length(self.prefix(), other.prefix());
        let length = min(min_length, common_length);
        Self::new(self.prefix(), length)
    }
}

impl<A: Afi, P: AddressPrimitive<A>> From<ConcreteAddress<A, P>> for ConcretePrefix<A, P> {
    fn from(addr: ConcreteAddress<A, P>) -> Self {
        Self::new(addr, ConcretePrefixLength::MAX)
    }
}

mod parse {
    use super::*;

    use core::str::FromStr;

    use crate::error::Error;

    impl<A: Afi, P: AddressPrimitive<A>> FromStr for ConcretePrefix<A, P> {
        type Err = Error<'static, A, P>;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            P::parse_prefix(s).and_then(|(addr, len)| {
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

    impl<A: Afi, P: AddressPrimitive<A>> fmt::Display for ConcretePrefix<A, P>
    where
        P: AddressDisplay<A>,
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
    use std::net::{Ipv4Addr, Ipv6Addr};

    impl<P> From<Ipv4Net> for ConcretePrefix<Ipv4, P>
    where
        P: AddressPrimitive<Ipv4, Width = u8> + From<Ipv4Addr>,
    {
        fn from(net: Ipv4Net) -> Self {
            let prefix = net.network().into();
            let length = ConcretePrefixLength::from_primitive(net.prefix_len())
                .expect("we trusted `ipnet` to enforce length bounds");
            Self::new(prefix, length)
        }
    }

    impl<P> From<Ipv6Net> for ConcretePrefix<Ipv6, P>
    where
        P: AddressPrimitive<Ipv6, Width = u8> + From<Ipv6Addr>,
    {
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

    impl<A: Afi, P: AddressPrimitive<A>> Arbitrary for ConcretePrefix<A, P>
    where
        ConcreteAddress<A, P>: Arbitrary,
        StrategyFor<ConcreteAddress<A, P>>: 'static,
        ConcretePrefixLength<A, P>: Arbitrary,
        StrategyFor<ConcretePrefixLength<A, P>>: 'static,
    {
        type Parameters = ParamsFor<(ConcreteAddress<A, P>, ConcretePrefixLength<A, P>)>;
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
            (
                any_with::<ConcreteAddress<A, P>>(params.0),
                any_with::<ConcretePrefixLength<A, P>>(params.1),
            )
                .prop_map(|(prefix, length)| Self::new(prefix, length))
                .boxed()
        }
    }
}
