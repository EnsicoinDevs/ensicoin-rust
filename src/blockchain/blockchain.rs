use model::block::Block;
use utils::error::Error;
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
        path.push("blockchain.db");
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
        path.push("next_block.db");
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

pub struct Utxos;
