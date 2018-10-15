#[derive(Debug)]
pub struct Hash {
    pub val : Vec<u8>
}

impl Hash {
    pub fn to_string(&self) -> String {
        let mut hash : String = "".to_owned();
        for i in self.val.iter() {
            hash.push_str(&i.to_string()) ;
        }
        hash
    }
}
