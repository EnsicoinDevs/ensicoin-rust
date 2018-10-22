use sha2::{Sha256, Digest};
use model::hash::ToHex;

#[derive(Debug)]
struct Input {
    previous_output : (String, u64),
    script : Vec<String>
}

impl Input {
    pub fn to_string(&self) -> String {
        let mut string : String = "".to_string();
        string = string + &self.previous_output.0.clone();
        string = string + &self.previous_output.1.to_string();
        for e in &self.script {
            string += &e.clone();
        }
        string
    }
}

#[derive(Debug)]
struct Output {
    value   : u64,
    script  : Vec<String>
}

impl Output {
    pub fn to_string(&self) -> String {
        let mut string : String = "".to_string();
        string += &self.value.to_string();
        for e in &self.script {
            string += &e.clone();
        }
        string
    }
}

#[derive(Debug)]
pub struct Transaction {
    version : u64,
    flags   : Vec<String>,
    inputs  : Vec<Input>,
    outputs : Vec<Output>,
}


impl Transaction {
    pub fn to_string(&self) -> String {
        let mut string : String = "".to_string();
        string += &self.version.to_string();
        for e in &self.flags {
            string += &e.clone();
        }
        for e in &self.inputs {
            string += &e.to_string().clone();
        }
        for e in &self.outputs {
            string += &e.to_string().clone();
        }
        string
    }
    pub fn hash(&self) -> String {
        let hash_string = format!("{}", self.to_string());
        let mut sha = Sha256::new();
        sha.input(hash_string);
        let result = sha.result();
        let result = result[..].to_hex();
        sha = Sha256::new();
        sha.input(result);
        let result = sha.result();
        result[..].to_hex()
    }
}
