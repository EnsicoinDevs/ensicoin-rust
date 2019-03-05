use model::message::Size;
use std::time::UNIX_EPOCH;
use bincode::serialize;
use std::net::IpAddr;
use bincode::deserialize;
use std::time::SystemTime;


#[derive(Debug)]
pub struct Address {
    timestamp   : u64,
    ip          : Vec<u8>,
    port        : u16
} impl Address {
    pub fn new(payload : Vec<u8>) -> Address {
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
#[derive(Clone)]
pub struct VarUint {
    size    : u8,
    value   : u64
} impl VarUint {
    pub fn new(payload : &Vec<u8>) -> VarUint {
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
#[derive(Clone)]
pub struct VarStr {
    size    : VarUint,
    value   : String
} impl VarStr {
    pub fn new(payload : &Vec<u8>) -> VarStr {
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

    pub fn val(&self) -> String {
        self.val().clone()
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
