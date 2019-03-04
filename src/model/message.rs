use std::net::IpAddr;
use std::cmp::min;
use bincode::{serialize, deserialize};
use std::net::TcpStream;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::mpsc;

//server messages
pub enum ServerMessage {
    CreatePeer(IpAddr),
    AddPeer(mpsc::Sender<ServerMessage>, IpAddr),
    DeletePeer(IpAddr),
    CloseConnection,
    ClosePeer(IpAddr),
    CloseServer,
}

trait Size {
    fn size(&self) -> u64;
}

#[derive(Debug)]
pub struct Address {
    timestamp   : u64,
    ip          : Vec<u8>,
    port        : u16
} impl Address {
    fn new(payload : Vec<u8>) -> Address {
        let mut timestamp   = payload[0..8].to_vec();
        let ip              = payload[8..24].to_vec();
        let mut port        = payload[24..].to_vec();

        timestamp.reverse();
        let timestamp : u64 = deserialize(&timestamp).unwrap();
        port.reverse();
        let port : u16 = deserialize(&port).unwrap();

        Address {
            timestamp: timestamp,
            ip: ip,
            port: port
        }
    }
    pub fn from_string(address : String) -> Result<Address, std::time::SystemTimeError> {

        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let ip : IpAddr = address.parse().unwrap();
        let bytes : [u8; 16];
        match ip {
            IpAddr::V4(ip) => { bytes = ip.to_ipv6_mapped().octets(); },
            IpAddr::V6(ip) => { bytes = ip.octets(); }
        }
        Ok(Address {
            timestamp: t.as_secs(),
            ip: bytes.to_vec(),
            port: 4224
        })

    }

    pub fn send(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut t = serialize(&self.timestamp).unwrap();
        t.reverse();
        buffer.append(&mut t);
        let mut ip = self.ip.clone();
        buffer.append(&mut ip);
        let mut p = serialize(&self.port).unwrap();
        p.reverse();
        buffer.append(&mut p);
        buffer
    }
}
impl Size for Address {
    fn size(&self) -> u64 {
        26
    }
}

#[derive(Debug)]
pub struct VarUint {
    size    : u8,
    value   : u64
} impl VarUint {
    fn new(payload : &Vec<u8>) -> VarUint {
        let mut size = payload[0];
        match size {
            0xFD => { size = 2; },
            0xFE => { size = 4; },
            0xFF => { size = 8; },
            _    => {return VarUint {size: 1, value: size as u64};}
        }

        let mut value = payload[1..size as usize].to_vec();
        value.reverse();
        let value : u64 = deserialize(&value).unwrap();

        VarUint {
            size: size,
            value: value
        }
    }

    pub fn from_u64(value : u64) -> VarUint {
        VarUint {
            size: 8,
            value: value
        }
    }

    pub fn send(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut v : Vec<u8>;
        match self.size {
            2 => v = serialize(&(self.value as u16)).unwrap(),
            4 => v = serialize(&(self.value as u32)).unwrap(),
            8 => v = serialize(&self.value).unwrap(),
            _ => panic!()
        }
        v.reverse();
        buffer.push(self.size);
        buffer.append(&mut v);
        buffer
    }
}
impl Size for VarUint {
    fn size(&self) -> u64 {
        (1 + self.size).into()
    }
}

#[derive(Debug)]
pub struct VarStr {
    size    : VarUint,
    value   : String
} impl VarStr {
    fn new(payload : &Vec<u8>) -> VarStr {
        let length : VarUint = VarUint::new(payload);
        let size = length.size() as usize;
        let value = payload[size..length.value as usize].to_vec();
        let value = String::from_utf8(value).unwrap();
        VarStr {
            size: length,
            value: value
        }
    }

    pub fn from_string(value: String) -> VarStr {
        VarStr {
            size: VarUint::from_u64(value.len() as u64),
            value: value
        }
    }

    pub fn send(&self) -> Vec<u8> {
        let mut buffer = self.size.send();
        buffer.append(&mut self.value.as_bytes().to_vec());
        buffer
    }
}
impl Size for VarStr {
    fn size(&self) -> u64 {
        self.size.size() + (self.value.len() as u64)
    }
}

pub struct InvVect {
    hash_type   : u32,
    hash        : Vec<u8>
}
impl InvVect {
    pub fn send(mut self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut h_type = serialize(&self.hash_type).unwrap();
        h_type.reverse();
        buffer.append(&mut h_type);
        buffer.append(&mut self.hash);
        buffer
    }
}
impl Size for InvVect {
    fn size(&self) -> u64 {
        36
    }
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
    count       : VarUint,
    inventory   : Vec<InvVect>
}
impl Inv {
    pub fn send(mut self) -> Vec<u8> {
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
