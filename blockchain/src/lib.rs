pub mod block;
pub use block::Block;
pub mod transaction;

use dirs::data_dir;
use transaction::TxOut;
use sled::Db;
use utils::error::Error;


//////////////////////////////////////////////////////////////
//
//  Contains all databases concercing blocks and txs
//
//////////////////////////////////////////////////////////////

pub struct Blockchain;

impl Blockchain {
    fn open() -> Result<Db, Error> {
        let mut path = data_dir().unwrap();
        path.push("ensicoin-rust/");
        path.push("blockchain");
        Ok(sled::Db::start_default(path)?)
    }

    pub fn add_genesis_block() -> Result<(), Error> {
        let gen = Block::genesis_block()?;
        Blockchain::insert_block(gen.hash_header()?, &gen)?;
        Ok(())
    }

    pub fn get_block(hash: &[u8]) -> Result<Block, Error> {
        let db = Blockchain::open()?;
        let b = match db.get(hash)? {
            Some(b) => Block::read(&b)?,
            None => return Err(Error::DBError),
        };
        Ok(b)
    }

    pub fn get_blocks() -> Result<Vec<Block>, Error> {
        let db = Blockchain::open()?;
        let r : Vec<Block> = db.into_iter().map(|x| Block::read(&x.unwrap().1.to_vec()).unwrap()).collect();
        Ok(r)
    }

    pub fn has_block(hash: &[u8]) -> Result<bool, Error> {
        let db = Blockchain::open()?;
        match db.get(hash)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub fn insert_block(hash: Vec<u8>, block: &Block) -> Result<(), Error> {
        let db = Blockchain::open()?;
        db.set(hash, block.send()?)?;
        db.flush()?;
        NextHash::insert_next_hash(block.previous_hash.clone(), block.hash()?)?;
        Ok(())
    }

}

// key is a block hash, value is next block's hash
pub struct NextHash;
impl NextHash {
    fn open() -> Result<Db, Error> {
        let mut path = data_dir().unwrap();
        path.push("ensicoin-rust/");
        path.push("next_block");
        Ok(sled::Db::start_default(path)?)
    }

    pub fn get_next_hash(hash: &[u8]) -> Result<Vec<u8>, Error> {
        let db = NextHash::open()?;
        let h = match db.get(hash)? {
            Some(hash) => hash.to_vec(),
            None => return Err(Error::DBError),
        };
        Ok(h)
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
        let mut path = data_dir().unwrap();
        path.push("ensicoin-rust/");
        path.push("utxos");
        Ok(sled::Db::start_default(path)?)
    }

    pub fn get_utxos(tx_hash : Vec<u8>) -> Result<Vec<TxOut>, Error> {
        let db = Utxos::open()?;
        let v = match db.get(tx_hash)? {
            Some(v) => v,
            None => sled::IVec::from(&[]),
        };
        let utxos = (v).to_vec();
        if utxos.is_empty() {
            return Err(Error::DBError)
        }
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
        if utxo.is_some() {
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
