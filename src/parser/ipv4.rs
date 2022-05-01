use crate::{
    concrete::Ipv4,
    error::{err, Error, ErrorKind},
};

use super::Parser;

#[inline(always)]
pub fn parse_addr(input: &str) -> Result<u32, Error<'static, Ipv4>> {
    Parser::new(input)
        .take_only(Parser::take_ipv4_octets)
        .ok_or_else(|| err!(ErrorKind::ParserError))
        .map(u32::from_be_bytes)
}

#[inline(always)]
pub fn parse_prefix(input: &str) -> Result<(u32, u8), Error<'static, Ipv4>> {
    Parser::new(input)
        .take_with_length(Parser::take_ipv4_octets)
        .ok_or_else(|| err!(ErrorKind::ParserError))
        .map(|(octets, len)| (u32::from_be_bytes(octets), len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ipv4_addr() {
        let input = "10.1.1.1";
        let addr = parse_addr(input).unwrap();
        assert_eq!(addr, 0x0a01_0101)
    }

    #[test]
    fn empty_octets() {
        let input = "...";
        let addr = parse_addr(input);
        assert!(addr.is_err());
    }

    #[test]
    fn disallow_octal() {
        let input = "1.01.0.0";
        let addr = parse_addr(input);
        #[cfg(feature = "std")]
        std::println!("{:?}", addr);
        assert!(addr.is_err());
    }

    #[test]
    fn consume_all_input() {
        let input = "192.168.0.1\0";
        let addr = parse_addr(input);
        assert!(addr.is_err());
    }

    #[cfg(feature = "std")]
    mod proptests {
        use std::net::Ipv4Addr;
        use std::string::ToString;

        use proptest::{arbitrary::any, proptest};

        use crate::concrete::Address;

        use super::*;

        proptest! {
            #[test]
            fn parse_any_ipv4_addr(addr in any::<Ipv4Addr>()) {
                let addr_num: u32 = addr.into();
                let addr_parsed = parse_addr(&addr.to_string()).unwrap();
                assert_eq!(addr_num, addr_parsed);
            }
        }

        proptest! {
            #[test]
            fn parse_any_utf8(s in r"\PC*") {
                let stdlib: Option<Ipv4Addr> = s.parse().ok();
                assert_eq!(parse_addr(&s).map(Address::new).ok(), stdlib.map(Address::from))
            }
        }

        #[cfg(feature = "ipnet")]
        use ipnet::Ipv4Net;

        #[cfg(feature = "ipnet")]
        proptest! {
            #[test]
            fn parse_any_ipv4_prefix(addr in any::<Ipv4Addr>(), len in 0..=32u8) {
                let prefix = Ipv4Net::new(addr, len).unwrap().trunc();
                let prefix_nums = (prefix.network().into(), prefix.prefix_len());
                let prefix_parsed = parse_prefix(&prefix.to_string()).unwrap();
                assert_eq!(prefix_nums, prefix_parsed);
            }
        }
    }
}
