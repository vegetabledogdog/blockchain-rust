use crate::transaction;
use crate::Block;
use crate::TXOutput;
use crate::Transaction;
use sled::Db;
use std::collections::HashMap;

const DB_FILE: &str = "blockchain.db";
const TIP_BLOCK_HASH: &str = "blocks"; // key for the last block hash
const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

pub struct Blockchain {
    tip: Vec<u8>, // last block hash
    db: Db,
}

impl Blockchain {
    pub fn create_blockchain(address: String) -> Blockchain {
        let db = sled::open(DB_FILE).expect("open");
        let data = db.get(TIP_BLOCK_HASH).unwrap();
        let tip;
        if data.is_none() {
            println!("No existing blockchain found. Creating a new one...");
            let coinbase = transaction::new_coinbase_tx(address, GENESIS_COINBASE_DATA.to_string());
            let genesis = Block::new_genesis_block(vec![coinbase]);
            let genesis_hash = genesis.get_hash();
            db.insert(genesis_hash.clone(), genesis.serialize())
                .unwrap();
            db.insert(TIP_BLOCK_HASH, genesis_hash.clone()).unwrap();
            tip = genesis_hash;
        } else {
            tip = data.unwrap().to_vec();
        }
        Blockchain { tip, db }
    }

    pub fn mine_block(&mut self, transactions: Vec<Transaction>) {
        for tx in &transactions {
            if !self.verify_transaction(tx) {
                panic!("ERROR: Invalid transaction");
            }
        }
        let block = Block::new_block(transactions, self.tip.clone());
        let block_hash = block.get_hash();
        self.db
            .insert(block_hash.clone(), block.serialize())
            .unwrap();
        self.db.insert(TIP_BLOCK_HASH, block_hash.clone()).unwrap();
        self.tip = block_hash;
    }

    pub fn find_unspent_transactions(&self, pub_key_hash: &Vec<u8>) -> Vec<Transaction> {
        let mut unspent_txs: Vec<Transaction> = vec![];

        // spend transaction outputs
        // transaction id -> transaciton vout index
        let mut spent_txos: HashMap<String, Vec<i64>> = HashMap::new();

        let mut blockchain_iterator = BlockchainIterator {
            current_hash: self.tip.clone(),
            db: self.db.clone(),
        };
        while let Some(block) = blockchain_iterator.next() {
            for tx in block.get_transactions() {
                let txid = hex::encode(tx.get_id());

                'Outputs: for (tx_output_index, tx_output) in tx.get_vout().iter().enumerate() {
                    if let Some(spent_txo) = spent_txos.get(&txid) {
                        for spent_out in spent_txo {
                            if spent_out.clone() == tx_output_index as i64 {
                                println!("contimue");
                                continue 'Outputs;
                            }
                        }
                    }

                    if tx_output.is_locked_with_key(pub_key_hash) {
                        unspent_txs.push(tx.clone());
                    }

                    if tx.is_coinbase() == false {
                        for tx_input in tx.get_vin() {
                            if tx_input.uses_key(pub_key_hash) {
                                let tx_input_id = hex::encode(tx_input.get_txid().clone());

                                spent_txos
                                    .entry(tx_input_id)
                                    .or_insert(vec![])
                                    .push(tx_input.get_vout());
                            }
                        }
                    }
                }
            }
        }
        unspent_txs
    }

    // finds and returns all unspent transaction outputs
    pub fn find_utxo(&self, pub_key_hash: &Vec<u8>) -> Vec<TXOutput> {
        let mut utxos: Vec<TXOutput> = vec![];
        let unspent_transactions = self.find_unspent_transactions(pub_key_hash);

        for tx in unspent_transactions {
            for out in tx.get_vout() {
                if out.is_locked_with_key(pub_key_hash) {
                    utxos.push(out);
                }
            }
        }

        utxos
    }

    // find all unspent outputs and ensure that they store enough value
    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: &Vec<u8>,
        amount: i64,
    ) -> (i64, HashMap<String, Vec<i64>>) {
        let mut unspent_outputs: HashMap<String, Vec<i64>> = HashMap::new();
        // find fn is wrong sometimes
        let unspent_transactions = self.find_unspent_transactions(pub_key_hash);
        let mut accumulated = 0;

        'Work: for unspent_transaction in unspent_transactions {
            let txid = hex::encode(unspent_transaction.get_id());

            for (tx_output_index, tx_output) in unspent_transaction.get_vout().iter().enumerate() {
                if tx_output.is_locked_with_key(pub_key_hash) && accumulated < amount {
                    accumulated += tx_output.get_value();

                    unspent_outputs
                        .entry(txid.clone())
                        .or_insert(Vec::new())
                        .push(tx_output_index as i64);
                }

                if accumulated >= amount {
                    break 'Work;
                }
            }
        }

        (accumulated, unspent_outputs)
    }

    pub fn sign_transaction(&self, tx: &mut Transaction, private_key: &Vec<u8>) {
        let mut prev_txs: HashMap<String, Transaction> = HashMap::new();
        for vin in &tx.get_vin() {
            let prev_tx = self.find_transaction(vin.get_txid());
            prev_txs.insert(hex::encode(prev_tx.get_id()), prev_tx);
        }
        tx.sign(private_key, &prev_txs);
    }

    pub fn verify_transaction(&self, tx: &Transaction) -> bool {
        let mut prev_txs: HashMap<String, Transaction> = HashMap::new();
        for vin in &tx.get_vin() {
            let prev_tx = self.find_transaction(vin.get_txid());
            prev_txs.insert(hex::encode(prev_tx.get_id()), prev_tx);
        }
        tx.verify(&prev_txs)
    }

    pub fn find_transaction(&self, id: Vec<u8>) -> Transaction {
        let mut blockchain_iterator = BlockchainIterator {
            current_hash: self.tip.clone(),
            db: self.db.clone(),
        };
        while let Some(block) = blockchain_iterator.next() {
            for tx in block.get_transactions() {
                if tx.get_id() == id {
                    return tx;
                }
            }
        }
        panic!("Transaction does not exist")
    }
}

pub struct BlockchainIterator {
    current_hash: Vec<u8>,
    db: Db,
}

impl BlockchainIterator {
    pub fn iterator(blockchain: &Blockchain) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: blockchain.tip.clone(),
            db: blockchain.db.clone(),
        }
    }

    pub fn next(&mut self) -> Option<Block> {
        let data = self.db.get(self.current_hash.clone()).unwrap();
        if data.is_none() {
            return None;
        }
        let block = Block::deserialize_block(data.unwrap().to_vec());
        self.current_hash = block.get_prev_block_hash();
        Some(block)
    }
}
