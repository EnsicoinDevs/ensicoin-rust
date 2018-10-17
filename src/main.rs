mod model;
mod blockchain;
use model::block::Block as Block;
use blockchain::blockchain::Blockchain as Blockchain;
use std::time::UNIX_EPOCH;
extern crate sha2;

fn main() {
    let gen_block = Block::genesis_block();
    let mut blockchain = Blockchain::new(gen_block);
    unsafe {
        let block = Block::new(blockchain.get_latest_block());
        blockchain.add_block(block);
        let block = Block::new(blockchain.get_latest_block());
        blockchain.add_block(block);
    }
    println!("{}", blockchain.get_latest_block().hash);
    match blockchain.get_latest_block().timestamp.duration_since(UNIX_EPOCH) {
        Ok(elapsed) => println!("{}", elapsed.as_secs()),
        Err(e) => panic!(e)
    }
}
