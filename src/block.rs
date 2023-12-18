use crate::ProofOfWork;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    timestamp: i64, // current timestamp(when the block is created)
    data: Vec<u8>,
    prev_block_hash: Vec<u8>, // Hash of the previous block
    hash: Vec<u8>,            // block headers, Hash of the current block
    nonce: i64,               // counter
}

impl Block {
    pub fn new_block(data: Vec<u8>, prev_block_hash: Vec<u8>) -> Block {
        let mut block = Block {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            data,
            prev_block_hash,
            hash: vec![],
            nonce: 0,
        };
        let pow = ProofOfWork::new_proof_of_work(block.clone());
        (block.nonce, block.hash) = pow.run();
        block
    }

    pub fn new_genesis_block() -> Block {
        Block::new_block("Genesis Block".as_bytes().to_vec(), vec![])
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

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn get_hash(&self) -> Vec<u8> {
        self.hash.clone()
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_nounce(&self) -> i64 {
        self.nonce
    }
}
