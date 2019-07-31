pub mod clp;
pub mod error;
pub mod hash;
pub mod merkle_tree;

pub use self::error::Error;
pub use self::hash::*;
pub use self::merkle_tree::*;

pub trait Size {
    fn size(&self) -> u64;
}

pub trait ToBytes {
    fn send(&self) -> Vec<u8>;
}
