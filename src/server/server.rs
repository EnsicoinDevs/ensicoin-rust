use blockchain::blockchain::Blockchain;
use model::transaction::Transaction;
use std::net;

pub struct Server {
    pub listener : Option<net::TcpListener>,
    pub address  : String,
    pub peers    : Vec<net::TcpStream>,

    pub blockchain : Blockchain,
    pub mempool    : Vec<Transaction>
}

impl Server {
    pub fn new() -> Server {
        let server = Server {
            listener : None,
            address  : "127.0.0.1:4224",
            peers    : Vec::new(),
            
        }
    }
}
