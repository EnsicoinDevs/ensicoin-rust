use bincode::deserialize;
use bincode::serialize;
use utils::Size;
use std::net::IpAddr;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone)]
pub struct Address {
    timestamp: u64,
    ip: Vec<u8>,
    port: u16,
}
impl Address {
    pub fn new(payload: Vec<u8>) -> Self {
        let mut timestamp = payload[0..8].to_vec();
        let ip = payload[8..24].to_vec();
        let mut port = payload[24..].to_vec();

        timestamp.reverse();
        let timestamp: u64 = deserialize(&timestamp).unwrap();
        port.reverse();
        let port: u16 = deserialize(&port).unwrap();

        Self {
            timestamp,
            ip,
            port,
        }
    }
    pub fn from_string(address: String) -> Result<Address, std::time::SystemTimeError> {
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let ip: IpAddr = address.parse().unwrap();
        let bytes: [u8; 16];
        match ip {
            IpAddr::V4(ip) => {
                bytes = ip.to_ipv6_mapped().octets();
            }
            IpAddr::V6(ip) => {
                bytes = ip.octets();
            }
        }
        Ok(Address {
            timestamp: t.as_secs(),
            ip: bytes.to_vec(),
            port: 4224,
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

#[derive(Debug, Clone)]
pub struct VarUint {
    size: u8,
    pub value: u64,
}
impl VarUint {
    pub fn new(payload: &[u8]) -> Self {
        let mut size = payload[0];
        match size {
            0xFD => {
                size = 2;
            }
            0xFE => {
                size = 4;
            }
            0xFF => {
                size = 8;
            }
            _ => {
                return Self {
                    size: 1,
                    value: u64::from(size),
                };
            }
        }

        let mut value = payload[1..size as usize].to_vec();
        value.reverse();
        let value: u64 = deserialize(&value).unwrap();

        Self {
            size,
            value,
        }
    }

    pub fn from_u64(value: u64) -> Self {
        let size;
        if value < 252 {
            size = 1;
        } else if value < 0xFFFF {
            size = 2;
        } else if value < 0xFFFFFFFF {
            size = 4;
        } else {
            size = 8;
        }
        Self {
            size,
            value,
        }
    }

    pub fn send(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut v: Vec<u8>;
        match self.size {
            1 => v = serialize(&(self.value as u8)).unwrap(),
            2 => {
                buffer.push(0xFD);
                v = serialize(&(self.value as u16)).unwrap();},
            4 => {
                buffer.push(0xFE);
                v = serialize(&(self.value as u32)).unwrap();},
            8 => {
                buffer.push(0xFF);
                v = serialize(&self.value).unwrap();},
            _ => panic!(),
        }
        v.reverse();
        buffer.append(&mut v);
        buffer
    }
}
impl Size for VarUint {
    fn size(&self) -> u64 {
        self.size.into()
    }
}

#[derive(Debug, Clone)]
pub struct VarStr {
    size: VarUint,
    pub value: String,
}
impl VarStr {
    pub fn new(payload: &[u8]) -> Self {
        let length: VarUint = VarUint::new(payload);
        let size = length.size() as usize;
        let value = payload[size..=length.value as usize].to_vec();
        let value = String::from_utf8(value).unwrap();
        Self {
            size: length,
            value,
        }
    }

    pub fn from_string(value: String) -> Self {
        Self {
            size: VarUint::from_u64(value.len() as u64),
            value,
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

#[derive(Debug)]
pub struct InvVect {
    pub hash_type: u32,
    pub hash: Vec<u8>,
}
impl InvVect {
    pub fn from_vec(array: Vec<u8>, hash_type: u32) -> Self {
        Self {
            hash_type,
            hash: array,
        }
    }

    pub fn read(buffer: &[u8]) -> Self {
        let mut hash_type = buffer[0..4].to_vec();
        hash_type.reverse();
        let hash_type : u32 = deserialize(&hash_type).unwrap();

        InvVect {
            hash_type,
            hash:   buffer[4..35].to_vec()
        }
    }

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
