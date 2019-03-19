use std::net::SocketAddr;
use std::net::IpAddr;
use std::net::TcpStream;
use std::net;
use std::thread;
use std::sync::Arc;
use std::sync::mpsc;
use std::collections::HashMap;
use mempool::mempool::Mempool;
use model::message;
use server::peer::Peer;
use rpc;
use rpc::discover_grpc::Discover;

pub struct Server {
    pub listener        : net::TcpListener,
    pub server_version  : Arc<u32>,
        peers           : HashMap<IpAddr, mpsc::Sender<message::ServerMessage>>,
        sender          : mpsc::Sender<message::ServerMessage>,
        receiver        : mpsc::Receiver<message::ServerMessage>,
        mempool         : Mempool,
}

impl Server {
    pub fn new(port: u16) -> Server {
        println!("Ensicoin started");
        let (tx, rx) = mpsc::channel();
        launch_discovery_server();
        Server {
            listener        : net::TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap(),
            server_version  : Arc::new(1),
            peers           : HashMap::new(),
            sender          : tx,
            receiver        : rx,
            mempool         : Mempool::new()
        }
    }

    pub fn listen(&mut self) {
        let listener = self.listener.try_clone().unwrap();
        let sender = self.sender.clone();
        thread::spawn(move || {
            for stream in listener.incoming() {
                println!("Incoming peer");
                let mut stream = stream.unwrap().try_clone().unwrap();
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
                        sender.send(message::ServerMessage::CreatePeer(ip.parse().unwrap())).unwrap();
                    },
                    "close\n" => {
                        println!("Enter a peer address to close: ");
                        std::io::stdin().read_line(&mut ip).unwrap();
                        ip = ip[..ip.len()-1].to_string();
                        sender.send(message::ServerMessage::ClosePeer(ip.parse().unwrap())).unwrap();
                    },
                    "exit\n" => {
                        sender.send(message::ServerMessage::CloseServer).unwrap();
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
                message::ServerMessage::CreatePeer(ip) => {
                    if !self.peers.contains_key(&ip) {
                        let sender = self.sender.clone();
                        let ip = *ip;
                        match TcpStream::connect(std::net::SocketAddr::new(ip, 4224)) {
                            Ok(tcp) => {
                                thread::Builder::new().name(ip.to_string()).spawn( move || {
                                Peer::new(tcp, sender, true).connect();
                            }).unwrap(); ()},
                            Err(e) => println!("{}", e),
                        }

                    } else {
                        println!("Already connected to peer");
                    }
                },
                message::ServerMessage::AddPeer(sender, ip) => {
                    self.peers.insert(*ip, sender.clone());
                },
                message::ServerMessage::DeletePeer(ip) => {
                    if self.peers.contains_key(&ip) {
                        self.peers.remove(&ip);
                    }
                    println!("peer deleted: {}", &ip);
                },
                message::ServerMessage::ClosePeer(ip) => {
                    if self.peers.contains_key(&ip) {
                        self.peers.get(&ip).unwrap().send(message::ServerMessage::CloseConnection).unwrap();
                    }
                },
                message::ServerMessage::CloseServer => {
                    for p in self.peers.values() {
                        p.send(message::ServerMessage::CloseConnection).unwrap();
                        drop(&self.listener);
                    }
                    panic!("Ensicoin stopped");
                },
                message::ServerMessage::CheckTxs(sender, hashes) => {
                    let mut inventory = Vec::new();
                    for hash in hashes {
                        if !self.mempool.contains_tx(hash.to_vec()) {
                            inventory.push(hash.to_vec());
                        }
                    }
                    sender.send(message::ServerMessage::AskTxs(inventory)).unwrap();
                },
                _ => ()
            }
        }
    }
}

struct DiscoverImpl;
impl Discover for DiscoverImpl {
    fn discover_peer(&self, _o: ::grpc::RequestOptions, p: rpc::discover::NewPeer) -> ::grpc::SingleResponse<rpc::discover::Ok> {
        let _ip : SocketAddr = p.get_address().parse().unwrap();
        //check known peer db
        println!("Received peer");
        ::grpc::SingleResponse::completed(rpc::discover::Ok::new())
    }
}

fn launch_discovery_server() {
    thread::spawn(move || {
        let mut server = grpc::ServerBuilder::new_plain();
        server.http.set_port(2442);
        server.add_service(rpc::discover_grpc::DiscoverServer::new_service_def(DiscoverImpl));
        let _server = server.build().expect("server");

        println!("discovery server started on port {}", 2442);

        loop {
            std::thread::park();
        }
    });
}
