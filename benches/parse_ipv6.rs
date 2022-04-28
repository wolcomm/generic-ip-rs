use core::str::FromStr;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use ip::{Address, Ipv6, Prefix};

pub fn addr_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipv6-address");
    ["2001:db8::1", "2c0f:fa90:f00:baa::ffff", "::ffff:10.0.0.1"]
        .iter()
        .for_each(|addr| {
            #[cfg(feature = "std")]
            group.bench_with_input(BenchmarkId::new("stdlib", addr), addr, |b, addr| {
                b.iter(|| std::net::Ipv6Addr::from_str(addr))
            });
            group.bench_with_input(BenchmarkId::new("crate", addr), addr, |b, addr| {
                b.iter(|| Address::<Ipv6>::from_str(addr))
            });
        });
    group.finish();
}

pub fn prefix_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipv6-prefix");
    [
        "2001:db8::/32",
        "2c0f:fa90:f00:baa::/64",
        "::ffff:10.0.0.0/102",
    ]
    .iter()
    .for_each(|prefix| {
        #[cfg(feature = "ipnet")]
        group.bench_with_input(BenchmarkId::new("ipnet", prefix), prefix, |b, prefix| {
            b.iter(|| ipnet::Ipv6Net::from_str(prefix))
        });
        group.bench_with_input(BenchmarkId::new("crate", prefix), prefix, |b, prefix| {
            b.iter(|| Prefix::<Ipv6>::from_str(prefix))
        });
    });
    group.finish();
}

criterion_group!(benches, addr_benchmark, prefix_benchmark,);
criterion_main!(benches);
