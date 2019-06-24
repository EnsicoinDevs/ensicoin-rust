extern crate blockchain;
extern crate model;
extern crate utils;

use std::collections::HashMap;
use model::transaction::*;
use blockchain::Utxos;
use utils::Error;

pub struct Mempool {
    pub txs                 : HashMap<Vec<u8>, Transaction>,
    pub orphans             : HashMap<Vec<u8>, Transaction>,
    pub outpoints           : HashMap<Vec<u8>, Outpoint>,
    pub orphans_outpoints   : HashMap<Vec<u8>, Outpoint>
}
impl Mempool {
    pub fn new() -> Mempool {
        Mempool {
            txs                 : HashMap::new(),
            orphans             : HashMap::new(),
            outpoints           : HashMap::new(),
            orphans_outpoints   : HashMap::new()
        }
    }

    // tx is not in self.txs and not in self.orphans
    pub fn add_tx(&mut self, tx: &Transaction) -> Result<(), Error> {
        for input in &tx.inputs {
            if !self.txs.contains_key(&input.previous_output.hash) {
                self.orphans.insert(tx.hash()?, tx.clone());
                self.orphans_outpoints.insert(input.previous_output.hash.clone(), input.previous_output.clone());
            }
        }
        //valid tx
        let txto = TxTxo::new(tx, Utxos::get_utxos(tx.hash()?)?);
        if !txto.is_valid() {

        }

        self.txs.insert(tx.hash()?, tx.clone());
        // check if TxOut of tx are in orphans
        dbg!(&self.txs);
        dbg!(&self.orphans);
        Ok(())
    }

    pub fn contains_tx(&self, hash: Vec<u8>) -> bool {
        self.txs.contains_key(&hash) || self.orphans.contains_key(&hash)
    }
}
