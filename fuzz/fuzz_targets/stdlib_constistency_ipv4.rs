#![no_main]

use std::net::Ipv4Addr;
use std::str::from_utf8;

use libfuzzer_sys::fuzz_target;

use ip::{Address, Ipv4};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = from_utf8(data) {
        let x = s.parse::<Ipv4Addr>().map(Address::<Ipv4>::from).ok();
        let y = s.parse::<Address<Ipv4>>().ok();
        assert_eq!(x, y);
    }
});
