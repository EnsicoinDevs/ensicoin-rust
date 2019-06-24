extern crate bincode;
extern crate blockchain;
extern crate dirs;
extern crate grpc;
extern crate mempool;
extern crate model;
extern crate rpc;
extern crate serde;
extern crate serde_json;
extern crate sled;
extern crate utils;

mod init;
mod server;
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
