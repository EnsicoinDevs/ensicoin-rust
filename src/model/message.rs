use std::net::IpAddr;
use std::cmp::min;
use bincode::serialize_into;
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
    ip          : [u8; 16],
    port        : u16
} impl Address {
    fn new(mut stream : &TcpStream) -> Address {
        let mut timestamp   : [u8; 8] = [0; 8];
        let mut ip          : [u8; 16] = [0; 16];
        let mut port        : [u8; 2] = [0; 2];

        stream.read(&mut timestamp).unwrap();
        stream.read(&mut ip).unwrap();
        stream.read(&mut port).unwrap();
        let mut timestamp = timestamp.to_vec();
        timestamp.reverse();
        let timestamp : u64 = deserialize(&timestamp).unwrap();
        let mut port = port.to_vec();
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
            ip: bytes,
            port: 4224
        })

    }

    pub fn send(&self, mut stream : &TcpStream) {
        let mut t = serialize(&self.timestamp).unwrap();
        t.reverse();
        stream.write(&t).unwrap();
        stream.write(&self.ip).unwrap();
        let mut p = serialize(&self.port).unwrap();
        p.reverse();
        stream.write(&p).unwrap();
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
    fn new(mut stream : &TcpStream) -> VarUint {
        let mut size : [u8; 1] = [0; 1];
        stream.read(&mut size).unwrap();
        let mut size : u8 = deserialize(&size).unwrap();
        let mut value : [u8; 1] = [0; 1];
        match size {
            0xFD => { size = 2; let mut value : [u8; 2] = [0; 2];},
            0xFE => { size = 4; let mut value : [u8; 4] = [0; 4];},
            0xFF => { size = 8; let mut value : [u8; 8] = [0; 8];},
            _    => {return VarUint {size: 1, value: size as u64};}
        }

        stream.read(&mut value).unwrap();
        let value : u64 = deserialize(&value).unwrap();

        VarUint {
            size: size as u8,
            value: value
        }
    }

    pub fn from_u64(value : u64) -> VarUint {
        VarUint {
            size: 8,
            value: value
        }
    }

    pub fn send(&self, mut stream : &TcpStream) {
        let mut v;
        match self.size {
            2 => v = serialize(&(self.value as u16)).unwrap(),
            4 => v = serialize(&(self.value as u32)).unwrap(),
            8 => v = serialize(&self.value).unwrap(),
            _ => panic!()
        }
        v.reverse();
        stream.write(&v).unwrap();
    }
}
impl Size for VarUint {
    fn size(&self) -> u64 {
        (1 + self.size*8).into()
    }
}

#[derive(Debug)]
pub struct VarStr {
    size    : VarUint,
    value   : String
} impl VarStr {
    fn new(mut stream : &TcpStream) -> VarStr {
        let length : VarUint = VarUint::new(stream);
        let mut value = vec![0; length.value as usize];
        stream.read_exact(&mut value).unwrap();
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

    pub fn send(&self, mut stream : &TcpStream) {
        self.size.send(stream);
        stream.write(&self.value.as_bytes()).unwrap();
    }
}
impl Size for VarStr {
    fn size(&self) -> u64 {
        self.size.size() + (self.value.len() as u64)
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
            version         : 1,
            from            : Address::from_string("46.193.66.26".to_string()).unwrap(),
            service_count   : VarUint::from_u64(1),
            services        : VarStr::from_string("node".to_string())
        }
    }

    pub fn read(mut stream : &TcpStream) -> WhoAmI {
        println!("reading whoami message...");
        let mut version : [u8; 4] = [0; 4];
        let address : Address;
        let service_count : VarUint;
        let services : VarStr;

        stream.read(&mut version).unwrap();
        let version : u32 = deserialize(&version).unwrap();
        address = Address::new(stream);
        service_count = VarUint::new(stream);
        services = VarStr::new(stream);

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
        if we_connected == false {
            println!("fully read incoming message, sending response");
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
        let magic : u32 = 422021; ////////////////// magic number
        let mut magic = serialize(&magic)?;
        magic.reverse();
        stream.write(&magic)?;
        let message_type = "whoamiack";
        stream.write(&message_type.as_bytes())?;
        stream.write(b"\0\0\0")?;
        serialize_into(stream, &(0 as u64))?;

        Ok(min(server_version, self.version))
    }

    pub fn send(message: WhoAmI, mut stream: &TcpStream) -> Result<(), Box<bincode::ErrorKind>> {

        let magic : u32 = 422021; ////////////////// magic number
        let mut magic = serialize(&magic)?;
        magic.reverse();
        stream.write(&magic)?;
        let message_type = "whoami";
        stream.write(&message_type.as_bytes())?;
        stream.write(b"\0\0\0\0\0\0")?;
        let mut size = serialize(&message.size())?;
        size.reverse();
        stream.write(&size)?;

        let mut version = serialize(&message.version)?;
        version.reverse();
        stream.write(&version)?;
        message.from.send(stream);
        message.service_count.send(stream);
        message.services.send(stream);
        println!("Are you kidding me?");
        Ok(())
    }

}
impl Size for WhoAmI {
    fn size(&self) -> u64 {
        8 + self.from.size() + self.service_count.size() + self.services.size()
    }
}

// pub struct GetBlocks {
//     hashes : Vec<String>,
//     stop_hash : String
// }
//
// pub struct GetMempool;
//
// pub struct Inv {
//     inv_type : char,
//     hashes : Vec<String>
// }
//
// pub struct GetData {
//     inv : Inv
// }
//
// pub struct NotFound {
//     not_found_type : char,
//     hash : String
// }
