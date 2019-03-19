use model::block::Block;
use sled::Db;

pub struct Blockchain;

impl Blockchain {
    fn open() -> Db {
        sled::Db::start_default("blockchain").unwrap()
    }

    /*****************
     * CRUD METHODS  *
     *****************/
    //TODO
    pub fn get_block(&self, hash: String) -> Block {
        let db = Blockchain::open();
        Block::read((&*db.get(hash).unwrap().unwrap()).to_vec())
    }
}
