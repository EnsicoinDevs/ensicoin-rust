use utils::types::*;
use utils::hash;
use utils::error::Error;
use model::transaction;
use sha2::{Digest, Sha256};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use bincode::{serialize, deserialize};


#[derive(Debug)]
pub struct Block {
    pub version: u32,
    pub flags: Vec<String>,
    pub previous_hash: Vec<u8>,
    pub merkle_root: Vec<u8>,
    pub timestamp: u64,
    pub height: u32,
    pub difficulty: u64,
    pub nonce: u64,
    pub transactions: Vec<transaction::Transaction>,
    pub hash: Vec<u8>,
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
            flags: Vec::new(),
            previous_hash: vec![0; 32],
            merkle_root: Vec::new(),
            timestamp: time,
            height: 0,
            difficulty: 0,
            nonce: 0,
            transactions: Vec::new(),
            hash: Vec::new(),
        };
        b.hash = b.hash();
        b
    }

    /**
     *  créer un nouveau bloc à l'aide du hash du bloc dernier bloc contenu dans la chaîne
     **/
    pub fn new(latest_block: &Block) -> Block {
        let mut time = 0;
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(elapsed) => time = elapsed.as_secs(),
            Err(e) => panic!(e),
        }
        let mut block = Block {
            version: 0,
            flags: Vec::new(),
            previous_hash: latest_block.hash.clone(),
            merkle_root: Vec::new(),
            timestamp: time,
            height: 1,
            difficulty: 0,
            nonce: 0,
            transactions: Vec::new(),
            hash: Vec::new(),
        };
        block.hash = block.hash();
        block
    }

    /**
     *  transforme les hash du tableau de transactions en chaîne de caractères
     **/
    fn hash_transactions(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        for tx in &self.transactions {
            buffer.append(&mut tx.hash());
        }

        buffer
    }

    /**
     *  calcule le hash d'un bloc
     **/
    pub fn hash(&self) -> Vec<u8> {
        let block = self.send_header().unwrap();
        let result = hash::hash(block);
        hash::hash(result)
    }

    pub fn is_valid(block: &Block) -> bool {
        if block.transactions.len() == 0 {
            return false;
        }
        for (i, b) in block.hash.iter().enumerate() {
            if i < block.difficulty as usize {
                if *b != 0 {
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

    //TODO
    pub fn send_header(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        let mut version = serialize(&self.version)?;
        version.reverse();
        buffer.append(&mut version);

        let nb_flags = VarUint::from_u64(self.flags.len() as u64);
        buffer.append(&mut nb_flags.send());
        for s in &self.flags {
            buffer.append(&mut VarStr::from_string(s.to_string()).send());
        }

        buffer.append(&mut self.previous_hash.clone());
        buffer.append(&mut self.merkle_root.clone());

        let mut timestamp = serialize(&self.timestamp)?;
        timestamp.reverse();
        buffer.append(&mut timestamp);

        let mut height = serialize(&self.height)?;
        height.reverse();
        buffer.append(&mut height);

        let mut difficulty = serialize(&self.difficulty)?;
        difficulty.reverse();
        buffer.append(&mut difficulty);

        let mut nonce = serialize(&self.nonce)?;
        nonce.reverse();
        buffer.append(&mut nonce);

        Ok(buffer)
    }

    pub fn send_tx(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let mut tx_count = serialize(&self.transactions.len()).unwrap();
        tx_count.reverse();
        buffer.append(&mut tx_count);

        for tx in &self.transactions {
            buffer.append(&mut tx.send());
        }

        buffer
    }

    //TODO
    //pub fn read(buffer: Vec<u8>) -> Block {
    //
    //}
}
