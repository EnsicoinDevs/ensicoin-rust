mod model;
use model::block::Block as Block;
use std::time::SystemTime;
extern crate sha2;

fn main() {
    let mut block : Block = Block {
        version: 1,
        index: 42,
        timestamp: SystemTime::now(),
        hash: " ".to_string(),
        previous_hash: "".to_string(),
        nonce: 0,
        transactions: Vec::new()
    };
    block.hash = block.hash();
    println!("{}", block.hash);
}
