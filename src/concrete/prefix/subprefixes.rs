use crate::{
    error::{err, Error, Kind},
    traits::{Afi, Prefix as _, PrefixLength as _},
};

use super::{Bitmask, Hostmask, Netmask, Prefix, PrefixLength};

/// Iterator returned by [`Prefix::subprefixes`].
#[derive(Debug, Clone)]
pub struct Subprefixes<A: Afi> {
    base: Prefix<A>,
    next: Option<Prefix<A>>,
    step: Option<Bitmask<A>>,
}

impl<A: Afi> Subprefixes<A> {
    pub(super) fn new(base: Prefix<A>, length: PrefixLength<A>) -> Result<Self, Error> {
        if length < base.length() {
            Err(err!(Kind::PrefixLength))
        } else {
            let next = Some(Prefix::new(base.prefix(), length));
            let step = length
                .decrement()
                .map(Hostmask::from)
                .map(|hostbits| hostbits & Netmask::from(length))
                .ok();
            Ok(Self { base, next, step })
        }
    }
}

impl<A: Afi> Iterator for Subprefixes<A> {
    type Item = Prefix<A>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next.take() {
            self.next = self
                .step
                .and_then(|step| next.map_addr(|addr| addr + step))
                .filter(|prefix| self.base.contains(prefix));
            Some(next)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{traits::Prefix as _, Any, Ipv4, Ipv6, Prefix};

    #[test]
    fn singleton_subprefix_ipv4() {
        let p: Prefix<Ipv4> = "192.0.2.0/24".parse().unwrap();
        let mut subprefixes = p.subprefixes(p.prefix_len()).unwrap();
        assert_eq!(subprefixes.next(), Some(p));
        assert_eq!(subprefixes.next(), None);
    }

    #[test]
    fn singleton_subprefix_ipv6() {
        let p: Prefix<Ipv6> = "2001:db8::/48".parse().unwrap();
        let mut subprefixes = p.subprefixes(p.prefix_len()).unwrap();
        assert_eq!(subprefixes.next(), Some(p));
        assert_eq!(subprefixes.next(), None);
    }

    #[test]
    fn singleton_subprefix_any() {
        let p: Prefix<Any> = "2001:db8:f00::/64".parse().unwrap();
        let mut subprefixes = p.subprefixes(p.prefix_len()).unwrap();
        assert_eq!(subprefixes.next(), Some(p));
        assert_eq!(subprefixes.next(), None);
    }

    #[test]
    fn count_subprefixes_of_default() {
        let p: Prefix<Ipv4> = "0.0.0.0/0".parse().unwrap();
        let len = p.new_prefix_length(8).unwrap();
        assert_eq!(p.subprefixes(len).unwrap().count(), 256);
    }
}
