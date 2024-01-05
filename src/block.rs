use crate::merkle_tree;
use crate::ProofOfWork;
use crate::Transaction;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    timestamp: i64,                 // current timestamp(when the block is created)
    transactions: Vec<Transaction>, // transactions
    prev_block_hash: Vec<u8>,       // Hash of the previous block
    hash: Vec<u8>,                  // block headers, Hash of the current block
    nonce: i64,                     // counter
    height: usize,                  // block height
}

impl Block {
    pub fn new_block(
        transactions: Vec<Transaction>,
        prev_block_hash: Vec<u8>,
        height: usize,
    ) -> Block {
        let mut block = Block {
            timestamp: Local::now().timestamp_millis(),
            transactions,
            prev_block_hash,
            hash: vec![],
            nonce: 0,
            height,
        };
        let pow = ProofOfWork::new_proof_of_work(block.clone());
        (block.nonce, block.hash) = pow.run();
        block
    }

    pub fn new_genesis_block(coinbase: Vec<Transaction>) -> Block {
        Block::new_block(coinbase, vec![], 0)
    }
    /*We want all transactions in a block to be uniquely identified
    by a single hash. To achieve this, we get hashes of each transaction,
    concatenate them, and get a hash of the concatenated combination. */
    pub fn hash_transactions(&self) -> Vec<u8> {
        let mut transactions = vec![];
        for tx in self.transactions.clone() {
            transactions.push(bincode::serialize(&tx).unwrap());
        }
        let m_tree = merkle_tree::new_merkle_tree(transactions);
        m_tree.root_node.data
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn deserialize_block(data: Vec<u8>) -> Block {
        bincode::deserialize(&data).unwrap()
    }

    pub fn get_prev_block_hash(&self) -> Vec<u8> {
        self.prev_block_hash.clone()
    }

    pub fn get_hash(&self) -> Vec<u8> {
        self.hash.clone()
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.transactions.clone()
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_nounce(&self) -> i64 {
        self.nonce
    }

    pub fn get_height(&self) -> usize {
        self.height
    }
}
