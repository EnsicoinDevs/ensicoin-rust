use bincode::{serialize, deserialize};
use std::net::TcpStream;
use std::io::prelude::*;

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
}

pub struct Var_uint {
    size    : u8,
    value   : u64
} impl Var_uint {
    fn new(mut stream : &TcpStream) -> Var_uint {
        let mut size : [u8; 1] = [0; 1];
        stream.read(&mut size).unwrap();
        let mut size : u8 = deserialize(&size).unwrap();
        let mut value : [u8; 1] = [0; 1];
        match size {
            0xFD => { size = 2; let mut value : [u8; 2] = [0; 2];},
            0xFE => { size = 4; let mut value : [u8; 4] = [0; 4];},
            0xFF => { size = 8; let mut value : [u8; 8] = [0; 8];},
            _    => {return Var_uint {size: 1, value: size as u64};}
        }

        stream.read_exact(&mut value).unwrap();
        let value : u64 = deserialize(&value).unwrap();

        Var_uint {
            size: size as u8,
            value: value
        }
    }
}

pub struct Var_str {
    size    : Var_uint,
    value   : String
} impl Var_str {
    fn new(stream : &TcpStream) -> Var_str {
        let length : Var_uint = Var_uint::new(stream);
        let mut value : String = "".to_owned();
        let mut adaptator = stream.take(length.value);
        adaptator.read_to_string(&mut value);
        Var_str {
            size: length,
            value: value
        }
    }
}

pub struct Who_am_i {
    version         : u64,
    from            : Address,
    service_count   : Var_uint,
    services        : Var_str
} impl Who_am_i {
    pub fn new(mut stream : &TcpStream) -> Who_am_i {
        let mut version : [u8; 8] = [0; 8];
        let address : Address;
        let service_count : Var_uint;
        let services : Var_str;

        stream.read(&mut version).unwrap();
        let version : u64 = deserialize(&version).unwrap();
        address = Address::new(stream);
        service_count = Var_uint::new(stream);
        services = Var_str::new(stream);

        Who_am_i {
            version: version,
            from: address,
            service_count: service_count,
            services: services
        }
    }

    pub fn handle(&self) {

    }
}

pub struct get_blocks {
    hashes : Vec<String>,
    stop_hash : String
}

pub struct Get_mempool;

pub struct Inv {
    inv_type : char,
    hashes : Vec<String>
}

pub struct Get_data {
    inv : Inv
}

pub struct Not_found {
    not_found_type : char,
    hash : String
}
