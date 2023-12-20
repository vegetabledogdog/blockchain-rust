use crate::Block;
use num_bigint::BigUint;
use sha2::{Digest, Sha256};
use std::ops::ShlAssign;

// requirement: first 24 bits of hash must be 0
//const TARGET_BITS: i32 = 24;
// for debug purpose, we set the target to 4 bits
const TARGET_BITS: i32 = 4;
const MAX_NONCE: i64 = i64::max_value(); //avoid a possible overflow of nonce

pub struct ProofOfWork {
    block: Block,
    target: BigUint,
}

impl ProofOfWork {
    pub fn new_proof_of_work(block: Block) -> ProofOfWork {
        /*  a target as the upper boundary of a range:
        if a number (a hash) is lower than the boundary, it’s valid, and vice versa. */
        let mut target = BigUint::from(1u32);
        target.shl_assign(256 - TARGET_BITS);
        ProofOfWork { block, target }
    }

    // nonce here is the counter from the Hashcash description
    fn prepare_data(&self, nonce: i64) -> Vec<u8> {
        let mut data = vec![];
        data.extend(self.block.get_prev_block_hash());
        data.extend(self.block.hash_transactions());
        data.extend(self.block.get_timestamp().to_be_bytes());
        data.extend(TARGET_BITS.to_be_bytes());
        data.extend(nonce.to_be_bytes());
        data
    }

    pub fn run(&self) -> (i64, Vec<u8>) {
        let mut nonce = 0i64;
        let mut hash = Vec::new();
        let mut hasher = Sha256::new();

        println!("Mining a new block");

        while nonce < MAX_NONCE {
            let data = self.prepare_data(nonce);

            hash.clear();
            hasher.update(data);
            hash = hasher.finalize_reset().to_vec();
            print!("\r{}", hex::encode(&hash));

            let hash_int = BigUint::from_bytes_be(&hash);

            // if hash_int < target, we find a valid hash
            if hash_int.lt(&self.target) {
                break;
            } else {
                nonce += 1;
            }
        }
        println!("\n");
        (nonce, hash)
    }

    pub fn validate(&self) -> bool {
        let data = self.prepare_data(self.block.get_nounce());
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize().to_vec();
        let hash_int = BigUint::from_bytes_be(&hash);

        hash_int.lt(&self.target)
    }
}
