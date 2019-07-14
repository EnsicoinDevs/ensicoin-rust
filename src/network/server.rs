use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, mpsc};
use std::thread;

use blockchain::*;
use mempool::Mempool;
use super::message::*;
use rpc;
use rpc::discover_grpc::Discover;
use super::Peer;
use super::KnownPeers;

pub struct Server {
    pub listener        : TcpListener,
    pub server_version  : Arc<u32>,
        peers           : HashMap<SocketAddr, mpsc::Sender<ServerMessage>>,
        sender          : mpsc::Sender<ServerMessage>,
        receiver        : mpsc::Receiver<ServerMessage>,
        mempool         : Mempool,
}

impl Server {
    pub fn new(port: u16) -> Server {
        println!("Ensicoin started");
        let (tx, rx) = mpsc::channel();
        launch_discovery_server();
        Server {
            listener        : TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap(),
            server_version  : Arc::new(1),
            peers           : HashMap::new(),
            sender          : tx,
            receiver        : rx,
            mempool         : Mempool::new(),
        }
    }

    pub fn listen(&mut self) {
        let sender = self.sender.clone();
        thread::spawn(move || {
            peer_routine(sender);
        });

        let listener = self.listener.try_clone().unwrap();
        let sender = self.sender.clone();
        thread::spawn(move || {
            for stream in listener.incoming() {
                println!("Incoming peer");
                let stream = stream.unwrap().try_clone().unwrap();
                let sender2 = sender.clone();
                thread::Builder::new().name(stream.peer_addr().unwrap().to_string()).spawn(move || {
                    Peer::new(stream, sender2, false).update();
                }).unwrap();
            }
        });
        self.message_listener();
    }

    pub fn interactive(&self) {
        let sender = self.sender.clone();
        thread::spawn(move || {
            let mut command : String = "".into();
            let mut ip : String = "".into();
            loop {
                std::io::stdin().read_line(&mut command).unwrap();
                match command.as_ref() {
                    "connect\n" => {
                        println!("Enter a valid ip address: ");
                        std::io::stdin().read_line(&mut ip).unwrap();
                        ip = ip[..ip.len()-1].to_string();
                        sender.send(ServerMessage::CreatePeer(ip.parse().unwrap())).unwrap();
                    },
                    "close\n" => {
                        println!("Enter a peer address to close: ");
                        std::io::stdin().read_line(&mut ip).unwrap();
                        ip = ip[..ip.len()-1].to_string();
                        sender.send(ServerMessage::ClosePeer(ip.parse().unwrap())).unwrap();
                    },
                    "exit\n" => {
                        sender.send(ServerMessage::CloseServer).unwrap();
                    },
                    _ => ()
                }
                command = "".into();
                ip = "".into();
            }
        });
    }

    pub fn message_listener(&mut self) {
        loop {
            let message = self.receiver.recv().unwrap();
            match &message {
                ServerMessage::CreatePeer(ip) => {
                    if !self.peers.contains_key(&ip) {
                        let sender = self.sender.clone();
                        let ip = *ip;
                        match TcpStream::connect(ip) {
                            Ok(tcp) => {
                                thread::Builder::new().name(ip.to_string()).spawn( move || {
                                Peer::new(tcp, sender, true).connect();
                            }).unwrap(); },
                            Err(e) => println!("{}", e),
                        }

                    }
                },
                ServerMessage::AddPeer(sender, ip) => {
                    self.peers.insert(*ip, sender.clone());
                    match KnownPeers.add_peer((*ip).to_string()) {
                        Ok(_) => (),
                        Err(e) => { println!("Known Peers database probably dead: {:?}", e); }
                    }
                },
                ServerMessage::DeletePeer(ip) => {
                    if self.peers.contains_key(&ip) {
                        self.peers.remove(&ip);
                    }
                    match KnownPeers.del_peer((*ip).to_string()) {
                        Ok(_) => (),
                        Err(e) => { println!("Known Peers database probably dead: {:?}", e); }
                    }

                    println!("peer deleted: {}", &ip);
                },
                ServerMessage::ClosePeer(ip) => {
                    if self.peers.contains_key(&ip) {
                        self.peers.get(&ip).unwrap().send(ServerMessage::CloseConnection).unwrap();
                    }
                },
                ServerMessage::CloseServer => {
                    for p in self.peers.values() {
                        p.send(ServerMessage::CloseConnection).unwrap();
                    }
                    panic!("Ensicoin stopped");
                },
                ServerMessage::CheckTxs(sender, hashes) => {
                    let mut inventory = Vec::new();
                    for hash in hashes {
                        if !self.mempool.contains_tx(hash.to_vec()) {
                            inventory.push(hash.to_vec());
                        }
                    }
                    sender.send(ServerMessage::AskTxs(inventory)).unwrap();
                },
                ServerMessage::GetBlocks(sender, message) => {
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
                    // if hashs.len() > 0 {
                        sender.send(ServerMessage::GetBlocksReply(hashs)).unwrap();
                    // }
                    //maybe send entire blockchain otherwise?
                },
                ServerMessage::AddTx(tx) => {
                    self.mempool.add_tx(tx).unwrap();
                },
                ServerMessage::CheckBlocks(sender, hashs) => {
                    let mut inv = Vec::new();
                    for hash in hashs {
                        match Blockchain::get_block(hash) {
                            Ok(_) => (),
                            Err(_) => {
                                inv.push((hash.clone(), 1));
                            }
                        }
                    }
                    sender.send(ServerMessage::AskBlocks(inv)).unwrap();
                },
                ServerMessage::AddBlock(block) => {
                    Blockchain::insert_block(block.hash().unwrap(), block).unwrap();
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

fn peer_routine(sender: std::sync::mpsc::Sender<ServerMessage>) {
    let db = KnownPeers;
    loop {
        let peers = db.get_peers().unwrap();
        for p in peers {
            sender.send(ServerMessage::CreatePeer(p.parse().unwrap())).unwrap();
        }

        thread::sleep(std::time::Duration::from_secs(180));
    }
}
