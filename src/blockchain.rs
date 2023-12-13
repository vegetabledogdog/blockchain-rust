use crate::Block;

pub struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    pub fn add_block(&mut self, data: Vec<u8>) {
        let prev_block = self.blocks.last().unwrap();
        let new_block = Block::new_block(data, prev_block.get_hash());
        self.blocks.push(new_block);
    }

    fn new_genesis_block() -> Block {
        Block::new_block("Genesis Block".as_bytes().to_vec(), vec![])
    }

    pub fn new_blockchain() -> Blockchain {
        let blockchain = Blockchain {
            blocks: vec![Blockchain::new_genesis_block()],
        };
        blockchain
    }

    pub fn get_blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
}
