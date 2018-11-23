use blockchain::blockchain::Blockchain;
use model::block::Block;
use model::transaction::Transaction;
use std::net;
use std::thread;

pub struct Server {
    pub listener    : Option<net::TcpListener>,
    pub address     : String,
    pub peers       : Vec<net::TcpStream>,

    pub blockchain  : Blockchain,
    pub mempool     : Vec<Transaction>
}

impl Server {
    pub fn new() -> Server {
        let server = Server {
            listener    : None,
            address     : "127.0.0.1:4224".to_string(),
            peers       : Vec::new(),
            blockchain  : Blockchain::new(Block::genesis_block()),
            mempool     : Vec::new()
        };
        return server;
    }

    pub fn open(&self) {
        self.listener = Some(net::TcpListener::bind(self.address).unwrap());
    }

    pub fn listen(&self) {
        match self.listener {
            Some(listener) => {
                thread::spawn(|| for stream in listener.incoming() {
                    let mut stream = stream.unwrap();
                    self.peers.push(stream);
                });
            }
            None => panic!()
        }
    }
}
