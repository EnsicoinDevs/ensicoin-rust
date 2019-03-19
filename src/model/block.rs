use bincode::{deserialize, serialize};
use model::message::Size;
use model::transaction;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use utils::error::Error;
use utils::hash;
use utils::types::*;

#[derive(Debug)]
pub struct Block {
    pub version: u32,
    pub flags: Vec<VarStr>,
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
            buffer.append(&mut s.send());
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

    pub fn read(mut buffer: Vec<u8>) -> Block {
        let mut version = buffer[0..4].to_vec();
        version.reverse();
        let version: u32 = deserialize(&version).unwrap();

        let flags_count = VarUint::new(&buffer[4..].to_vec());
        let mut flags = Vec::new();
        let mut offset: usize = 4 + flags_count.size() as usize;
        for _ in 0..flags_count.value {
            let s = VarStr::new(&buffer[offset..].to_vec());
            offset += s.size() as usize;
            flags.push(s);
        }

        let prev_block = buffer[offset..offset + 32].to_vec();
        offset += 32;

        let merkle_root = buffer[offset..offset + 32].to_vec();
        offset += 32;

        let mut timestamp = buffer[offset..offset + 8].to_vec();
        timestamp.reverse();
        let timestamp = deserialize(&timestamp).unwrap();
        offset += 8;

        let mut height = buffer[offset..offset + 4].to_vec();
        height.reverse();
        let height = deserialize(&height).unwrap();
        offset += 4;

        let mut target = buffer[offset..offset + 4].to_vec();
        target.reverse();
        let target = deserialize(&target).unwrap();
        offset += 4;

        let mut nonce = buffer[offset..offset + 8].to_vec();
        nonce.reverse();
        let nonce = deserialize(&nonce).unwrap();
        offset += 8;

        let tx_count = VarUint::new(&buffer[offset..].to_vec());
        offset += tx_count.size() as usize;

        let mut txs = Vec::new();
        for _ in 0..tx_count.value {
            let tx = transaction::Transaction::read(&buffer[offset..].to_vec());
            offset += tx.size() as usize;
            txs.push(tx);
        }

        let mut b = Block {
            version: version,
            flags: flags,
            previous_hash: prev_block.to_vec(),
            merkle_root: merkle_root.to_vec(),
            timestamp: timestamp,
            height: height,
            difficulty: target,
            nonce: nonce,
            transactions: txs,
            hash: Vec::new(),
        };
        b.hash = hash::hash(b.send_header().unwrap());
        b
    }
}
