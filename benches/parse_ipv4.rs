use core::str::FromStr;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use ip::{Address, Ipv4, Prefix};

pub fn addr_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipv4-address");
    [
        "10.1.1.1",
        "172.16.0.255",
        "198.10.199.250",
        "240.130.10.10",
    ]
    .iter()
    .for_each(|addr| {
        #[cfg(feature = "std")]
        group.bench_with_input(BenchmarkId::new("stdlib", addr), addr, |b, addr| {
            b.iter(|| std::net::Ipv4Addr::from_str(addr))
        });
        group.bench_with_input(BenchmarkId::new("crate", addr), addr, |b, addr| {
            b.iter(|| Address::<Ipv4>::from_str(addr))
        });
    });
    group.finish();
}

pub fn prefix_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipv4-prefix");
    [
        "10.1.1.1/32",
        "172.16.0.0/12",
        "198.10.199.250/31",
        "240.130.10.176/29",
    ]
    .iter()
    .for_each(|prefix| {
        #[cfg(feature = "ipnet")]
        group.bench_with_input(BenchmarkId::new("ipnet", prefix), prefix, |b, prefix| {
            b.iter(|| ipnet::Ipv4Net::from_str(prefix))
        });
        group.bench_with_input(BenchmarkId::new("crate", prefix), prefix, |b, prefix| {
            b.iter(|| Prefix::<Ipv4>::from_str(prefix))
        });
    });
    group.finish();
}

criterion_group!(benches, addr_benchmark, prefix_benchmark,);
criterion_main!(benches);
