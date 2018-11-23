use model::hash::ToHex;
use model::transaction;
use sha2::{Digest, Sha256};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

static mut CURRENT_INDEX: u64 = 1;

unsafe fn increment_index() {
    CURRENT_INDEX += 1;
}

#[derive(Debug)]
pub struct Block {
    pub version: u64,
    pub index: u64,
    pub flags: Vec<String>,
    pub timestamp: u64,
    pub hash: String,
    pub previous_hash: String,
    pub difficulty: u64,
    pub nonce: u64,
    pub transactions: Vec<transaction::Transaction>,
}

impl Block {
    /**
     *  création du bloc génésis qui n'a pas de previous hash et a pour index 0
     **/
    pub fn genesis_block() -> Block {
        let mut time = 0;
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(elapsed) => {
                time = elapsed.as_secs();
            }
            Err(e) => {
                panic!(e);
            }
        }
        let mut b: Block = Block {
            version: 0,
            index: 0,
            flags: Vec::new(),
            timestamp: time,
            hash: "".to_string(),
            previous_hash: "".to_string(),
            difficulty: 0,
            nonce: 0,
            transactions: Vec::new(),
        };
        b.hash = b.hash();
        b
    }

    /**
     *  créer un nouveau bloc à l'aide du hash du bloc dernier bloc contenu dans la chaîne
     **/
    pub unsafe fn new(latest_block: &Block) -> Block {
        let mut time = 0;
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(elapsed) => time = elapsed.as_secs(),
            Err(e) => panic!(e),
        }
        let mut block = Block {
            version: 0,
            index: CURRENT_INDEX,
            flags: Vec::new(),
            timestamp: time,
            hash: "".to_string(),
            previous_hash: latest_block.hash.clone().to_string(),
            difficulty: 0,
            nonce: 0,
            transactions: Vec::new(),
        };
        block.hash = block.hash();
        increment_index();
        return block;
    }

    /**
     *  transforme le tableau de flags en chaîne de caractères
     **/
    fn flags_string(&self) -> String {
        let mut string: String = String::new();
        for e in &self.flags {
            string += &e.clone();
        }
        string
    }

    /**
     *  transforme les hash du tableau de transactions en chaîne de caractères
     **/
    fn hash_transactions(&self) -> String {
        let mut string = String::new();
        for e in &self.transactions {
            string += &e.hash().clone();
        }
        string
    }

    /**
     *  calcule le hash d'un bloc
     **/
    pub fn hash(&self) -> String {
        let hash_string = format!(
            "{}{}{}{}{}{}",
            self.version,
            self.index,
            self.flags_string(),
            self.timestamp,
            self.previous_hash,
            self.hash_transactions()
        );
        let mut sha = Sha256::new();
        sha.input(hash_string);
        let result = sha.result();
        let result = result[..].to_hex();
        sha = Sha256::new();
        sha.input(result);
        let result = sha.result();
        result[..].to_hex()
    }

    pub fn is_valid(block: &Block) -> bool {
        if block.transactions.len() == 0 {
            return false;
        }
        for (i, c) in block.hash.chars().enumerate() {
            if i < block.difficulty as usize {
                if c != '0' {
                    return false;
                }
            }
        }

        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(now) => {
                if block.timestamp >= (now.as_secs() + 7200) {
                    return false;
                }
            }
            Err(error) => {
                panic!(error);
            }
        }

        //first transaction is coinbase
        //validate each transaction

        true
    }
}
