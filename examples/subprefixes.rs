use ip::{traits::Prefix as _, Error, Ipv4, Prefix};

fn main() -> Result<(), Error> {
    let prefix = "192.0.2.0/24".parse::<Prefix<Ipv4>>()?;
    let length = prefix.new_prefix_length(26)?;
    prefix.subprefixes(length)?.for_each(|p| println!("{}", p));
    Ok(())
}
