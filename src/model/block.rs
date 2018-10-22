use std::time::UNIX_EPOCH;
use std::time::SystemTime;
use model::transaction;
use model::hash::ToHex;
use sha2::{Sha256, Digest};

static mut CURRENT_INDEX: u64 = 1;

unsafe fn increment_index() {
    CURRENT_INDEX += 1;
}

#[derive(Debug)]
pub struct Block {
    pub version : u64,
    pub index : u64,
    pub flags : Vec<String>,
    pub timestamp : SystemTime,
    pub hash : String,
    pub previous_hash : String,
    pub difficulty : u64,
    pub nonce : u64,
    pub transactions : Vec<transaction::Transaction>,
}

impl Block {

    /**
     *  création du bloc génésis qui n'a pas de previous hash et a pour index 0
     **/
    pub fn genesis_block() -> Block {
        let mut b : Block = Block {
            version         : 0,
            index           : 0,
            flags           : Vec::new(),
            timestamp       : SystemTime::now(),
            hash            : "".to_string(),
            previous_hash   : "".to_string(),
            difficulty      : 0,
            nonce           : 0,
            transactions    : Vec::new()
        };
        b.hash = b.hash();
        b
    }

    /**
     *  créer un nouveau bloc à l'aide du hash du bloc dernier bloc contenu dans la chaîne
     **/
    pub unsafe fn new(latest_block : &Block) -> Block {
        let mut block = Block{ version : 0, index : CURRENT_INDEX, flags : Vec::new(), timestamp : SystemTime::now(), hash : "".to_string(), previous_hash : latest_block.hash.clone().to_string(), difficulty: 0, nonce : 0, transactions : Vec::new()};
        block.hash = block.hash();
        increment_index();
        return block;
    }

    fn flags_string(&self) -> String {
        let mut string : String = String::new();
        for e in &self.flags {
            string += &e.clone();
        }
        string
    }

    /**
     *  calcule le hash d'un bloc
     **/
    pub fn hash(&self) -> String {
        match self.timestamp.duration_since(UNIX_EPOCH) {
           Ok(elapsed) => {
               let hash_string = format!("{}{}{}{}{}", self.version, self.index, self.flags_string(), elapsed.as_secs(), self.previous_hash);
               let mut sha = Sha256::new();
               sha.input(hash_string);
               let result = sha.result();
               let result = result[..].to_hex();
               sha = Sha256::new();
               sha.input(result);
               let result = sha.result();
               result[..].to_hex()
           }
           Err(e) => {
               panic!(e);
           }
        }
    }

}
