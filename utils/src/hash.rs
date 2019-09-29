use sha2::{Digest, Sha256};

pub fn hash(s: Vec<u8>) -> Vec<u8> {
    let mut sha = Sha256::new();
    sha.input(s.as_slice());
    let result = sha.result();
    result[..].to_vec()
}

pub fn hash_to_string(hash: &Vec<u8>) -> String {
    hash.iter().fold(String::new(), |acc, b| format!("{}{:02x}", acc, b))
}
