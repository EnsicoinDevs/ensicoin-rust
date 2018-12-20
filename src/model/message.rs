use std::net::TcpStream;

pub struct Address {
    timestamp   : u64,
    ipv6_4      : String,
    port        : u16
}

pub struct Var_uint {

}

pub struct Var_str {

}

pub struct Who_am_i {
    version         : u64,
    from            : Address,
    service_count   : Var_uint,
    services        : Var_str
} impl Who_am_i {
    fn new(stream : &TcpStream) -> Who_am_i {

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
