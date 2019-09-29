pub mod init;
mod network;
use std::error::Error;
use network::server::Server;
use utils::clp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = clp::args();
    init::read_config()?;
    tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber
        ::builder()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .inherit_fields(true)
        .finish()).unwrap();
    let server = Server::new();
    server.interactive().await;
    server.listen(args.port).await?;
    Ok(())
}
