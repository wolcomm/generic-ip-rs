use core::fmt::Debug;
use core::hash::Hash;

pub trait Mask: Clone + Copy + Debug + Hash + PartialEq + Eq {}
