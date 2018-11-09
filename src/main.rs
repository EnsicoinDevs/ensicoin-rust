extern crate sha2;
mod model;
mod blockchain;
use std::io::Write;
use std::io::Read;
use model::block::Block as Block;
use blockchain::blockchain::Blockchain as Blockchain;
use std::net;
use std::thread;


fn main() {
    let tcp = net::TcpListener::bind("127.0.0.1:4242").unwrap();

    for stream in tcp.incoming() {
        let mut stream = stream.unwrap();
        stream.write(b"hello boys").unwrap();
    }

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
