#![no_main]

use std::net::Ipv4Addr;

use libfuzzer_sys::fuzz_target;

use ip::{addr::Address, af::Ipv4};

fuzz_target!(|x: Ipv4Addr| {
    let y: Address<Ipv4> = x.into();
    assert_eq!(x.to_string(), y.to_string());
});
