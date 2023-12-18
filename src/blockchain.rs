use crate::Block;
use sled::Db;

const DB_FILE: &str = "blockchain.db";
const TIP_BLOCK_HASH: &str = "blocks"; // key for the last block hash

pub struct Blockchain {
    tip: Vec<u8>, // last block hash
    db: Db,
}

impl Blockchain {
    pub fn add_block(&mut self, data: Vec<u8>) {
        let block = Block::new_block(data, self.tip.clone());
        let block_hash = block.get_hash();
        self.db
            .insert(block_hash.clone(), block.serialize())
            .unwrap();
        self.db.insert(TIP_BLOCK_HASH, block_hash.clone()).unwrap();
        self.tip = block_hash;
    }

    pub fn new_blockchain() -> Blockchain {
        let db = sled::open(DB_FILE).expect("open");
        let data = db.get(TIP_BLOCK_HASH).unwrap();
        let tip;
        if data.is_none() {
            let genesis = Block::new_genesis_block();
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
