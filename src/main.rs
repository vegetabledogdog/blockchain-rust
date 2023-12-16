use blockchain_rust::Blockchain;
use blockchain_rust::ProofOfWork;

fn main() {
    let mut bc = Blockchain::new_blockchain();
    bc.add_block("Send 1 BTC to Ivan".as_bytes().to_vec());
    bc.add_block("Send 2 more BTC to Ivan".as_bytes().to_vec());

    for block in bc.get_blocks() {
        println!(
            "Prev. hash: {}",
            hex::encode(block.get_hash())
        );
        println!("Data: {}", String::from_utf8(block.get_data()).unwrap());
        println!("Hash: {}", hex::encode(block.get_hash()));
        let pow = ProofOfWork::new_proof_of_work(block.clone());
        println!("PoW: {}\n", pow.validate());
    }
}
