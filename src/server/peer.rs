use bincode::serialize;
use bincode::deserialize;
use model::message;
use model::message::ServerMessage;
use std::net::TcpStream;
use std::io::prelude::*;
use utils::error::Error;
use std::sync::mpsc;

#[derive(PartialEq)]
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
            server_sender       : server_sender,
            sender              : tx,
            receiver            : rx,
            connection_version  : 1,
            initiated_by_us     : initiated_by_us,
            connection_state    : State::Tcp,
        }
    }

    pub fn read_message(&mut self) -> Result<(), Error> {
        let header = self.read_header()?;
        let mut magic = header[0..4].to_vec();
        magic.reverse();
        let magic : u32 = deserialize(&magic)?;
        if magic != 422021 {
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
                        self.connection_version = message::WhoAmI::read(payload).handle(&self.stream, 1, self.initiated_by_us)?;
                        self.connection_state = State::WhoAmI;
                    } else {
                        println!("received unusual number of whoami");
                        self.close()?;
                    }
                },
                "whoamiack\u{0}\u{0}\u{0}" => {
                    if self.connection_state == State::WhoAmI {
                        self.connection_state = State::Acknowledged;
                        self.server_sender.send(ServerMessage::AddPeer(self.sender.clone(), self.stream.peer_addr().unwrap().ip())).unwrap();
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
            match message_type.as_str() {
                "2plus2is4" => {
                    let message = self.prepare_header("minus1thats3".to_string(), 0)?;
                    self.send(message);
                },
                "getaddr" => {},
                "addr" => {},
                _ => ()
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
                        _                               => ()
                    }
                },
                Err(e)  =>  match e {
                                std::sync::mpsc::TryRecvError::Disconnected => break,
                                _ => ()
                            }
            }
            match self.read_message() {
                Ok(_) => (),
                Err(e) => { match e {
                                Error::IOError(e) => {
                                    if e.kind() == std::io::ErrorKind::WouldBlock {
                                        ()
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
        let message = message::WhoAmI::new();
        message::WhoAmI::send(message, &self.stream).unwrap();
        self.update();
        }

    fn close(&self) -> Result<(), Error>{
        self.server_sender.send(ServerMessage::DeletePeer(self.stream.peer_addr().unwrap().ip())).unwrap();
        self.stream.shutdown(std::net::Shutdown::Both)?;
        Err(Error::ConnectionClosed)
    }

    fn prepare_header(&self, message_type : String, size : u64) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        let magic : u32 = 422021; ////////////////// magic number
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
        self.stream.write(&message).unwrap();
    }

    fn read_header(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer : [u8; 24] = [0; 24];
        self.stream.read(&mut buffer)?;
        Ok(buffer.to_vec())
    }

    fn read_payload(&mut self, length : usize) -> Result<Vec<u8>, Error> {
        if length != 0 {
            let mut buffer = vec![0; length];
            self.stream.read(&mut buffer)?;
            Ok(buffer)
        } else {
            Ok(vec![0])
        }
    }
}
