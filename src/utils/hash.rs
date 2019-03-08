use sha2::{Digest, Sha256};

pub fn hash(s: Vec<u8>) -> Vec<u8> {
    let mut sha = Sha256::new();
    sha.input(s);
    let result = sha.result();
    result[..].to_vec()
}
