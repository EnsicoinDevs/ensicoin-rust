use irc::client::prelude::*;
use dirs::data_dir;

pub fn init_irc_client() -> IrcClient {
    let mut path = data_dir().unwrap();
    path.push("ensicoin-rust/");
    path.push("irc.toml");
    let client = IrcClient::new(path).unwrap();
    client.identify().unwrap();
    client
}

pub fn discover_client(client: IrcClient) {
    let users = client.list_users("#ensicoin").unwrap();

}
