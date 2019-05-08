use bincode::deserialize;
use bincode::serialize;
use model::message::Size;
use utils::hash;
use utils::Error;
use utils::types::*;

#[derive(Debug, Clone)]
pub struct Outpoint {
    pub hash: Vec<u8>,
    pub index: u32,
}

impl Outpoint {
    pub fn send(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = self.hash.clone();
        buffer.append(&mut self.index.to_be_bytes().to_vec());

        Ok(buffer)
    }

    pub fn read(buffer: &Vec<u8>) -> Outpoint {
        let hash = buffer[0..32].to_vec();
        let mut index = buffer[32..36].to_vec();
        index.reverse();
        let index = deserialize(&index).unwrap();

        Outpoint {
            hash: hash,
            index: index
        }
    }
}
impl Size for Outpoint {
    fn size(&self) -> u64 {
        32 + 4
    }
}

#[derive(Debug, Clone)]
pub struct TxIn {
    pub previous_output: Outpoint,
    pub script: VarStr,
    pub shash: Vec<u8>,
}

impl TxIn {
    pub fn send(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = self.previous_output.send()?;
        buffer.append(&mut self.script.send());

        Ok(buffer)
    }

    pub fn read(buffer: &Vec<u8>) -> TxIn {
        let previous_output = Outpoint::read(buffer);
        let script = VarStr::new(&buffer[previous_output.size() as usize..].to_vec());

        TxIn {
            previous_output: previous_output,
            script: script,
            shash:  Vec::new()
        }
    }
}
impl Size for TxIn {
    fn size(&self) -> u64 {
        self.previous_output.size() + self.script.size()
    }
}

#[derive(Debug, Clone)]
pub struct TxOut {
    pub value: u64,
    pub script: VarStr,
}

impl TxOut {
    pub fn send(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.append(&mut self.value.to_be_bytes().to_vec());
        buffer.append(&mut self.script.send());

        buffer
    }

    pub fn read(buffer: &Vec<u8>) -> TxOut {
        let mut value = buffer[0..8].to_vec();
        value.reverse();
        let value = deserialize(&value).unwrap();

        let script = VarStr::new(&buffer[8..].to_vec());

        TxOut {
            value: value,
            script: script
        }
    }
}
impl Size for TxOut {
    fn size(&self) -> u64 {
        8 + self.script.size()
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub version: u32,
    pub flags_count: VarUint,
    pub flags: Vec<VarStr>,
    pub inputs_count: VarUint,
    pub inputs: Vec<TxIn>,
    pub outputs_count: VarUint,
    pub outputs: Vec<TxOut>,
}

impl Transaction {

    pub fn generic_shash_part(&mut self) -> Vec<u8> {
        let mut hash_vec = Vec::new();

        let mut version = serialize(&self.version).unwrap();
        version.reverse();
        hash_vec.append(&mut version);

        let mut flags_count = serialize(&self.flags_count.value).unwrap();
        flags_count.reverse();
        hash_vec.append(&mut flags_count);

        for s in &self.flags {
            hash_vec.append(&mut serialize(&s.value).unwrap());
        }
        for i in &self.inputs {
            let mut outpoint_hash = ::utils::hash::hash(i.previous_output.send().unwrap());
            outpoint_hash = ::utils::hash::hash(outpoint_hash);
            hash_vec.append(&mut outpoint_hash);
        }

        hash_vec
    }

    pub fn is_sane(&self) -> bool {
        if self.inputs_count.value == 0 && self.outputs_count.value == 0 {
            return false;
        }

        for txo in &self.outputs {
            if txo.value == 0 {
                return false;
            }
        }

        true
    }

    pub fn hash(&self) -> Result<Vec<u8>, Error> {
        let buffer = self.send()?;

        Ok(hash::hash(hash::hash(buffer)))
    }

    pub fn send(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        let mut version = serialize(&self.version)?;
        version.reverse();
        buffer.append(&mut version);

        buffer.append(&mut self.flags_count.send());
        for flag in &self.flags {
            buffer.append(&mut flag.send());
        }

        buffer.append(&mut self.inputs_count.send());
        for input in &self.inputs {
            buffer.append(&mut input.send()?);
        }

        buffer.append(&mut self.outputs_count.send());
        for output in &self.outputs {
            buffer.append(&mut output.send());
        }

        Ok(buffer)
    }

    pub fn read(buffer: &Vec<u8>) -> Transaction {
        let mut version = buffer[0..4].to_vec();
        version.reverse();
        let version = deserialize(&version).unwrap();

        let flags_count = VarUint::new(&buffer[4..].to_vec());
        let mut offset : usize = 4 + flags_count.size() as usize;

        let mut flags = Vec::new();
        for _ in 0..flags_count.value {
            let flag = VarStr::new(&buffer[offset..].to_vec());
            offset += flag.size() as usize;
            flags.push(flag);
        }
        let inputs_count = VarUint::new(&buffer[offset..].to_vec());
        offset += inputs_count.size() as usize;

        let mut inputs = Vec::new();
        for _ in 0..inputs_count.value {
            let input = TxIn::read(&buffer[offset..].to_vec());
            offset += input.size() as usize;
            inputs.push(input);
        }

        let outputs_count = VarUint::new(&buffer[offset..].to_vec());
        offset += outputs_count.size() as usize;

        let mut outputs = Vec::new();
        for _ in 0..outputs_count.value {
            let output = TxOut::read(&buffer[offset..].to_vec());
            offset += output.size() as usize;
            outputs.push(output);
        }

        Transaction {
            version: version,
            flags_count: flags_count,
            flags: flags,
            inputs_count: inputs_count,
            inputs: inputs,
            outputs_count: outputs_count,
            outputs: outputs,
        }
    }
}
impl Size for Transaction {
    fn size(&self) -> u64 {
        let mut s = 4 + self.flags_count.size();
        for f in &self.flags {
            s += f.size();
        }
        s += self.inputs_count.size();
        for i in &self.inputs {
            s += i.size();
        }
        s += self.outputs_count.size();
        for o in &self.outputs {
            s += o.size();
        }
        s as u64
    }
}

pub struct TxTxo<'a> {
    pub tx      : &'a Transaction,
    pub txos    : Vec<TxOut>,
}
impl<'a> TxTxo<'a> {
    pub fn new(tx: &'a Transaction, txos: Vec<TxOut>) -> TxTxo {
        TxTxo {
            tx: tx,
            txos: txos,
        }
    }

    pub fn is_valid(&self) -> bool {
        if !self.tx.is_sane() {
            return false;
        }

        let mut entry_sum = 0;
        for entry in &self.txos {
            entry_sum += entry.value;
        }
        let mut output_sum = 0;
        for output in &self.tx.outputs {
            output_sum += output.value;
        }

        if output_sum >= entry_sum {
            return false;
        }

        //check is tx exists in main chain
        //check all txos if said output is not already in some tx input in mainchain
        true
    }
}
