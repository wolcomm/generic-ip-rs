use core::any::Any;
use ip::{traits::Prefix as _, Error, Ipv4, Prefix};

fn main() -> Result<(), Error> {
    let prefix = "192.0.2.0/24".parse::<Prefix<Ipv4>>()?;
    dbg!(&prefix);
    let new_length = prefix.new_prefix_length(24)?;
    dbg!(&new_length);
    Ok(())
}
