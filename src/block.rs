use Transaction;
use sha2::Sha256;

struct Block {
    version : u64,
    index : u64,
    timestamp : u64,
    hash : Sha256,
    previousHash : Sha256,
    nonce : u64,
    transactions : Vec<Transaction>,
}

impl Block {
    fn hash(&self) -> Sha256 {
        hashString = format!("{}{}{}{}", version as str, index as str, timestamp as str, previousHash as str);
        sha = Sha256::new().input(version as str)
    }
}
