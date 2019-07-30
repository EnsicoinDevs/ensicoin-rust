use std::cmp::min;
use bincode::serialize;
use bincode::deserialize;
use super::message::*;
use std::net::TcpStream;
use std::io::prelude::*;
use utils::Error;
use std::sync::mpsc;
use utils::Size;
use utils::ToBytes;

#[derive(PartialEq, Debug)]
enum State {
    Tcp,
    WhoAmI,
    Acknowledged
}

pub struct Peer {
    stream              : TcpStream,
    server_sender       : mpsc::Sender<ServerMessage>,
    sender              : mpsc::Sender<ServerMessage>,
    receiver            : mpsc::Receiver<ServerMessage>,
    connection_version  : u32,
    initiated_by_us     : bool,
    connection_state    : State,
} impl Peer {
    pub fn new(s : TcpStream, server_sender : mpsc::Sender<ServerMessage>, initiated_by_us : bool) -> Peer {
        let (tx, rx) = mpsc::channel();
        s.set_nonblocking(true).unwrap();
        Peer {
            stream              : s,
            server_sender,
            sender              : tx,
            receiver            : rx,
            connection_version  : 1,
            initiated_by_us,
            connection_state    : State::Tcp,
        }
    }

    pub fn read_message(&mut self) -> Result<(), Error> {
        let (message_type, payload) = self.check_header()?;

        if self.connection_state != State::Acknowledged {
            self.handle_handshake(message_type, payload)?;
        }
        else {
            match message_type.as_str() {
                "2plus2is4\u{0}\u{0}\u{0}" => {
                    println!("2 plus 2 is 4!");
                    let message = Message::MinusOne;
                    self.send(message)?;
                },
                "inv\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                    let message = Inv::read(&payload);
                    println!("Received Inv message with {} items", &message.count.value);
                    let mut txs = Vec::new();
                    let mut blocks = Vec::new();
                    for item in message.inventory {
                        if item.hash_type == 0 { //TX
                            txs.push(item.hash);
                        } else {
                            blocks.push(item.hash);
                        }
                    }
                    if !txs.is_empty() {
                        self.server_sender.send(ServerMessage::CheckTxs(self.sender.clone(), txs))?;
                    }
                    if !blocks.is_empty() {
                        self.server_sender.send(ServerMessage::CheckBlocks(self.sender.clone(), blocks))?;
                    }
                },
                "getdata\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                    let message = Message::GetData(Inv::read(&payload));
                    dbg!(&message);
                },
                "getblocks\u{0}\u{0}\u{0}" => {
                    println!("Received getblocks");
                    let message = GetBlocks::read(&payload);
                    self.server_sender.send(ServerMessage::GetBlocks(self.sender.clone(), message))?;
                },
                "transaction\u{0}" => {
                    let tx = blockchain::transaction::Transaction::read(&payload);
                    self.server_sender.send(ServerMessage::AddTx(tx))?;
                },
                "block\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                    let block = blockchain::block::Block::read(&payload)?;
                    self.server_sender.send(ServerMessage::AddBlock(block))?;
                },
                _ => { println!("didn't understand message type: {}", message_type); }
            }
        }
        Ok(())
    }

    pub fn update(mut self) -> Result<(), Error>{
        loop {
            //try to read incoming server message
            match self.receiver.try_recv() {
                Ok(m)    => {
                    match m {
                        ServerMessage::CloseConnection  => { self.close().unwrap(); },
                        ServerMessage::AskTxs(hashes)   => {
                            //construct invvect and send it
                            let mut inventory = Vec::new();
                            let length = hashes.len();
                            for hash in hashes {
                                inventory.push(model::InvVect::from_vec(hash, 0));
                            }
                            let message = Message::Inv(Inv {
                                count: model::VarUint::from_u64(length as u64),
                                inventory
                            });
                            self.send(message)?;
                        },
                        ServerMessage::GetBlocksReply(hashs) => {
                            let message = Message::Inv(Inv::from_vec(hashs));
                            self.send(message)?;
                        },
                        ServerMessage::AskBlocks(hashs) => {
                            println!("Asking blocks");
                            let message = Message::GetData(Inv::from_vec(hashs));
                            self.send(message)?;
                        },
                        _   => ()
                    }
                },
                Err(e)  =>  if let std::sync::mpsc::TryRecvError::Disconnected = e {
                                break
                            }
            }
            match self.read_message() {
                Ok(_) => (),
                Err(e) => { match e {
                                Error::IOError(e) => {
                                    if e.kind() == std::io::ErrorKind::WouldBlock {
                                        // println!("nothing to read");
                                    } else {
                                        println!("{:?}", e); break;
                                    }
                                },
                                _ => { println!("{:?}", e); break; }
                            }
                        }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        self.close()?;
        Ok(())
    }

    pub fn connect(mut self) -> Result<(), Error> {
        let message = Message::WhoAmI(WhoAmI::default());
        self.send(message)?;
        self.update()?;
        Ok(())
    }

    fn close(&self) -> Result<(), Error>{
        self.server_sender.send(ServerMessage::DeletePeer(self.stream.peer_addr().unwrap())).unwrap();
        self.stream.shutdown(std::net::Shutdown::Both)?;
        Err(Error::ConnectionClosed)
    }

    fn prepare_header(&self, message_type : &str, size : u64) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        let magic : u32 = 42_2021; ////////////////// magic number
        let mut magic = serialize(&magic)?;
        magic.reverse();
        buffer.append(&mut magic);
        buffer.append(&mut message_type.as_bytes().to_vec());
        if message_type.len() < 12 {
            buffer.append(&mut vec![0; 12-message_type.len()]);
        }
        let mut size = serialize(&size)?;
        size.reverse();
        buffer.append(&mut size);
        Ok(buffer)
    }

    fn send(&mut self, message : Message) -> Result<(), Error> {
        let mut buffer = self.prepare_header(message.name(), message.size())?;
        buffer.append(&mut message.send());
        self.stream.write_all(&buffer)?;
        Ok(())
    }

    fn read_header(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer : [u8; 24] = [0; 24];
        self.stream.read_exact(&mut buffer)?;
        Ok(buffer.to_vec())
    }

    fn read_payload(&mut self, length : usize) -> Result<Vec<u8>, Error> {
        if length != 0 {
            let mut buffer = vec![0; length];
            self.stream.set_nonblocking(false).unwrap();
            self.stream.read_exact(&mut buffer)?;
            self.stream.set_nonblocking(true).unwrap();

            Ok(buffer)
        } else {
            Ok(vec![])
        }
    }
    fn check_header(&mut self) -> Result<(String, Vec<u8>), Error> {
        let header = self.read_header()?;
        let mut magic = header[0..4].to_vec();
        magic.reverse();
        let magic : u32 = deserialize(&magic)?;
        if magic != 42_2021 {
            println!("wrong magic number : {}", magic);
            self.close()?;
        }
        let message_type : String = String::from_utf8(header[4..16].to_vec())?;
        let mut length = header[16..24].to_vec();
        length.reverse();
        let length : u64 = deserialize(&length)?;
        let payload = self.read_payload(length as usize)?;
        Ok((message_type, payload))
    }

    fn handle_handshake(&mut self, message_type: String, payload: Vec<u8>) -> Result<(), Error> {
        match message_type.as_str() {
            "whoami\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                if self.connection_state == State::Tcp {
                    println!("Received message whoami");
                    let message = WhoAmI::read(payload);
                    let conn_ver = message.version;
                    if !self.initiated_by_us {
                        // send WhoAmI
                        let message = Message::WhoAmI(WhoAmI::new(self.connection_version));
                        self.send(message)?;
                    }
                    self.send(Message::WhoAmIAck)?;
                    self.connection_version = min(conn_ver, self.connection_version);
                    self.connection_state = State::WhoAmI;
                } else {
                    println!("received unusual number of whoami");
                    self.close()?;
                }
            },
            "whoamiack\u{0}\u{0}\u{0}" => {
                if self.connection_state == State::WhoAmI {
                    self.connection_state = State::Acknowledged;
                    self.server_sender.send(ServerMessage::AddPeer(self.sender.clone(), self.stream.peer_addr().unwrap()))?;
                    println!("Handshake completed");

                    let getblocks = Message::GetBlocks(GetBlocks::from_hashes(vec![blockchain::Block::genesis_block()?.hash_header()?], vec![0;32]));
                    self.send(getblocks)?;
                } else {
                    println!("reveiced whoamiack message before whoami message");
                    self.close()?;
                }
            },
            _ => {
                println!("Recieved incorrect message_type : {:?}", message_type.as_str());
                self.close()?;
            }
        }
        Ok(())
    }
}
