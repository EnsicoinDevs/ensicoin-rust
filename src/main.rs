extern crate sha2;
// extern crate serde_derive;
extern crate bincode;
mod blockchain;
mod model;
mod server;
use server::server::Server;
// use blockchain::blockchain::Blockchain;
// use model::block::Block;
// use std::io::Read;
// use std::io::Write;
// use std::net;

fn main() {
    let mut server = Server::new();
    server.listen();

}
