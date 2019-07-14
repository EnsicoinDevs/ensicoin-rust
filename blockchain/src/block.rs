use bincode::{deserialize, serialize};
use model::*;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use super::transaction::*;
use utils::Error;
use utils::hash;
use utils::Size;

#[derive(Debug, Clone)]
pub struct Block {
    pub version: u32,
    pub flags: Vec<VarStr>,
    pub previous_hash: Vec<u8>,
    pub merkle_root: Vec<u8>,
    pub timestamp: u64,
    pub height: u32,
    pub difficulty: Vec<u8>,
    pub nonce: u64,
    pub transactions: Vec<Transaction>,
    pub hash: Vec<u8>,
}

impl Block {
    /**
     *  création du bloc génésis qui n'a pas de previous hash et a pour index 0
     **/
    pub fn genesis_block() -> Result<Block, Error> {

        let time = 1_558_540_052;
        let flag = VarStr::from_string("ici cest limag".to_string());
        let mut b: Block = Block {
            version: 0,
            flags: vec![flag],
            previous_hash: vec![0; 32],
            merkle_root: vec![0; 32],
            timestamp: time,
            height: 0,
            difficulty: vec![0,0,0,0,0,0,15,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            nonce: 42,
            transactions: Vec::new(),
            hash: Vec::new(),
        };
        b.hash = b.hash()?;
        Ok(b)
    }

    /**
     *  créer un nouveau bloc à l'aide du hash du bloc dernier bloc contenu dans la chaîne
     **/
    pub fn new(latest_block: &Block) -> Result<Block, Error> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(elapsed) => {
                let mut block = Block {
                    version: 0,
                    flags: Vec::new(),
                    previous_hash: latest_block.hash.clone(),
                    merkle_root: Vec::new(),
                    timestamp: elapsed.as_secs(),
                    height: 1,
                    difficulty: vec![0,0,0,0,0,0,15,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                    nonce: 0,
                    transactions: Vec::new(),
                    hash: Vec::new(),
                };
                block.hash = block.hash()?;
                Ok(block)
            },
            Err(e) => panic!(e),
        }
    }

    /**
     *  transforme les hash du tableau de transactions en chaîne de caractères
     **/
    fn hash_transactions(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();

        for tx in &self.transactions {
            buffer.append(&mut tx.hash()?);
        }

        Ok(buffer)
    }

    /**
     *  calcule le hash d'un bloc
     **/
    pub fn hash(&self) -> Result<Vec<u8>, Error> {
        let mut block = self.send_header()?;
        block.append(&mut self.hash_transactions()?);
        let result = hash::hash(block);
        Ok(hash::hash(result))
    }

    pub fn is_sane(&self) -> bool {
        if self.transactions.is_empty() {
            return false;
        }
        // if self.hash[0..(self.difficulty as usize)] != vec![0; self.difficulty as usize][..] {
        //     return false;
        // } NIQUE TA MERE.

        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(now) => {
                if self.timestamp >= (now.as_secs() + 7200) {
                    return false;
                }
            }
            Err(error) => {
                panic!(error);
            }
        }

        if self.merkle_root != ::utils::merkle_tree::compute_merkle_root(self.transactions.iter().map( |tx| tx.hash().unwrap() ).collect()) {
            return false;
        }

        true
    }

    pub fn is_valid(&self) -> bool {
        if !self.is_sane() {
            return false;
        }

        //valid txs
        let mut utxos : Vec<TxOut>;
        let mut tx_txo : super::transaction::TxTxo;
        for tx in &self.transactions[1..] {
            utxos = super::Utxos::get_utxos(tx.hash().unwrap()).unwrap();
            tx_txo = TxTxo::new(tx, utxos);
            if !tx_txo.is_valid() {
                return false
            }
        }
        true
    }

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

        buffer.append(&mut self.difficulty.clone());

        let mut nonce = serialize(&self.nonce)?;
        nonce.reverse();
        buffer.append(&mut nonce);

        Ok(buffer)
    }

    pub fn send_tx(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();

        let mut tx_count = serialize(&self.transactions.len())?;
        tx_count.reverse();
        buffer.append(&mut tx_count);

        for tx in &self.transactions {
            buffer.append(&mut tx.send()?);
        }

        Ok(buffer)
    }

    pub fn send(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = self.send_header()?;
        buffer.append(&mut self.send_tx()?);
        Ok(buffer)
    }

    pub fn read(buffer: &[u8]) -> Result<Block, Error> {
        let mut version = buffer[0..4].to_vec();
        version.reverse();
        let version: u32 = deserialize(&version)?;

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
        let timestamp = deserialize(&timestamp)?;
        offset += 8;

        let mut height = buffer[offset..offset + 4].to_vec();
        height.reverse();
        let height = deserialize(&height)?;
        offset += 4;

        let target = buffer[offset..offset + 32].to_vec();
        offset += 32;

        let mut nonce = buffer[offset..offset + 8].to_vec();
        nonce.reverse();
        let nonce = deserialize(&nonce)?;
        offset += 8;

        let tx_count = VarUint::new(&buffer[offset..].to_vec());
        offset += tx_count.size() as usize;

        let mut txs = Vec::new();
        for _ in 0..tx_count.value {
            let tx = Transaction::read(&buffer[offset..].to_vec());
            offset += tx.size() as usize;
            txs.push(tx);
        }

        let mut b = Block {
            version,
            flags,
            previous_hash: prev_block.to_vec(),
            merkle_root: merkle_root.to_vec(),
            timestamp,
            height,
            difficulty: target,
            nonce,
            transactions: txs,
            hash: Vec::new(),
        };
        b.hash = hash::hash(b.send_header()?);
        Ok(b)
    }
}

impl Size for Block {
    fn size(&self) -> u64 {
        1
    }
}
