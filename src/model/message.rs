use std::cmp::min;
use serde::ser::Serializer;
use bincode::serialize_into;
use bincode::{serialize, deserialize};
use serde::ser::{SerializeSeq};
use std::net::TcpStream;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;

trait Size {
    fn size(&self) -> u64;
}

#[derive(Debug)]
pub struct Address {
    timestamp   : u64,
    ipv6_4      : String,
    port        : u16
} impl Address {
    fn new(mut stream : &TcpStream) -> Address {
        let mut timestamp   : [u8; 8] = [0; 8];
        let mut ip          : [u8; 16] = [0; 16];
        let mut port        : [u8; 2] = [0; 2];

        stream.read(&mut timestamp).unwrap();
        stream.read(&mut ip).unwrap();
        stream.read(&mut port).unwrap();

        let timestamp : u64 = deserialize(&timestamp).unwrap();
        let ip : String = deserialize(&ip).unwrap();
        let port : u16 = deserialize(&port).unwrap();

        Address {
            timestamp: timestamp,
            ipv6_4: ip,
            port: port
        }
    }
    pub fn from_string(address : String) -> Result<Address, std::time::SystemTimeError> {

        let t = SystemTime::now().duration_since(UNIX_EPOCH)?;
        Ok(Address {
            timestamp: t.as_secs(),
            ipv6_4: address,
            port: 4224
        })

    }
}
impl Size for Address {
    fn size(&self) -> u64 {
        10 + (self.ipv6_4.len() as u64)
    }
}
impl serde::ser::Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(self.size() as usize))?;
        seq.serialize_element(&self.timestamp)?;
        for b in self.ipv6_4.bytes() {
            seq.serialize_element(&b)?;
        }
        let mut i = self.ipv6_4.len();
        while i < 16 {
            seq.serialize_element(&'\0')?;
            i += 1;
        }
        seq.serialize_element(&self.port)?;
        seq.end()
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
}
impl Size for VarUint {
    fn size(&self) -> u64 {
        (1 + self.size*8).into()
    }
}
impl serde::ser::Serialize for VarUint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(self.size() as usize))?;
        seq.serialize_element(&self.size)?;
        match self.size {
            2 => seq.serialize_element(&(self.value as u16))?,
            4 => seq.serialize_element(&(self.value as u32))?,
            8 => seq.serialize_element(&self.value)?,
            _ => panic!()
        }
        seq.end()
    }
}

#[derive(Debug)]
pub struct VarStr {
    size    : VarUint,
    value   : String
} impl VarStr {
    fn new(stream : &TcpStream) -> VarStr {
        let length : VarUint = VarUint::new(stream);
        let mut value : String = "".to_owned();
        let mut adaptator = stream.take(length.value);
        adaptator.read_to_string(&mut value).unwrap();
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
}
impl Size for VarStr {
    fn size(&self) -> u64 {
        self.size.size() + (self.value.len() as u64)
    }
}
impl serde::ser::Serialize for VarStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(self.size() as usize))?;
        seq.serialize_element(&self.size)?;
        for b in self.value.bytes() {
            seq.serialize_element(&b)?;
        }
        seq.end()
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
            from            : Address::from_string("127.0.0.1".to_string()).unwrap(),
            service_count   : VarUint::from_u64(1),
            services        : VarStr::from_string("node".to_string())
        }
    }

    pub fn read(mut stream : &TcpStream) -> WhoAmI {
        let mut version : [u8; 4] = [0; 4];
        let address : Address;
        let service_count : VarUint;
        let services : VarStr;

        stream.read(&mut version).unwrap();
        dbg!(&version);
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
    pub fn handle(&self, mut stream: &TcpStream, server_version : u32) -> Result<Arc<u32>, Box<bincode::ErrorKind>> {
        dbg!(&self);
        // send WhoAmI
        let message = WhoAmI {
            version         : server_version,
            from            : Address::from_string("127.0.0.1:4224".to_owned()).unwrap(),
            service_count   : VarUint::from_u64(1),
            services        : VarStr::from_string("node".to_owned()),
        };
        WhoAmI::send(message, stream)?;

        // send WhoAmIAck
        let magic : u32 = 422021; ////////////////// magic number
        serialize_into(stream, &magic)?;
        let message_type = "whoamiack";
        let message_type = serialize(&message_type.as_bytes())?;
        stream.write(&message_type)?;
        stream.write(b"\0\0\0")?;
        serialize_into(stream, &(0 as u64))?;

        Ok(Arc::new(min(server_version, self.version)))
    }

    pub fn send(message: WhoAmI, mut stream: &TcpStream) -> Result<(), Box<bincode::ErrorKind>> {

        let magic : u32 = 422021; ////////////////// magic number
        let mut magic = serialize(&magic)?;
        magic.reverse();
        stream.write(&magic)?;
        let message_type = "whoami";
        // let message_type = serialize(&message_type.as_bytes())?;
        // let message_type = message.reverse
        stream.write(&message_type.as_bytes())?;
        stream.write(b"\0\0\0\0\0\0")?;
        serialize_into(stream, &message.size())?;

        serialize_into(stream, &(0 as u64)).unwrap();
        serialize_into(stream, &message.version)?;
        serialize_into(stream, &message.from)?;
        serialize_into(stream, &message.service_count)?;
        serialize_into(stream, &message.services)?;
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
