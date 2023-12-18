use crate::Blockchain;
use crate::BlockchainIterator;
use crate::ProofOfWork;

pub struct Cli {
    blockchain: Blockchain,
}

impl Cli {
    pub fn new_cli(blockchain: Blockchain) -> Cli {
        Cli { blockchain }
    }

    fn print_usage() {
        println!("Usage:");
        println!("  addblock -data BLOCK_DATA - add a block to the blockchain");
        println!("  printchain - print all the blocks of the blockchain");
    }

    fn validate_args() {
        if std::env::args().len() < 2 {
            Cli::print_usage();
            std::process::exit(1);
        }
    }

    fn add_block(&mut self, data: Vec<u8>) {
        self.blockchain.add_block(data);
        println!("Success!");
    }

    fn print_chain(&self) {
        let mut blockchain_iter = BlockchainIterator::iterator(&self.blockchain);
        loop {
            let block = blockchain_iter.next();
            match block {
                Some(block) => {
                    println!("Prev. hash: {}", hex::encode(block.get_prev_block_hash()));
                    println!("Data: {}", String::from_utf8(block.get_data()).unwrap());
                    println!("Hash: {}", hex::encode(block.get_hash()));
                    let pow = ProofOfWork::new_proof_of_work(block.clone());
                    println!("PoW: {}\n", pow.validate());
                    println!();
                }
                None => break,
            }
        }
    }

    pub fn run(&mut self) {
        Cli::validate_args();

        let args: Vec<String> = std::env::args().collect();
        match args[1].as_str() {
            "addblock" => {
                if args.len() != 4 {
                    Cli::print_usage();
                    std::process::exit(1);
                }
                self.add_block(args[3].as_bytes().to_vec());
            }
            "printchain" => {
                self.print_chain();
            }
            _ => {
                Cli::print_usage();
                std::process::exit(1);
            }
        }
    }
}
