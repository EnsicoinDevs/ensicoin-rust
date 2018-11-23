extern crate sha2;
mod blockchain;
mod model;
mod server;
use server::server::Server;
use blockchain::blockchain::Blockchain;
use model::block::Block;
use std::io::Read;
use std::io::Write;
use std::net;

fn main() {
    let mut server = Server::new();
    server.open();
    server.listen();
    let gen_block = Block::genesis_block();
    let mut blockchain = Blockchain::new(gen_block);
    unsafe {
        let block = Block::new(blockchain.get_latest_block());
        blockchain.add_block(block);
        let block = Block::new(blockchain.get_latest_block());
        blockchain.add_block(block);
    }
    println!("{}", Block::is_valid(blockchain.get_latest_block()));
}
