use std::time::SystemTime;
use model::transaction;
use model::hash::ToHex;
use sha2::{Sha256, Digest};

static mut current_index: u64 = 0;

unsafe fn increment_index() {
    current_index += 1;
}

#[derive(Debug)]
pub struct Block {
    pub version : u64,
    pub index : u64,
    pub timestamp : SystemTime,
    pub hash : String,
    pub previous_hash : String,
    pub nonce : u64,
    pub transactions : Vec<transaction::Transaction>,
}

impl Block {

    // pub fn genesis_block() -> Block {
    //     Block : {
    //         version : 1
    //     }
    // }

    pub unsafe fn new() -> Block {
        let mut block = Block{ version : 0, index : current_index, timestamp : SystemTime::now(), hash : "".to_string(), previous_hash : "".to_string(), nonce : 0, transactions : Vec::new()};
        block.hash = block.hash();
        increment_index();
        return block;
    }

    pub fn hash(&self) -> String {
        match self.timestamp.elapsed() {
           Ok(elapsed) => {
               let hash_string = format!("{}{}{}{}", self.version, self.index, elapsed.as_secs(), self.previous_hash);
               let mut sha = Sha256::new();
               sha.input(hash_string);
               let result = sha.result();
               let result = result[..].to_hex();
               result
           }
           Err(e) => {
               panic!(e);
           }
        }
    }

    pub fn hash_block(version : u64, index : u64, timestamp : SystemTime, previous_hash : String, nonce : u64) -> String {
        match timestamp.elapsed() {
            Ok(elapsed) => {
                let hash_string = format!("{}{}{}{}", version, index, elapsed.as_secs(), previous_hash.to_string());
                let mut sha = Sha256::new();
                sha.input(hash_string);
                let result = sha.result();
                let result = result[..].to_hex();
                result
            }
            Err(e) => {
                panic!(e)
            }
        }
    }
}
