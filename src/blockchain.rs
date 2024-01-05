use crate::transaction;
use crate::Block;
use crate::TXOutput;
use crate::Transaction;
use sled::Db;
use std::collections::HashMap;

const DB_FILE: &str = "blockchain_{}.db";
const TIP_BLOCK_HASH: &str = "blocks"; // key for the last block hash
const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

#[derive(Clone)]
pub struct Blockchain {
    tip: Vec<u8>, // last block hash
    db: Db,
}

impl Blockchain {
    pub fn get_db(&self) -> &Db {
        &self.db
    }
    pub fn create_blockchain(address: String, node_id: String) -> Blockchain {
        let path = DB_FILE.replace("{}", &node_id);
        let db = sled::open(path).expect("open");
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

    pub fn new_blockchain(node_id: String) -> Result<Blockchain, String> {
        let path = DB_FILE.replace("{}", &node_id);
        let db = sled::open(path).expect("open");
        let data = db.get(TIP_BLOCK_HASH).unwrap();
        let tip;
        if data.is_none() {
            return Err("No existing blockchain found. Create one first.".to_string());
        } else {
            tip = data.unwrap().to_vec();
        }
        Ok(Blockchain { tip, db })
    }

    // returns the height of the latest block
    pub fn get_best_height(&self) -> usize {
        let block_hash = self.db.get(TIP_BLOCK_HASH).unwrap();
        let data = self.db.get(block_hash.unwrap().to_vec()).unwrap();
        let tip_block = Block::deserialize_block(data.unwrap().to_vec());
        tip_block.get_height()
    }

    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Block {
        for tx in &transactions {
            if !self.verify_transaction(tx) {
                panic!("ERROR: Invalid transaction");
            }
        }
        let best_height = self.get_best_height();
        let block = Block::new_block(transactions, self.tip.clone(), best_height + 1);
        let block_hash = block.get_hash();
        self.db
            .insert(block_hash.clone(), block.serialize())
            .unwrap();
        self.db.insert(TIP_BLOCK_HASH, block_hash.clone()).unwrap();
        self.tip = block_hash;
        block
    }

    pub fn add_block(&mut self, block: Block) {
        let block_hash = block.get_hash();
        if let Some(_) = self.db.get(block_hash.clone()).unwrap() {
            return;
        }
        println!("Added block {} to the blockchain", hex::encode(&block_hash));
        self.db
            .insert(block_hash.clone(), block.serialize())
            .unwrap();
        let tip_block_hash = self.db.get(TIP_BLOCK_HASH).unwrap();
        let tip_block = self.db.get(tip_block_hash.unwrap().to_vec()).unwrap();
        let last_block = Block::deserialize_block(tip_block.unwrap().to_vec());
        if block.get_height() > last_block.get_height() {
            self.db.insert(TIP_BLOCK_HASH, block_hash.clone()).unwrap();
            self.tip = block_hash;
        }
    }

    pub fn get_block(&self, block_hash: &Vec<u8>) -> Option<Block> {
        let data = self.db.get(block_hash).unwrap();
        if data.is_none() {
            return None;
        }
        Some(Block::deserialize_block(data.unwrap().to_vec()))
    }

    pub fn get_block_hashes(&self) -> Vec<Vec<u8>> {
        let mut blocks: Vec<Vec<u8>> = vec![];
        let mut blockchain_iterator = BlockchainIterator {
            current_hash: self.tip.clone(),
            db: self.db.clone(),
        };
        while let Some(block) = blockchain_iterator.next() {
            blocks.push(block.get_hash());
        }
        blocks
    }

    // finds all unspent transaction outputs and returns transactions with spent outputs removed
    pub fn find_utxo(&self) -> HashMap<String, Vec<TXOutput>> {
        let mut utxo: HashMap<String, Vec<TXOutput>> = HashMap::new();

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
                                println!("continue");
                                continue 'Outputs;
                            }
                        }
                    }

                    utxo.entry(txid.clone())
                        .or_insert(vec![])
                        .push(tx_output.clone());
                }

                if tx.is_coinbase() == false {
                    for tx_input in tx.get_vin() {
                        let tx_input_id = hex::encode(tx_input.get_txid().clone());

                        spent_txos
                            .entry(tx_input_id)
                            .or_insert(vec![])
                            .push(tx_input.get_vout());
                    }
                }
            }
        }
        utxo
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
        if tx.is_coinbase() {
            return true;
        }
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
