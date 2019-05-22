use crate:: model::block::Block;
use crate::model::transaction::TxOut;
use crate::utils::error::Error;
use sled::Db;
use dirs::data_dir;

//////////////////////////////////////////////////////////////
//
//  Contains all databases concercing blocks and txs
//
//////////////////////////////////////////////////////////////

pub struct Blockchain;

impl Blockchain {
    fn open() -> Result<Db, Error> {
        let mut path = data_dir()?;
        path.push("ensicoin-rust/");
        path.push("blockchain");
        Ok(sled::Db::start_default(path)?)
    }

    pub fn get_block(&self, hash: &Vec<u8>) -> Result<Block, Error> {
        let db = Blockchain::open()?;
        Ok(Block::read(&(&*db.get(hash)??).to_vec())?)
    }

    pub fn get_blocks(&self) -> Result<Vec<Block>, Error> {
        let db = Blockchain::open()?;
        db.iter().map( |x| Block::read(&x.unwrap().0) ).collect()
    }

    pub fn insert_block(&self, hash: Vec<u8>, block: &Block) -> Result<(), Error> {
        let db = Blockchain::open()?;
        db.set(hash, block.send()?)?;
        db.flush()?;
        NextHash::insert_next_hash(block.previous_hash.clone(), block.hash()?)?;
        dbg!("hey");
        Ok(())
    }

}

// key is a block hash, value is next block's hash
pub struct NextHash;
impl NextHash {
    fn open() -> Result<Db, Error> {
        let mut path = data_dir()?;
        path.push("ensicoin-rust/");
        path.push("next_block");
        Ok(sled::Db::start_default(path)?)
    }

    pub fn get_next_hash(hash: &Vec<u8>) -> Result<Vec<u8>, Error> {
        let db = NextHash::open()?;
        Ok((&*db.get(hash)??).to_vec())
    }

    pub fn insert_next_hash(hash : Vec<u8>, next_hash: Vec<u8>) -> Result<(), Error> {
        let db = NextHash::open()?;
        db.set(hash, next_hash)?;
        db.flush()?;

        Ok(())
    }
}

//key is a tx hash, value is a vec of all outputs used as entry for this tx
pub struct Utxos;
impl Utxos {
    fn open() -> Result<Db, Error> {
        let mut path = data_dir()?;
        path.push("ensicoin-rust/");
        path.push("utxos");
        Ok(sled::Db::start_default(path)?)
    }

    pub fn get_utxos(tx_hash : Vec<u8>) -> Result<Vec<TxOut>, Error> {
        let db = Utxos::open()?;
        let utxos = (&*db.get(tx_hash)??).to_vec();
        let offset = 0;
        let mut result = Vec::new();
        while offset < utxos.len() {
            result.push(TxOut::read(&utxos[offset..].to_vec()));
        }

        Ok(result)
    }

    pub fn tx_exist(tx_hash : Vec<u8>) -> Result<bool, Error> {
        let db = Utxos::open()?;
        let utxo = db.get(tx_hash)?;
        if let Some(_) = utxo {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn insert_utxos(utxos : Vec<u8>, tx_hash : Vec<u8>) -> Result<(), Error>{
        let db = Utxos::open()?;
        db.set(tx_hash.clone(), utxos)?;
        db.flush()?;

        Ok(())
    }
}
