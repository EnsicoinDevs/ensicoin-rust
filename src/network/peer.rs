use bincode::serialize;
use bincode::deserialize;
use blockchain::Block;
use super::message::*;
use std::net::TcpStream;
use std::io::prelude::*;
use utils::Error;
use std::sync::mpsc;
use utils::Size;

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
        let header = self.read_header()?;
        let mut magic = header[0..4].to_vec();
        magic.reverse();
        let magic : u32 = deserialize(&magic)?;
        if magic != 42_2021 {
            println!("wrong magic number : {}", magic);
            self.close().unwrap();
        }
        let message_type : String = String::from_utf8(header[4..16].to_vec())?;
        let mut length = header[16..24].to_vec();
        length.reverse();
        let length : u64 = deserialize(&length)?;
        let payload = self.read_payload(length as usize)?;
        if self.connection_state != State::Acknowledged {
            match message_type.as_str() {
                "whoami\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                    if self.connection_state == State::Tcp {
                        println!("Received message whoami");
                        self.connection_version = WhoAmI::read(payload).handle(&self.stream, 1, self.initiated_by_us)?;
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

                        let getblocks = GetBlocks::from_hashes(vec![blockchain::Block::genesis_block()?.hash_header()?], vec![0;32]);
                        let mut buffer = self.prepare_header("getblocks\u{0}\u{0}\u{0}".to_string(), getblocks.size())?;
                        buffer.append(&mut getblocks.send());
                        self.stream.write_all(&buffer)?;

                        println!("Handshake completed");
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

        }
        else {
            dbg!(&message_type);
            match message_type.as_str() {
                "2plus2is4\u{0}\u{0}\u{0}" => {
                    println!("2 plus 2 is 4!");
                    let message = self.prepare_header("minus1thats3".to_string(), 0)?;
                    self.send(message);
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
                _ => { println!("didn't understand message type: {}", message_type); dbg!(&payload);}
            }
        }
        Ok(())
    }

    pub fn update(mut self) {
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
                            let message = Inv {
                                count: model::VarUint::from_u64(length as u64),
                                inventory
                            };
                            let mut buffer = self.prepare_header("getdata\u{0}\u{0}\u{0}\u{0}\u{0}".to_string(), message.size()).unwrap();
                            buffer.append(&mut message.send());
                            self.send(buffer);
                        },
                        ServerMessage::GetBlocksReply(hashs) => {
                            let inv = Inv::from_vec(hashs);
                            let mut message = self.prepare_header("inv\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}".to_string(), inv.size()).unwrap();
                            message.append(&mut inv.send());
                            self.send(message);
                        },
                        ServerMessage::AskBlocks(hashs) => {
                            let inv = Inv::from_vec(hashs);
                            let mut message = self.prepare_header("getdata\u{0}\u{0}\u{0}\u{0}\u{0}".to_string(), inv.size()).unwrap();
                            message.append(&mut inv.send());
                            self.send(message);
                        },
                        _                               => ()
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
        self.close().unwrap();
    }

    pub fn connect(self) {
        let message = WhoAmI::new();
        WhoAmI::send(message, &self.stream).unwrap();
        self.update();
        }

    fn close(&self) -> Result<(), Error>{
        self.server_sender.send(ServerMessage::DeletePeer(self.stream.peer_addr().unwrap())).unwrap();
        self.stream.shutdown(std::net::Shutdown::Both)?;
        Err(Error::ConnectionClosed)
    }

    fn prepare_header(&self, message_type : String, size : u64) -> Result<Vec<u8>, Error> {
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

    fn send(&mut self, message : Vec<u8>) {
        self.stream.write_all(&message).unwrap();
    }

    fn read_header(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer : [u8; 24] = [0; 24];
        self.stream.read_exact(&mut buffer)?;
        Ok(buffer.to_vec())
    }

    fn read_payload(&mut self, length : usize) -> Result<Vec<u8>, Error> {
        if length != 0 {
            let mut buffer = vec![0; length];
            self.stream.read_exact(&mut buffer)?;
            Ok(buffer)
        } else {
            Ok(vec![0])
        }
    }
}
