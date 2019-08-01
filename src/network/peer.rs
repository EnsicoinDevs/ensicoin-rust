use tokio::io::AsyncWrite;
use std::cmp::min;
use bincode::serialize;
use bincode::deserialize;
use super::message::*;
use tokio::net::TcpStream;
use tokio::io::{ AsyncWriteExt, AsyncReadExt };
use tokio::sync::mpsc;
use tokio::sync::lock::Lock;
use utils::Error;
use utils::Size;
use utils::ToBytes;

#[derive(PartialEq, Debug)]
enum State {
    Tcp,
    WhoAmI,
    Acknowledged
}

pub struct Peer {
    stream              : Lock<TcpStream>,
    server_sender       : mpsc::Sender<ServerMessage>,
    sender              : mpsc::Sender<ServerMessage>,
    // receiver            : mpsc::Receiver<ServerMessage>,
    connection_version  : Lock<u32>,
    initiated_by_us     : bool,
    connection_state    : Lock<State>,
} impl Peer {
    pub fn new(stream : TcpStream, server_sender : mpsc::Sender<ServerMessage>, initiated_by_us : bool) -> Peer {
        let (sender, receiver) = mpsc::channel(512);
        let stream = Lock::new(stream);
        let s2 = stream.clone();
        tokio::spawn(async move { Peer::handle_server_message(receiver, s2).await.unwrap(); });
        Peer {
            stream,
            server_sender,
            sender,
            // receiver,
            connection_version  : tokio::sync::lock::Lock::new(1),
            initiated_by_us,
            connection_state    : tokio::sync::lock::Lock::new(State::Tcp),
        }
    }

    pub async fn read_message(&mut self) -> Result<(), Error> {
        let (message_type, payload) = self.check_header().await?;
        let state = self.connection_state.lock().await;
        let mut stream = self.stream.lock().await;
        if *state != State::Acknowledged {
            self.handle_handshake(message_type, payload, state).await?;
        }
        else {
            drop(state);
            match message_type.as_str() {
                "2plus2is4\u{0}\u{0}\u{0}" => {
                    println!("2 plus 2 is 4!");
                    let message = Message::MinusOne;
                    Peer::send(message, &mut *stream).await?;
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
                        self.server_sender.send(ServerMessage::CheckTxs(self.sender.clone(), txs)).await?;
                    }
                    if !blocks.is_empty() {
                        self.server_sender.send(ServerMessage::CheckBlocks(self.sender.clone(), blocks)).await?;
                    }
                },
                "getdata\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                    let message = Message::GetData(Inv::read(&payload));
                    dbg!(&message);
                },
                "getblocks\u{0}\u{0}\u{0}" => {
                    println!("Received getblocks");
                    let message = GetBlocks::read(&payload);
                    self.server_sender.send(ServerMessage::GetBlocks(self.sender.clone(), message)).await?;
                },
                "transaction\u{0}" => {
                    let tx = blockchain::transaction::Transaction::read(&payload);
                    self.server_sender.send(ServerMessage::AddTx(tx)).await?;
                },
                "block\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                    let block = blockchain::block::Block::read(&payload)?;
                    self.server_sender.send(ServerMessage::AddBlock(block)).await?;
                },
                _ => { println!("didn't understand message type: {}", message_type); }
            }
        }
        Ok(())
    }

    pub async fn update(mut self) -> Result<(), Error> {
        //try to read incoming server message

        loop {
            match self.read_message().await {
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
        }
        self.close().await?;
        Ok(())
    }

    pub async fn connect(mut self) -> Result<(), Error> {
        let mut stream = self.stream.lock().await;
        let message = Message::WhoAmI(WhoAmI::default());
        Peer::send(message, &mut *stream).await?;
        self.update().await?;
        Ok(())
    }

    async fn close(&mut self) -> Result<(), Error>{
        self.server_sender.send(ServerMessage::DeletePeer(self.stream.lock().await.peer_addr().unwrap())).await.unwrap();
        self.stream.lock().await.shutdown(std::net::Shutdown::Both)?;
        Err(Error::ConnectionClosed)
    }

    fn prepare_header(message_type : &str, size : u64) -> Result<Vec<u8>, Error> {
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

    async fn send<T>(message : Message, stream: &mut T) -> Result<(), Error>
        where T: AsyncWrite + std::marker::Unpin {
        let mut buffer = Peer::prepare_header(message.name(), message.size())?;
        buffer.append(&mut message.send());
        stream.write_all(&buffer).await?;
        Ok(())
    }

    async fn read_header(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer : [u8; 24] = [0; 24];
        let mut stream = self.stream.lock().await;
        stream.read_exact(&mut buffer).await?;
        Ok(buffer.to_vec())
    }

    async fn read_payload(&mut self, length : usize) -> Result<Vec<u8>, Error> {
        if length != 0 {
            let mut buffer = vec![0; length];
            let mut stream = self.stream.lock().await;
            stream.read_exact(&mut buffer).await?;

            Ok(buffer)
        } else {
            Ok(vec![])
        }
    }
    async fn check_header(&mut self) -> Result<(String, Vec<u8>), Error> {
        let header = self.read_header().await?;
        let mut magic = header[0..4].to_vec();
        magic.reverse();
        let magic : u32 = deserialize(&magic)?;
        if magic != 42_2021 {
            println!("wrong magic number : {}", magic);
            self.close().await?;
        }
        let message_type : String = String::from_utf8(header[4..16].to_vec())?;
        let mut length = header[16..24].to_vec();
        length.reverse();
        let length : u64 = deserialize(&length)?;
        let payload = self.read_payload(length as usize).await?;
        Ok((message_type, payload))
    }

    async fn handle_handshake(&mut self, message_type: String, payload: Vec<u8>, mut state: tokio::sync::lock::LockGuard<State>) -> Result<(), Error> {
        let mut stream = self.stream.lock().await;
        match message_type.as_str() {
            "whoami\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                if *state == State::Tcp {
                    println!("Received message whoami");
                    let message = WhoAmI::read(payload);
                    let message_ver = message.version;
                    let mut conn_ver = self.connection_version.lock().await;
                    if !self.initiated_by_us {
                        // send WhoAmI
                        let message = Message::WhoAmI(WhoAmI::new(*conn_ver));
                        Peer::send(message, &mut *stream).await?;
                    }
                    Peer::send(Message::WhoAmIAck, &mut *stream).await?;
                    *conn_ver = min(message_ver, *conn_ver);
                    *state = State::WhoAmI;
                } else {
                    println!("received unusual number of whoami");
                    self.close().await?;
                }
            },
            "whoamiack\u{0}\u{0}\u{0}" => {
                if *state == State::WhoAmI {
                    *state = State::Acknowledged;
                    self.server_sender.send(ServerMessage::AddPeer(self.sender.clone(), stream.peer_addr().unwrap())).await?;
                    println!("Handshake completed");

                    let getblocks = Message::GetBlocks(GetBlocks::from_hashes(vec![blockchain::Block::genesis_block()?.hash_header()?], vec![0;32]));
                    Peer::send(getblocks, &mut *stream).await?;
                } else {
                    println!("reveiced whoamiack message before whoami message");
                    self.close().await?;
                }
            },
            _ => {
                println!("Recieved incorrect message_type : {:?}", message_type.as_str());
                self.close().await?;
            }
        }
        Ok(())
    }

    async fn handle_server_message(mut receiver: tokio::sync::mpsc::Receiver<ServerMessage>, mut stream: Lock<TcpStream>) -> Result<(), Error> {
        loop {
            match receiver.recv().await {
                Some(m)    => {
                    match m {
                        ServerMessage::CloseConnection  => {
                            receiver.close();
                            stream.lock().await.shutdown(std::net::Shutdown::Both)?;
                            break;
                        },
                        ServerMessage::AskTxs(hashes)   => {
                            let mut stream = stream.lock().await;
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
                            Peer::send(message, &mut *stream).await.unwrap();
                        },
                        ServerMessage::GetBlocksReply(hashs) => {
                            let mut stream = stream.lock().await;
                            let message = Message::Inv(Inv::from_vec(hashs));
                            Peer::send(message, &mut *stream).await.unwrap();
                        },
                        ServerMessage::AskBlocks(hashs) => {
                            let mut stream = stream.lock().await;
                            println!("Asking blocks");
                            let message = Message::GetData(Inv::from_vec(hashs));
                            Peer::send(message, &mut *stream).await.unwrap();
                        },
                        _   => ()
                    }
                },
                None  =>  break,
            }
        }
        Ok(())
    }
}
