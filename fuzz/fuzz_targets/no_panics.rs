#![no_main]

use std::str::{from_utf8, FromStr};

use libfuzzer_sys::fuzz_target;

use ip::{
    addr::Address,
    af::{Ipv4, Ipv6},
    prefix::Prefix,
};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = from_utf8(data) {
        let _ = s.parse::<Address<Ipv4>>();
        let _ = s.parse::<Address<Ipv6>>();
        let _ = s.parse::<Prefix<Ipv4>>();
        let _ = s.parse::<Prefix<Ipv6>>();
    }
});
