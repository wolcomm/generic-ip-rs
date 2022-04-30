use core::fmt::Debug;

pub trait Type: Copy + Debug {}

#[derive(Clone, Copy, Debug)]
pub enum Net {}
impl Type for Net {}

#[derive(Clone, Copy, Debug)]
pub enum Host {}
impl Type for Host {}
