use std::collections::HashMap;
use blockchain::transaction::*;
use blockchain::Utxos;
use utils::Error;

#[derive(Debug)]
pub struct Mempool {
    pub txs                 : HashMap<Vec<u8>, Transaction>,
    pub orphans             : HashMap<Vec<u8>, Transaction>,
    pub outpoints           : HashMap<Vec<u8>, Outpoint>,
    pub orphans_outpoints   : HashMap<Vec<u8>, Outpoint>
}

impl Default for Mempool {
    fn default() -> Self {
        Self {
            txs                 : HashMap::new(),
            orphans             : HashMap::new(),
            outpoints           : HashMap::new(),
            orphans_outpoints   : HashMap::new(),
        }
    }
}

impl Mempool {
    pub fn new() -> Self {
        Self::default()
    }

    // tx is not in self.txs and not in self.orphans
    pub fn add_tx(&mut self, tx: &Transaction) -> Result<(), Error> {
        let mut utxos = Vec::new();
        for input in &tx.inputs {
            if !self.txs.contains_key(&input.previous_output.hash) {
                self.orphans.insert(tx.hash()?, tx.clone());
                self.orphans_outpoints.insert(input.previous_output.hash.clone(), input.previous_output.clone());
            }
            let mut utxo = Utxos::get_utxos(input.previous_output.hash.clone())?;
            if utxo.is_empty() {
                return Err(Error::TxNotValid)
            } else {
                utxos.append(&mut utxo);
            }
        }
        //valid tx
        let txto = TxTxo::new(tx, utxos);
        if !txto.is_valid() {
        }

        self.txs.insert(tx.hash()?, tx.clone());
        // check if TxOut of tx are in orphans
        Ok(())
    }

    pub fn contains_tx(&self, hash: Vec<u8>) -> bool {
        self.txs.contains_key(&hash) || self.orphans.contains_key(&hash)
    }
}
