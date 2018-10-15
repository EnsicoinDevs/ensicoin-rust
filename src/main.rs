mod model;
use model::block::Block as Block;
use model::hash::Hash;
extern crate sha2;

fn main() {
    let hachich = Hash { val: Vec::new() };
    let hachich2 = Hash { val: Vec::new() };
    let mut block : Block = Block {
        version: 1,
        index: 42,
        timestamp: 123456789,
        hash: hachich2,
        previous_hash: hachich,
        nonce: 0,
        transactions: Vec::new()
    };
    block.hash = block.hash();
    println!("{:?}", block);
}
