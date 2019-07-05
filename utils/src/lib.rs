// #![feature(try_trait)]
extern crate bincode;
#[macro_use]
extern crate clap;
extern crate sha2;
extern crate sled;

pub mod clp;
pub mod error;
pub mod hash;
pub mod merkle_tree;

pub use self::error::Error;
pub use self::hash::*;
pub use self::merkle_tree::*;
