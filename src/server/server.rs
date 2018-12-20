// use blockchain::blockchain::Blockchain;
// use model::block::Block;
use model::transaction::Transaction;
use model::message::Message;
use std::str;
use std::net;
use std::io::prelude::*;

pub struct Server {
    pub listener    : net::TcpListener,
    pub peers       : Vec<net::TcpStream>,
    pub max_peers   : u64,
    // pub blockchain  : Blockchain,
    pub mempool     : Vec<Transaction>
}

impl Server {
    pub fn new() -> Server {
        let server = Server {
            listener    : net::TcpListener::bind("127.0.0.1:4224").unwrap(),
            peers       : Vec::new(),
            max_peers   : 42,
            // blockchain  : Blockchain::new(Block::genesis_block()),
            mempool     : Vec::new()
        };
        return server;
    }

    pub fn listen(&mut self) {
        for stream in self.listener.incoming() {
            std::thread::spawn(|| {
                //read message
                let stream = stream.unwrap();
                let mut magic : [u8; 4];
                let mut message_type : [u8; 12];
                let mut length : [u8; 8];

                stream.read_exact(&mut magic).unwrap();
                stream.read(&mut message_type).unwrap();
                stream.read(&mut length).unwrap();

                // let size = length.iter().fold(0, |x, &i| x << 8 | i as u64);
                let message_type = str::from_utf8(&message_type).unwrap();
                match message_type {
                    "whoami" => {Message::Who_am_i::new(&stream)},
                    "whoamiack" => {},
                    "getaddr" => {},
                    "addr" => {},
                    _ => ()
                }
                // let mut payload : [u8; size];
            });
        }
    }
}
