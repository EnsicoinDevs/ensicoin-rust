// use blockchain::blockchain::Blockchain;
// use model::block::Block;
use model::transaction::Transaction;
use std::net;
use std::io::prelude::*;

pub struct Server {
    pub listener    : net::TcpListener,
    pub peers       : Vec<net::TcpStream>,

    // pub blockchain  : Blockchain,
    pub mempool     : Vec<Transaction>
}

impl Server {
    pub fn new() -> Server {
        let server = Server {
            listener    : net::TcpListener::bind("127.0.0.1:4224").unwrap(),
            peers       : Vec::new(),
            // blockchain  : Blockchain::new(Block::genesis_block()),
            mempool     : Vec::new()
        };
        return server;
    }

    pub fn listen(&mut self) {
        for stream in self.listener.incoming() {
            // self.peers.push(stream.unwrap());
            let mut buffer = [0; 512];
            stream.unwrap().read(&mut buffer).unwrap();
            println!("request: {}", String::from_utf8_lossy(&buffer[..]));
        }
    }
}
