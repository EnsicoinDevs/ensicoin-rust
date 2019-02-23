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
        let mut magic : [u8; 4] = [0; 4];
        let mut message_type : [u8; 12] = [0; 12];
        let mut length : [u8; 8] = [0; 8];

        self.stream.read(&mut magic)?;
        let mut magic = magic.to_vec();
        magic.reverse();
        let magic : u32 = deserialize(&magic)?;
        if magic != 422021 {
            println!("wrong magic number : {}", magic);
            self.close().unwrap();
        }
        self.stream.read(&mut message_type)?;
        self.stream.read(&mut length)?;

        let mut length = length.to_vec();
        length.reverse();
        let size : u64 = deserialize(&length)?;
        let message_type : String = String::from_utf8(message_type.to_vec())?;
        if self.connection_state != State::Acknowledged {
            match message_type.as_str() {
                "whoami\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}" => {
                    if self.connection_state == State::Tcp {
                        println!("Received message whoami");
                        self.connection_version = message::WhoAmI::read(&self.stream).handle(&self.stream, 1, self.initiated_by_us)?;
                        self.connection_state = State::WhoAmI;
                    } else {
                        println!("received unusual number of whoami");
                        self.close()?;
                    }
                },
                "whoamiack\u{0}\u{0}\u{0}" => {
                    if self.connection_state == State::WhoAmI {
                        println!("Please print this");
                        self.connection_state = State::Acknowledged;
                        self.server_sender.send(ServerMessage::AddPeer(self.sender.clone(), self.stream.peer_addr().unwrap().ip())).unwrap();
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
            if size > 0 {
                match message_type.as_str() {
                    "getaddr" => {},
                    "addr" => {},
                    _ => ()
                }
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
                                std::sync::mpsc::TryRecvError::Disconnected => panic!("Peer tried to read but server died."),
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
}
