use blockchain_rust::Blockchain;

fn main() {
    let mut bc = Blockchain::new_blockchain();
    bc.add_block("Send 1 BTC to Ivan".as_bytes().to_vec());
    bc.add_block("Send 2 more BTC to Ivan".as_bytes().to_vec());

    for block in bc.get_blocks() {
        println!("Prev. hash: {}", String::from_utf8(block.get_prev_block_hash()).unwrap());
        println!("Data: {}", String::from_utf8(block.get_data()).unwrap());
        println!("Hash: {}\n", String::from_utf8(block.get_hash()).unwrap());
    }
}
