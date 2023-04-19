use super::Parser;
use crate::error::{err, Error, Kind};

#[allow(clippy::inline_always)]
#[inline(always)]
pub(crate) fn parse_addr(input: &str) -> Result<u32, Error> {
    Parser::new(input)
        .take_only(Parser::take_ipv4_octets)
        .ok_or_else(|| err!(Kind::ParserError))
        .map(u32::from_be_bytes)
}

#[allow(clippy::inline_always)]
#[inline(always)]
pub(crate) fn parse_prefix(input: &str) -> Result<(u32, u8), Error> {
    Parser::new(input)
        .take_with_length(Parser::take_ipv4_octets)
        .ok_or_else(|| err!(Kind::ParserError))
        .map(|(octets, len)| (u32::from_be_bytes(octets), len))
}

#[allow(clippy::inline_always)]
#[inline(always)]
pub(crate) fn parse_range(input: &str) -> Result<(u32, u8, u8, u8), Error> {
    Parser::new(input)
        .take_with_length_range(Parser::take_ipv4_octets)
        .ok_or_else(|| err!(Kind::ParserError))
        .map(|(octets, len, lower, upper)| (u32::from_be_bytes(octets), len, lower, upper))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ipv4_addr() {
        let input = "10.1.1.1";
        let addr = parse_addr(input).unwrap();
        assert_eq!(addr, 0x0a01_0101);
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
        std::println!("{addr:?}");
        assert!(addr.is_err());
    }

    #[test]
    fn consume_all_input() {
        let input = "192.168.0.1\0";
        let addr = parse_addr(input);
        assert!(addr.is_err());
    }

    #[test]
    fn prefix_range() {
        let input = "192.0.2.0/24,25,26";
        let range = parse_range(input).unwrap();
        assert_eq!(range, (0xc000_0200, 24, 25, 26));
    }

    #[cfg(feature = "std")]
    mod proptests {
        use std::net::Ipv4Addr;
        use std::string::ToString;

        use proptest::{arbitrary::any, proptest};

        use super::*;
        use crate::concrete::Address;

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
                assert_eq!(parse_addr(&s).map(Address::new).ok(), stdlib.map(Address::from));
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
