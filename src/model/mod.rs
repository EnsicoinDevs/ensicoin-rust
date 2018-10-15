pub mod block;
pub mod transaction;
pub mod hash;

use sha2::{Sha256, Digest};
use self::hash::Hash;

pub fn hashBlock(version : u64, index : u64, timestamp : u64, previous_hash : Hash, nonce : u64) -> Hash {
    let hash_string = format!("{}{}{}{}", version, index, timestamp, previous_hash.to_string());
    let mut sha = Sha256::new();
    sha.input(hash_string);
    let result = sha.result();
    let result = result[..].to_vec();
    let hash = Hash { val: result };
    hash
}
