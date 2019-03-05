use sled::Db;
use model::block::Block;

pub struct Blockchain;

impl Blockchain {
    fn open() -> Db {
        sled::Db::start_default("blockchain").unwrap()
    }

    /*****************
    * CRUD METHODS  *
    *****************/
    //TODO
    // pub fn get_block(&self, hash: String) -> Block {
    //     let db = Blockchain::open();
    //     &*db.get(hash).unwrap().unwrap()
    // }
}
