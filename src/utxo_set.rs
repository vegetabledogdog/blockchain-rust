use crate::Block;
use crate::Blockchain;
use crate::TXOutput;
use std::collections::HashMap;

const UTXO_TREE: &str = "chainstate";

pub struct UtxoSet {
    blockchain: Blockchain,
}

impl UtxoSet {
    pub fn new(blockchain: Blockchain) -> UtxoSet {
        UtxoSet { blockchain }
    }

    // rebuilds the UTXO set
    pub fn reindex(&self) {
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).unwrap();
        utxo_tree.clear().unwrap();

        let utxo_map = self.blockchain.find_utxo();
        for (tx_hex, outs) in &utxo_map {
            let txid = hex::decode(tx_hex).unwrap();
            let value = bincode::serialize(outs).unwrap();
            utxo_tree.insert(txid, value).unwrap();
        }
    }

    // finds and returns unspent outputs to reference in inputs
    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: &Vec<u8>,
        amount: i64,
    ) -> (i64, HashMap<String, Vec<usize>>) {
        let mut unspent_outputs: HashMap<String, Vec<usize>> = HashMap::new();
        let mut accumulated = 0;
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).unwrap();

        for item in utxo_tree.iter() {
            let (k, v) = item.unwrap();
            let txid = hex::encode(k);
            let outs: Vec<TXOutput> = bincode::deserialize(&v).unwrap();
            for (idx, out) in outs.iter().enumerate() {
                if out.is_locked_with_key(pub_key_hash) && accumulated < amount {
                    accumulated += out.get_value();
                    unspent_outputs
                        .entry(txid.clone())
                        .or_insert(Vec::new())
                        .push(idx);
                }
            }
        }

        (accumulated, unspent_outputs)
    }

    // finds UTXO for a public key hash
    pub fn find_utxo(&self, pub_key_hash: &Vec<u8>) -> Vec<TXOutput> {
        let mut utxo: Vec<TXOutput> = Vec::new();
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).unwrap();

        for item in utxo_tree.iter() {
            let (_, v) = item.unwrap();
            let outs: Vec<TXOutput> = bincode::deserialize(&v).unwrap();
            for out in outs {
                if out.is_locked_with_key(pub_key_hash) {
                    utxo.push(out);
                }
            }
        }

        utxo
    }

    /*updates the UTXO set with transactions from the Block
    The Block is considered to be the tip of a blockchain */
    pub fn update(&self, block: Block) {
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).unwrap();

        for tx in block.get_transactions() {
            if !tx.is_coinbase() {
                for vin in tx.get_vin() {
                    let mut updated_outs: Vec<TXOutput> = Vec::new();
                    let outs_bytes = utxo_tree.get(vin.get_txid()).unwrap().unwrap();
                    let outs: Vec<TXOutput> = bincode::deserialize(&outs_bytes).unwrap();
                    for (out_idx, out) in outs.iter().enumerate() {
                        if out_idx != vin.get_vout() as usize {
                            updated_outs.push(out.clone());
                        }
                    }
                    if updated_outs.len() == 0 {
                        utxo_tree.remove(vin.get_txid()).unwrap();
                    } else {
                        let outs_bytes = bincode::serialize(&updated_outs).unwrap();
                        utxo_tree.insert(vin.get_txid(), outs_bytes).unwrap();
                    }
                }
            }

            let mut new_outputs: Vec<TXOutput> = Vec::new();
            for out in tx.get_vout() {
                new_outputs.push(out.clone());
            }
            let outs_bytes = bincode::serialize(&new_outputs).unwrap();
            utxo_tree.insert(tx.get_id(), outs_bytes).unwrap();
        }
    }

    pub fn get_blockchain(&self) -> &Blockchain {
        &self.blockchain
    }

    pub fn count_transactions(&self) -> i32 {
        let db = self.blockchain.get_db();
        let utxo_tree = db.open_tree(UTXO_TREE).unwrap();
        let mut count = 0;
        for _ in utxo_tree.iter() {
            count += 1;
        }
        count
    }
}
