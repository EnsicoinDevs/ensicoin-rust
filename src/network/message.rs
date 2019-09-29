use std::net::SocketAddr;
use bincode::{serialize, deserialize};
use tokio::sync::mpsc;
use model::*;
use blockchain::transaction::Transaction;
use blockchain::block::Block;
use utils::Size;
use utils::ToBytes;

//server messages
#[derive(Debug)]
pub enum ServerMessage {
    CreatePeer(SocketAddr),
    AddPeer(mpsc::Sender<ServerMessage>, SocketAddr),
    DeletePeer(SocketAddr),

    CheckBlocks(mpsc::Sender<ServerMessage>, Vec<Vec<u8>>),
    AskBlocks(Vec<(Vec<u8>, u32)>),
    AddBlock(Block),

    CheckTxs(mpsc::Sender<ServerMessage>, Vec<Vec<u8>>),
    AskTxs(Vec<Vec<u8>>),
    AddTx(Transaction),

    GetBlocks(mpsc::Sender<ServerMessage>, GetBlocks),
    GetBlocksReply(Vec<(Vec<u8>, u32)>),

    CloseConnection,
    ClosePeer(SocketAddr),
    CloseServer,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Message {
    WhoAmI(WhoAmI),
    WhoAmIAck,
    Inv(Inv),
    GetData(Inv),
    GetBlocks(GetBlocks),
    TwoPlusTwo,
    MinusOne,
}

impl Message {
    pub fn name(&self) -> &str {
        match self {
            Message::WhoAmI(_)      => "whoami\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}",
            Message::WhoAmIAck      => "whoamiack\u{0}\u{0}\u{0}",
            Message::Inv(_)         => "inv\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}",
            Message::GetData(_)     => "getdata\u{0}\u{0}\u{0}\u{0}\u{0}",
            Message::GetBlocks(_)   => "getblocks\u{0}\u{0}\u{0}",
            Message::TwoPlusTwo     => "2plus2is4\u{0}\u{0}\u{0}",
            Message::MinusOne       => "minus1thats3",
        }
    }
}

impl Size for Message {
    fn size(&self) -> u64 {
        match self {
            Message::WhoAmI(m)      => m.size(),
            Message::Inv(m)         => m.size(),
            Message::GetData(m)     => m.size(),
            Message::GetBlocks(m)   => m.size(),
            _                       => 0,
        }
    }
}

impl ToBytes for Message {
    fn send(&self) -> Vec<u8> {
        match self {
            Message::WhoAmI(m) => m.send(),
            Message::Inv(m) => m.send(),
            Message::GetData(m) => m.send(),
            Message::GetBlocks(m) => m.send(),
            _ => Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct WhoAmI {
    pub version     : u32,
    from            : Address,
    service_count   : VarUint,
    services        : VarStr,
} impl WhoAmI {
    pub fn new(version: u32) -> Self {
        let mut m = Self::default();
        m.version = version;
        m
    }

    pub fn read(payload : Vec<u8>) -> Self {
        let mut version = payload[0..4].to_vec();
        version.reverse();
        let version : u32 = deserialize(&version).unwrap();

        let address = Address::new(payload[4..30].to_vec());
        let service_count = VarUint::new(&payload[30..].to_vec());
        let size = service_count.size() as usize;
        let services = VarStr::new(&payload[29+size..].to_vec());
        Self {
            version,
            from: address,
            service_count,
            services,
        }
    }
}

impl ToBytes for WhoAmI {
    fn send(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let mut version = serialize(&self.version).unwrap();
        version.reverse();
        buffer.append(&mut version);
        buffer.append(&mut self.from.send());
        buffer.append(&mut self.service_count.send());
        buffer.append(&mut self.services.send());
        buffer
    }
}

impl Default for WhoAmI {
    fn default() -> Self {
        Self {
            version         : 0,
            from            : Address::from_string("46.193.66.26".to_string()).unwrap(),
            service_count   : VarUint::from_u64(1),
            services        : VarStr::from_string("node".to_string())
        }
    }
}

impl Size for WhoAmI {
    fn size(&self) -> u64 {
        4 + self.from.size() + self.service_count.size() + self.services.size()
    }
}

// inv or getdata or notfound message
#[derive(Debug)]
pub struct Inv {
    pub count       : VarUint,
    pub inventory   : Vec<InvVect>
}
impl Inv {

    pub fn from_vec(hashs: Vec<(Vec<u8>, u32)>) -> Inv {
        let count = VarUint::from_u64(hashs.len() as u64);
        let mut inventory = Vec::new();
        for hash in hashs {
            inventory.push(InvVect::from_vec(hash.0, hash.1));
        }

        Inv {
            count,
            inventory
        }
    }

    pub fn read(buffer: &[u8]) -> Inv {
        let count = VarUint::new(buffer);
        let mut offset : usize = count.size() as usize;

        let mut inventory = Vec::new();
        for _ in 0..count.value {
            let inv = InvVect::read(&buffer[offset..].to_vec());
            offset += inv.size() as usize;
            inventory.push(inv);
        }

        Inv {
            count,
            inventory
        }
    }
}

impl ToBytes for Inv {
    fn send(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.append(&mut self.count.send());
        for i in &self.inventory {
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

#[derive(Debug)]
pub struct GetBlocks {
    pub count           : VarUint,
    pub block_locator   : Vec<Vec<u8>>,
    pub hash_stop       : Vec<u8>
}
impl GetBlocks {

    pub fn from_hashes(hashes: Vec<Vec<u8>>, hash_stop: Vec<u8>) -> Self {
        Self {
            count: VarUint::from_u64(hashes.len() as u64),
            block_locator: hashes,
            hash_stop,
        }
    }

    pub fn read(buffer: &[u8]) -> GetBlocks {
        let count = VarUint::new(buffer);
        let mut offset : usize = count.size() as usize;

        let mut block_locator = Vec::new();
        for _ in 0..count.value {
            block_locator.push(buffer[offset..offset+32].to_vec());
            offset += 32;
        }
        let hash_stop = buffer[offset..offset+32].to_vec();

        GetBlocks {
            count,
            block_locator,
            hash_stop
        }
    }
}

impl ToBytes for GetBlocks {
    fn send(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.append(&mut self.count.send());

        for hash in &self.block_locator {
            buffer.append(&mut hash.clone());
        }
        buffer.append(&mut self.hash_stop.clone());

        buffer
    }
}

impl Size for GetBlocks {
    fn size(&self) -> u64 {
        self.count.size() + 32 * self.block_locator.len() as u64 + 32
    }
}
