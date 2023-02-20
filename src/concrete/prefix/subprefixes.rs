use crate::{
    error::{err, Error, Kind},
    traits::Afi,
};

use super::{Address, Prefix, PrefixLength};

pub struct Subprefixes<A: Afi> {
    base: Address<A>,
}

impl<A: Afi> Subprefixes<A> {
    pub(super) fn new(base: Prefix<A>, length: PrefixLength<A>) -> Result<Self, Error> {
        if length < base.length() {
            Err(err!(Kind::PrefixLength))
        } else {
            Ok(Self {
                base: base.prefix(),
            })
        }
    }
}

impl<A: Afi> Iterator for Subprefixes<A> {
    type Item = Prefix<A>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
