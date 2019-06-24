#![feature(try_trait)]
#[macro_use]
extern crate clap;
extern crate sha2;
extern crate serde;
extern crate serde_json;
extern crate bincode;
extern crate protobuf;
extern crate grpc;
extern crate futures;
extern crate futures_cpupool;
extern crate sled;
extern crate dirs;
// extern crate irc;
mod blockchain;
mod init;
// mod irc;
mod mempool;
mod model;
mod rpc;
mod server;
mod utils;
use std::error::Error;
use server::server::Server;
use utils::clp;


fn main() -> Result<(), Box<dyn Error>> {
    let cli = clp::initiate_cli();
    init::read_config()?;
    let mut server = Server::new(cli.port);
    server.interactive();
    server.listen();
    Ok(())
}
