#![no_main]

use std::net::Ipv6Addr;
use std::str::from_utf8;

use libfuzzer_sys::fuzz_target;

use ip::{Address, Ipv6};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = from_utf8(data) {
        let x = s.parse::<Ipv6Addr>().map(Address::<Ipv6>::from).ok();
        let y = s.parse::<Address<Ipv6>>().ok();
        assert_eq!(x, y);
    }
});
