use crate::{
    error::{err, Error, Kind},
    traits::primitive::IntoIpv6Segments as _,
};

use super::Parser;

#[allow(clippy::inline_always)]
#[inline(always)]
pub(crate) fn parse_addr(input: &str) -> Result<u128, Error> {
    Parser::new(input)
        .take_only(Parser::take_ipv6_segments)
        .ok_or_else(|| err!(Kind::ParserError))
        .map(u128::from_segments)
}

#[allow(clippy::inline_always)]
#[inline(always)]
pub(crate) fn parse_prefix(input: &str) -> Result<(u128, u8), Error> {
    Parser::new(input)
        .take_with_length(Parser::take_ipv6_segments)
        .ok_or_else(|| err!(Kind::ParserError))
        .map(|(segments, len)| (u128::from_segments(segments), len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let input = "2001:db8:0:0:0:0:0:1";
        let addr = parse_addr(input).unwrap();
        assert_eq!(addr, 0x2001_0db8_0000_0000_0000_0000_0000_0001);
    }

    #[test]
    fn simple_elided() {
        let input = "2001:db8::";
        let addr = parse_addr(input).unwrap();
        assert_eq!(addr, 0x2001_0db8_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn complex_elided() {
        let input = "2001:db8::dead:beef";
        let addr = parse_addr(input).unwrap();
        assert_eq!(addr, 0x2001_0db8_0000_0000_0000_0000_dead_beef);
    }

    #[test]
    fn ipv4_mapped() {
        let input = "::ffff:192.0.2.1";
        let addr = parse_addr(input).unwrap();
        assert_eq!(addr, 0x0000_0000_0000_0000_0000_ffff_c000_0201);
    }

    #[test]
    fn trailing_elided() {
        let input = "::1";
        let addr = parse_addr(input).unwrap();
        assert_eq!(addr, 0x0000_0000_0000_0000_0000_0000_0000_0001);
    }

    #[test]
    fn explicit_ipv4_mapped() {
        let input = "0:0:0:0:0:ffff:192.0.2.1";
        let addr = parse_addr(input).unwrap();
        assert_eq!(addr, 0x0000_0000_0000_0000_0000_ffff_c000_0201);
    }

    #[test]
    fn disallow_excess_digits() {
        let input = "1:0ffff::";
        let addr = parse_addr(input);
        assert!(addr.is_err());
    }

    #[test]
    fn disallow_excess_parts() {
        let input = "1:2::4:5:6:7:8:9";
        let addr = parse_addr(input);
        assert!(addr.is_err());
    }

    #[test]
    fn disallow_mapped_ipv4_overflow() {
        let input = "::1:2:3:4:5:6:7.8.9.0";
        let addr = parse_addr(input);
        assert!(addr.is_err());
    }

    #[test]
    fn disallow_empty() {
        let input = "";
        let addr = parse_addr(input);
        assert!(addr.is_err());
    }

    #[test]
    fn disallow_missing_colons() {
        let input = "0";
        let addr = parse_addr(input);
        assert!(addr.is_err());
    }

    #[test]
    fn simple_prefix() {
        let input = "2001:db8::/32";
        let addr = parse_prefix(input).unwrap();
        assert_eq!(addr, (0x2001_0db8_0000_0000_0000_0000_0000_0000, 32));
    }

    #[test]
    fn ipv4_mapped_prefix() {
        let input = "::ffff:192.0.0.0/112";
        let addr = parse_prefix(input).unwrap();
        assert_eq!(addr, (0x0000_0000_0000_0000_0000_ffff_c000_0000, 112));
    }

    #[cfg(feature = "std")]
    mod proptests {
        use std::net::Ipv6Addr;
        use std::string::ToString;

        use proptest::{arbitrary::any, proptest};

        use crate::concrete::Address;

        use super::*;

        proptest! {
            #[test]
            fn parse_any_ipv6_addr(addr in any::<Ipv6Addr>()) {
                let addr_num: u128 = addr.into();
                let addr_parsed = parse_addr(&addr.to_string()).unwrap();
                assert_eq!(addr_num, addr_parsed);
            }
        }

        proptest! {
            #[test]
            fn parse_any_utf8(s in r"\PC*") {
                let stdlib: Option<Ipv6Addr> = s.parse().ok();
                assert_eq!(parse_addr(&s).map(Address::new).ok(), stdlib.map(Address::from))
            }
        }

        #[cfg(feature = "ipnet")]
        use ipnet::Ipv6Net;

        #[cfg(feature = "ipnet")]
        proptest! {
            #[test]
            fn parse_any_ipv6_prefix(addr in any::<Ipv6Addr>(), len in 0..=128u8) {
                let prefix = Ipv6Net::new(addr, len).unwrap();
                let prefix_nums = (prefix.addr().into(), prefix.prefix_len());
                std::dbg!(prefix);
                let prefix_parsed = parse_prefix(&prefix.to_string()).unwrap();
                assert_eq!(prefix_nums, prefix_parsed);
            }
        }
    }
}
