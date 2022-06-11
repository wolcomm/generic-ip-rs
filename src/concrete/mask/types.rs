use core::fmt::Debug;
use core::hash::Hash;

/// Marker trait for types of IP address mask.
pub trait Type: Copy + Debug + Hash + PartialEq + Eq {}

/// A "net"-mask, used to mask the network identifier bits of an IP address.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Net {}
impl Type for Net {}

/// A "host"-mask, used to mask the host identifier bits of an IP address.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Host {}
impl Type for Host {}
