#[macro_use]
extern crate clap;
extern crate sha2;
extern crate serde;
extern crate bincode;
extern crate protobuf;
extern crate grpc;
extern crate futures;
extern crate futures_cpupool;
extern crate sled;
extern crate dirs;
mod mempool;
mod utils;
mod blockchain;
mod model;
mod server;
mod rpc;
use std::error::Error;
use server::server::Server;
use utils::clp;


fn main() -> Result<(), Box<dyn Error>> {
    let cli = clp::initiate_cli();
    let mut server = Server::new(cli.port);
    server.interactive();
    server.listen();
    Ok(())
}
