use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use tokio::io::AsyncWrite;
use std::cmp::min;
use bincode::serialize;
use bincode::deserialize;
use super::message::*;
use tokio::net::TcpStream;
use tokio::io::{ AsyncWriteExt, AsyncReadExt };
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::{ debug, error, info, span, warn };
use utils::Error;
use utils::Size;
use utils::ToBytes;

#[derive(PartialEq, Debug)]
enum State {
    Tcp,
    WhoAmI,
    Acknowledged
}

type Locked<T> = Arc<Mutex<T>>;

pub struct Peer {
    stream              : Locked<TcpStream>,
    server_sender       : mpsc::Sender<ServerMessage>,
    sender              : mpsc::Sender<ServerMessage>,
    connection_version  : Arc<AtomicU32>,
    initiated_by_us     : bool,
    connection_state    : Locked<State>,
    peer_addr           : String,
} impl Peer {
    pub fn new(stream : TcpStream, server_sender : mpsc::Sender<ServerMessage>, initiated_by_us : bool) -> Peer {
        let (sender, receiver) = mpsc::channel(512);
        let ip = stream.peer_addr().unwrap();
        let stream = Arc::new(Mutex::new(stream));
        let s2 = stream.clone();
        tokio::spawn(async move { Peer::handle_server_message(receiver, &s2).await.unwrap(); });
        Peer {
            stream,
            server_sender,
            sender,
            connection_version  : Arc::new(AtomicU32::new(1)),
            initiated_by_us,
            connection_state    : Arc::new(Mutex::new(State::Tcp)),
            peer_addr           : ip.to_string(),
        }
    }

    pub async fn read_message(&mut self) -> Result<(), Error> {
        let ip = self.peer_addr.clone();
        let span = span!(tracing::Level::DEBUG, "Reading message", ip = ip.as_str());
        let _enter = span.enter();
        let stream = self.stream.clone();
        let (message_type, payload) = self.check_header(&stream).await?;
        let state = self.connection_state.clone();
        let state = state.lock().await;
        if *state != State::Acknowledged {
            self.handle_handshake(message_type, payload, state, &stream).await?;
        }
        else {
            drop(state);
            match message_type.as_str() {
                "2plus2is4\u{0}\u{0}\u{0}" => {
                    debug!(ip = ip.as_str(), "2 plus 2 is 4!");
                    let message = Message::MinusOne;
                    Peer::send(message, &stream).await?;
                },
                "inv\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                    let message = Inv::read(&payload);
                    debug!("Received Inv message with {} items", &message.count.value);
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
                    debug!("{:?}", &message);
                },
                "getblocks\u{0}\u{0}\u{0}" => {
                    debug!("Received getblocks");
                    let message = GetBlocks::read(&payload);
                    self.server_sender.send(ServerMessage::GetBlocks(self.sender.clone(), message)).await?;
                },
                "transaction\u{0}" => {
                    let tx = blockchain::transaction::Transaction::read(&payload);
                    info!("Received tx, tx_hash: {}", utils::hash_to_string(&tx.hash().unwrap()));
                    self.server_sender.send(ServerMessage::AddTx(tx)).await?;
                },
                "block\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                    let block = blockchain::block::Block::read(&payload)?;
                    info!("Received block, block_hash: {}", utils::hash_to_string(&block.hash().unwrap()));
                    self.server_sender.send(ServerMessage::AddBlock(block)).await?;
                },
                _ => { warn!("didn't understand message type: {}", message_type); }
            }
        }
        Ok(())
    }

    pub async fn update(mut self) -> Result<(), Error> {

        loop {
            match self.read_message().await {
                Ok(_) => (),
                Err(e) => {
                    let ip = self.peer_addr.clone();
                    let span = span!(tracing::Level::ERROR, "Peer update loop ", ip = ip.as_str());
                    let _enter = span.enter();
                    match e {
                    Error::IOError(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                        } else {
                            error!("{:?}", e); break;
                        }
                    },
                    _ => { error!("{:?}", e); break; }
                    }
                }
            }
        }
        self.close().await?;
        Ok(())
    }

    pub async fn connect(self) -> Result<(), Error> {
        {
            let message = Message::WhoAmI(WhoAmI::default());
            Peer::send(message, &self.stream).await?;
        }
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

    async fn send<T>(message : Message, stream: &Locked<T>) -> Result<(), Error>
        where T: AsyncWrite + std::marker::Unpin {
        let mut stream = stream.lock().await;
        let mut buffer = Peer::prepare_header(message.name(), message.size())?;
        buffer.append(&mut message.send());
        stream.write_all(&buffer).await?;
        Ok(())
    }

    async fn read_header(&mut self, stream: &Locked<TcpStream>) -> Result<Vec<u8>, Error> {
        let mut buffer : [u8; 24] = [0; 24];
        let mut stream = stream.lock().await;
        stream.read_exact(&mut buffer).await?;
        Ok(buffer.to_vec())
    }

    async fn read_payload(&mut self, length : usize, stream: &Locked<TcpStream>) -> Result<Vec<u8>, Error> {
        if length != 0 {
            let mut buffer = vec![0; length];
            let mut stream = stream.lock().await;
            stream.read_exact(&mut buffer).await?;

            Ok(buffer)
        } else {
            Ok(vec![])
        }
    }

    async fn check_header(&mut self, stream: &Locked<TcpStream>) -> Result<(String, Vec<u8>), Error> {
        let header = self.read_header(stream).await?;
        let mut magic = header[0..4].to_vec();
        magic.reverse();
        let magic : u32 = deserialize(&magic)?;
        if magic != 42_2021 {
            error!("wrong magic number : {}", magic);
            return Err(Error::ConnectionClosed)
        }
        let message_type : String = String::from_utf8(header[4..16].to_vec())?;
        let mut length = header[16..24].to_vec();
        length.reverse();
        let length : u64 = deserialize(&length)?;
        let payload = self.read_payload(length as usize, stream).await?;
        Ok((message_type, payload))
    }

    async fn handle_handshake(&mut self, message_type: String, payload: Vec<u8>, mut state: tokio::sync::MutexGuard<'_, State>, stream: &Locked<TcpStream>) -> Result<(), Error> {
        match message_type.as_str() {
            "whoami\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                if *state == State::Tcp {
                    debug!("Received message whoami");
                    let message = WhoAmI::read(payload);
                    let message_ver = message.version;
                    let conn_ver = self.connection_version.load(std::sync::atomic::Ordering::Acquire);
                    if !self.initiated_by_us {
                        // send WhoAmI
                        let message = Message::WhoAmI(WhoAmI::new(conn_ver));
                        Peer::send(message, stream).await?;
                    }
                    Peer::send(Message::WhoAmIAck, stream).await?;
                    self.connection_version.store(min(message_ver, conn_ver),
                        std::sync::atomic::Ordering::Release);
                    *state = State::WhoAmI;
                } else {
                    error!("received unusual number of whoami");
                    return Err(Error::ConnectionClosed)
                }
            },
            "whoamiack\u{0}\u{0}\u{0}" => {
                if *state == State::WhoAmI {
                    *state = State::Acknowledged;
                    let stream_locked = stream.lock().await;
                    self.server_sender.send(ServerMessage::AddPeer(self.sender.clone(), stream_locked.peer_addr().unwrap())).await?;
                    debug!("Handshake completed");
                    drop(stream_locked);
                    info!("Asking blocks");
                    let getblocks = Message::GetBlocks(GetBlocks::from_hashes(vec![blockchain::Block::genesis_block()?.hash_header()?], vec![0;32]));
                    Peer::send(getblocks, stream).await?;
                } else {
                    error!("reveiced whoamiack message before whoami message");
                    return Err(Error::ConnectionClosed)
                }
            },
            _ => {
                error!("Recieved incorrect message_type : {:?}", message_type.as_str());
                return Err(Error::ConnectionClosed)
            }
        }
        Ok(())
    }

    async fn handle_server_message(mut receiver: tokio::sync::mpsc::Receiver<ServerMessage>, stream: &Locked<TcpStream>) -> Result<(), Error> {
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
                            Peer::send(message, stream).await.unwrap();
                        },
                        ServerMessage::GetBlocksReply(hashs) => {
                            let message = Message::Inv(Inv::from_vec(hashs));
                            Peer::send(message, stream).await.unwrap();
                        },
                        ServerMessage::AskBlocks(hashs) => {
                            debug!("Asking blocks");
                            let message = Message::GetData(Inv::from_vec(hashs));
                            Peer::send(message, stream).await.unwrap();
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
