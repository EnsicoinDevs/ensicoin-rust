use std::net::IpAddr;
use std::cmp::min;
use bincode::{serialize, deserialize};
use std::net::TcpStream;
use std::io::prelude::*;
use std::sync::mpsc;
use utils::types::*;

//server messages
#[derive(Debug)]
pub enum ServerMessage {
    CreatePeer(IpAddr),
    AddPeer(mpsc::Sender<ServerMessage>, IpAddr),
    DeletePeer(IpAddr),

    CheckTxs(mpsc::Sender<ServerMessage>, Vec<Vec<u8>>),
    AskTxs(Vec<Vec<u8>>),

    CloseConnection,
    ClosePeer(IpAddr),
    CloseServer,
}

pub trait Size {
    fn size(&self) -> u64;
}


#[derive(Debug)]
pub struct WhoAmI {
    version         : u32,
    from            : Address,
    service_count   : VarUint,
    services        : VarStr,
} impl WhoAmI {
    pub fn new() -> WhoAmI {
        WhoAmI {
            version         : 0,
            from            : Address::from_string("46.193.66.26".to_string()).unwrap(),
            service_count   : VarUint::from_u64(1),
            services        : VarStr::from_string("node".to_string())
        }
    }

    pub fn read(payload : Vec<u8>) -> WhoAmI {
        println!("reading whoami message...");
        let mut version = payload[0..4].to_vec();
        version.reverse();
        let version : u32 = deserialize(&version).unwrap();

        let address = Address::new(payload[4..30].to_vec());
        let service_count = VarUint::new(&payload[30..].to_vec());
        let size = service_count.size() as usize;
        let services = VarStr::new(&payload[29+size..].to_vec());
        WhoAmI {
            version         : version,
            from            : address,
            service_count   : service_count,
            services        : services,
        }
    }

    // handle incoming WhoAmI
    // send WhoAmI and WhoAmIAck
    pub fn handle(&self, mut stream: &TcpStream, server_version : u32, we_connected : bool) -> Result<u32, Box<bincode::ErrorKind>> {
        println!("fully read incoming whoami, sending response");
        if we_connected == false {
            // send WhoAmI
            let message = WhoAmI {
                version         : server_version,
                from            : Address::from_string("46.193.66.26".to_owned()).unwrap(),
                service_count   : VarUint::from_u64(1),
                services        : VarStr::from_string("node".to_owned()),
            };
            WhoAmI::send(message, stream)?;
        }
        // send WhoAmIAck
        let mut buffer = Vec::new();
        let magic : u32 = 422021; ////////////////// magic number
        let mut magic = serialize(&magic)?;
        magic.reverse();
        buffer.append(&mut magic);
        let message_type = "whoamiack";
        buffer.append(&mut message_type.as_bytes().to_vec());
        buffer.append(&mut vec![0; 3]);
        buffer.append(&mut vec![0; 8]);
        stream.write(&buffer)?;

        Ok(min(server_version, self.version))
    }

    pub fn send(message: WhoAmI, mut stream: &TcpStream) -> Result<(), Box<bincode::ErrorKind>> {
        let mut buffer = Vec::new();
        let magic : u32 = 422021; ////////////////// magic number
        let mut magic = serialize(&magic)?;
        magic.reverse();
        buffer.append(&mut magic);
        let message_type = "whoami";
        buffer.append(&mut message_type.as_bytes().to_vec());
        buffer.append(&mut vec![0; 6]);
        let mut size = serialize(&message.size())?;
        size.reverse();
        buffer.append(&mut size);

        let mut version = serialize(&message.version)?;
        version.reverse();
        buffer.append(&mut version);
        buffer.append(&mut message.from.send());
        buffer.append(&mut message.service_count.send());
        buffer.append(&mut message.services.send());
        stream.write(&buffer)?;
        Ok(())
    }

}
impl Size for WhoAmI {
    fn size(&self) -> u64 {
        8 + self.from.size() + self.service_count.size() + self.services.size()
    }
}
// inv or getdata or notfound message
pub struct Inv {
    pub count       : VarUint,
    pub inventory   : Vec<InvVect>
}
impl Inv {
    pub fn read(buffer: &Vec<u8>) -> Inv {
        let count = VarUint::new(&buffer[..].to_vec());
        let mut offset : usize = count.size() as usize;

        let mut inventory = Vec::new();
        for _ in 0..count.value {
            let inv = InvVect::read(&buffer[offset..].to_vec());
            offset += inv.size() as usize;
            inventory.push(inv);
        }

        Inv {
            count:      count,
            inventory:  inventory
        }
    }

    pub fn send(self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.append(&mut self.count.send());
        for i in self.inventory {
            buffer.append(&mut i.send());
        }
        buffer
    }

}
impl Size for Inv {
    fn size(&self) -> u64 {
        self.count.size() + 36 * (self.inventory.len() as u64)
    }
}
