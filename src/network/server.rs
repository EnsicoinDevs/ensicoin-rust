use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use tokio::net::{ TcpListener, TcpStream };
use tokio::sync::mpsc;

use blockchain::*;
use mempool::Mempool;
use super::message::*;
use rpc;
use rpc::discover_grpc::Discover;
use super::Peer;
use super::KnownPeers;

pub struct Server {
    pub server_version  : Arc<u32>,
        peers           : HashMap<SocketAddr, mpsc::Sender<ServerMessage>>,
        sender          : mpsc::Sender<ServerMessage>,
        receiver        : mpsc::Receiver<ServerMessage>,
        mempool         : Mempool,
}

impl Server {
    pub fn new() -> Server {
        // dbg!(format!("{:?}", Blockchain::get_blocks().unwrap()));
        println!("Ensicoin started");
        let (tx, rx) = mpsc::channel(512);
        launch_discovery_server();
        Server {
            server_version  : Arc::new(1),
            peers           : HashMap::new(),
            sender          : tx,
            receiver        : rx,
            mempool         : Mempool::new(),
        }
    }

    pub async fn listen(self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            peer_routine(sender).await;
        });

        let mut listener = TcpListener::bind(&SocketAddr::new("0.0.0.0".parse().unwrap(), port)).unwrap();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                println!("Incoming peer");
                let sender2 = sender.clone();
                tokio::spawn(async move {
                    Peer::new(stream, sender2, false).update().await.unwrap();
                });
            }
        });
        self.message_listener().await;
        Ok(())
    }

    pub async fn interactive(&self) {
        let mut sender = self.sender.clone();
        tokio::spawn(async move {
            let mut command : String = "".into();
            let mut ip : String = "".into();
            loop {
                std::io::stdin().read_line(&mut command).unwrap();
                match command.as_ref() {
                    "connect\n" => {
                        println!("Enter a valid ip address: ");
                        std::io::stdin().read_line(&mut ip).unwrap();
                        ip = ip[..ip.len()-1].to_string();
                        sender.send(ServerMessage::CreatePeer(ip.parse().unwrap())).await.unwrap();
                    },
                    "close\n" => {
                        println!("Enter a peer address to close: ");
                        std::io::stdin().read_line(&mut ip).unwrap();
                        ip = ip[..ip.len()-1].to_string();
                        sender.send(ServerMessage::ClosePeer(ip.parse().unwrap())).await.unwrap();
                    },
                    "exit\n" => {
                        sender.send(ServerMessage::CloseServer).await.unwrap();
                    },
                    _ => ()
                }
                command = "".into();
                ip = "".into();
            }
        });
    }

    pub async fn message_listener(mut self) {
        loop {
            let message = self.receiver.recv().await.unwrap();
            match message {
                ServerMessage::CreatePeer(ip) => {
                    if !self.peers.contains_key(&ip) {
                        let sender = self.sender.clone();
                        let ip = ip;
                        match TcpStream::connect(&ip).await {
                            Ok(tcp) => {
                                tokio::spawn(async move {
                                Peer::new(tcp, sender, true).connect().await.unwrap();
                            });
                        },
                            Err(e) => println!("{}", e),
                        }

                    }
                },
                ServerMessage::AddPeer(sender, ip) => {
                    self.peers.insert(ip, sender.clone());
                    match KnownPeers.add_peer((ip).to_string()) {
                        Ok(_) => (),
                        Err(e) => { println!("Known Peers database probably dead: {:?}", e); }
                    }
                },
                ServerMessage::DeletePeer(ip) => {
                    if self.peers.contains_key(&ip) {
                        self.peers.remove(&ip);
                    }
                    match KnownPeers.del_peer((ip).to_string()) {
                        Ok(_) => (),
                        Err(e) => { println!("Known Peers database probably dead: {:?}", e); }
                    }

                    println!("peer deleted: {}", &ip);
                },
                ServerMessage::ClosePeer(ip) => {
                    if self.peers.contains_key(&ip) {
                        self.peers.get_mut(&ip).unwrap().send(ServerMessage::CloseConnection).await.unwrap();
                    }
                },
                ServerMessage::CloseServer => {
                    for p in self.peers.values_mut() {
                        p.send(ServerMessage::CloseConnection).await.unwrap();
                    }
                    panic!("Ensicoin stopped");
                },
                ServerMessage::CheckTxs(mut sender, hashes) => {
                    let mut inventory = Vec::new();
                    for hash in hashes {
                        if !self.mempool.contains_tx(hash.to_vec()) {
                            inventory.push(hash.to_vec());
                        }
                    }
                    sender.send(ServerMessage::AskTxs(inventory)).await.unwrap();
                },
                ServerMessage::GetBlocks(mut sender, message) => {
                    let mut hashs = Vec::new();
                    for hash in &message.block_locator {
                        if let Ok(b) = Blockchain::get_block(&hash) {
                            let mut hash = b.hash().unwrap();
                            while let Ok(h) = NextHash::get_next_hash(&hash) {
                                hashs.push((h.clone(), 1));
                                if h == message.hash_stop {
                                    break;
                                }
                                hash = h;
                            }
                            break;
                        }
                    }
                    match sender.send(ServerMessage::GetBlocksReply(hashs)).await {
                        Ok(_) => (),
                        Err(e) => println!("could not send message: {:?}", e),
                    }
                },
                ServerMessage::AddTx(tx) => {
                    self.mempool.add_tx(&tx).unwrap();
                },
                ServerMessage::CheckBlocks(mut sender, hashs) => {
                    let mut inv;
                    for hash in hashs {
                        inv = Vec::new();
                        match Blockchain::has_block(&hash) {
                            Ok(true) => (),
                            Ok(false) => {
                                inv.push((hash.clone(), 1));
                                sender.send(ServerMessage::AskBlocks(inv)).await.unwrap();
                            },
                            Err(e) => println!("Something went wrong: {:?}", e),
                        }
                    }
                },
                ServerMessage::AddBlock(block) => {
                    Blockchain::insert_block(block.hash().unwrap(), &block).unwrap();
                },
                _ => ()
            }
        }
    }

}

struct DiscoverImpl {
    peers: KnownPeers
}
impl Discover for DiscoverImpl {
    fn discover_peer(&self, _o: grpc::RequestOptions, p: rpc::discover::NewPeer) -> grpc::SingleResponse<rpc::discover::Ok> {
        let ip : SocketAddr = p.get_address().parse().unwrap();
        //check known peer db
        println!("Received peer");
        match self.peers.add_peer(ip.to_string()) {
            Ok(_) => (),
            Err(_) => { println!("failed to add peer"); }
        }
        grpc::SingleResponse::completed(rpc::discover::Ok::new())
    }
}

fn launch_discovery_server() {
    thread::spawn(move || {
        let mut server = grpc::ServerBuilder::new_plain();
        server.http.set_port(2442);
        server.add_service(rpc::discover_grpc::DiscoverServer::new_service_def(DiscoverImpl{peers: KnownPeers}));
        let _server = server.build().expect("server");

        println!("discovery server started on port 2442");

        loop {
            std::thread::park();
        }
    });
}

async fn peer_routine(mut sender: tokio::sync::mpsc::Sender<ServerMessage>) {
    let db = KnownPeers;
    loop {
        let peers = db.get_peers().unwrap();
        for p in peers {
            sender.send(ServerMessage::CreatePeer(p.parse().unwrap())).await.unwrap();
        }

        thread::sleep(std::time::Duration::from_secs(180));
    }
}
