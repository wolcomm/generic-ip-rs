use std::str::FromStr;

use ip::{traits::PrefixSet as _, Afi, Error, Ipv4, Prefix, PrefixRange, PrefixSet};

fn main() -> Result<(), Error> {
    let prefixes: Vec<Prefix<Ipv4>> = ["192.0.2.0/25", "192.0.2.128/25"]
        .into_iter()
        .map(Prefix::<Ipv4>::from_str)
        .collect::<Result<_, _>>()?;
    assert_eq!(
        into_ranges::<Ipv4, _, _>(prefixes).first().unwrap(),
        &"192.0.2.0/24,25,25".parse::<PrefixRange<Ipv4>>()?
    );
    Ok(())
}

fn into_ranges<A, I, T>(iter: I) -> Vec<PrefixRange<A>>
where
    A: Afi,
    I: IntoIterator<Item = T>,
    PrefixSet<A>: FromIterator<T>,
{
    iter.into_iter()
        .collect::<PrefixSet<A>>()
        .ranges()
        .collect()
}
