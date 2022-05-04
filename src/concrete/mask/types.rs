use core::fmt::Debug;
use core::hash::Hash;

pub trait Type: Copy + Debug + Hash + PartialEq + Eq {}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Net {}
impl Type for Net {}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Host {}
impl Type for Host {}
