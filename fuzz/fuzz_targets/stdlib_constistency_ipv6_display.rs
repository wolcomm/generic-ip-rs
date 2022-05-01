#![no_main]
use std::net::Ipv6Addr;

use libfuzzer_sys::fuzz_target;

use ip::{Address, Ipv6};

fuzz_target!(|x: Ipv6Addr| {
    let y: Address<Ipv6> = x.into();
    assert_eq!(x.to_string(), y.to_string());
});
