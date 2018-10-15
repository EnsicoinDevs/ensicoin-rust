use std::time::SystemTime;
use model::transaction;
use model::hash::Hash;
use sha2::{Sha256, Digest};
use std::time;

static mut current_index: u64 = 0;

unsafe fn increment_index() {
    current_index += 1;
}

#[derive(Debug)]
pub struct Block {
    pub version : u64,
    pub index : u64,
    pub timestamp : SystemTime,
    pub hash : Hash,
    pub previous_hash : Hash,
    pub nonce : u64,
    pub transactions : Vec<transaction::Transaction>,
}

impl Block {

    pub unsafe fn new() -> Block {
        let mut block = Block{ version : 0, index : current_index, timestamp : SystemTime::now(), hash : Hash { val: Vec::new() }, previous_hash : Hash { val: Vec::new() }, nonce : 0, transactions : Vec::new()};
        block.hash = block.hash();
        increment_index();
        return block;
    }

    pub fn hash(&self) -> Hash {
        let hash_string = format!("{}{}{}{}", self.version, self.index, self.timestamp.to_string(), self.previous_hash.to_string());
        let mut sha = Sha256::new();
        sha.input(hash_string);
        let result = sha.result();
        let result = result[..].to_vec();
        let hash = Hash { val: result };
        hash
    }
}
