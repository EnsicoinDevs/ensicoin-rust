pub mod init;
mod network;
use std::error::Error;
use network::server::Server;
use utils::clp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = clp::args();
    init::read_config()?;
    let server = Server::new();
    server.interactive().await;
    server.listen(args.port).await?;
    Ok(())
}
