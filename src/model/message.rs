pub struct Message<Message_type> {
    pub magic       : String,
    pub type        : String,
    pub timestamp   : u64,
    pub message     : Message_type
}

pub struct Who_am_i {
    version : u64
}

pub struct get_blocks {
    hashes : Vec<String>,
    stop_hash : String = "".to_owned()
}

pub struct Get_mempool;

pub struct Inv {
    type : char,
    hashes : Vec<String>
}

pub struct Get_data {
    inv : Inv
}

pub struct Not_found {
    type : char,
    hash : String = "".to_owned()
}
