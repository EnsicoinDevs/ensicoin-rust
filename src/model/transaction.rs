use bincode::serialize;
use utils::types::*;
use utils::hash;

#[derive(Debug)]
pub struct Outpoint {
    pub hash:   Vec<u8>,
    pub index:  u32
}

impl Outpoint {
    pub fn send(&self) -> Vec<u8> {
        let mut buffer = self.hash.clone();
        let mut index = serialize(&self.index).unwrap();
        index.reverse();
        buffer.append(&mut index);

        buffer
    }
}

#[derive(Debug)]
pub struct TxIn {
    pub previous_output:    Outpoint,
    pub script:             VarStr
}

impl TxIn {
    pub fn send(&self) -> Vec<u8> {
        let mut buffer = self.previous_output.send();
        buffer.append(&mut self.script.send());

        buffer
    }
}

#[derive(Debug)]
pub struct TxOut {
    pub value:  u64,
    pub script: VarStr
}

impl TxOut {
    pub fn send(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let mut value = serialize(&self.value).unwrap();
        value.reverse();
        buffer.append(&mut value);
        buffer.append(&mut self.script.send());

        buffer
    }
}

#[derive(Debug)]
pub struct Transaction {
    pub version:        u32,
    pub flags_count:    VarUint,
    pub flags:          Vec<VarStr>,
    pub inputs_count:   VarUint,
    pub inputs:         Vec<TxIn>,
    pub outputs_count:  VarUint,
    pub outputs:        Vec<TxOut>
}

impl Transaction {
    pub fn hash(&self) -> Vec<u8> {
        let buffer = self.send();

        hash::hash(hash::hash(buffer))
    }

    pub fn send(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut version = serialize(&self.version).unwrap();
        version.reverse();
        buffer.append(&mut version);

        buffer.append(&mut self.flags_count.send());
        for flag in &self.flags {
            buffer.append(&mut flag.send());
        }

        buffer.append(&mut self.inputs_count.send());
        for input in &self.inputs {
            buffer.append(&mut input.send());
        }

        buffer.append(&mut self.outputs_count.send());
        for output in &self.outputs {
            buffer.append(&mut output.send());
        }

        buffer
    }
}
