use std::time::SystemTime;
use sha256::digest;

pub struct Block {
    timestamp: i64, // current timestamp(when the block is created)
    data: Vec<u8>,
    prev_block_hash: Vec<u8>, // Hash of the previous block
    hash: Vec<u8>,            // block headers, Hash of the current block
}

impl Block {
    // calculates and sets block hash
    pub fn set_hash(&mut self) {
        let timestamp = self.timestamp.to_string().into_bytes();
        let headers = [self.prev_block_hash.clone(), self.data.clone(), timestamp].concat();
        let hash = digest(&headers);

        self.hash = hash.into_bytes();
    }

    pub fn new_block(data: Vec<u8>, prev_block_hash: Vec<u8>) -> Block {
        let mut block = Block {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            data,
            prev_block_hash,
            hash: vec![],
        };
        block.set_hash();
        block
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
}
