use super::{Address, Mask};

pub trait Prefix: Sized {
    type Address: Address;
    type PrefixLength: PrefixLength;
    type Hostmask: Mask;
    type Netmask: Mask;
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

pub trait PrefixLength {}
