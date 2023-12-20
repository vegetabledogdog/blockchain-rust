use crate::transaction;
use crate::Blockchain;
use crate::BlockchainIterator;
use crate::ProofOfWork;

pub struct Cli {}

impl Cli {
    fn print_usage() {
        println!("Usage:");
        println!(" getbalance -address ADDRESS - Get balance of ADDRESS");
        println!(" createblockchain -address ADDRESS - Create a blockchain and send genesis block reward to ADDRESS");
        println!(" printchain - Print all the blocks of the blockchain");
        println!(
            " send -from FROM -to TO -amount AMOUNT - Send AMOUNT of coins from FROM address to TO"
        );
    }

    fn validate_args() {
        if std::env::args().len() < 2 {
            Cli::print_usage();
            std::process::exit(1);
        }
    }

    fn print_chain(&self) {
        let bc = Blockchain::create_blockchain("".to_string());
        let mut blockchain_iter = BlockchainIterator::iterator(&bc);
        loop {
            let block = blockchain_iter.next();
            match block {
                Some(block) => {
                    println!("Prev. hash: {}", hex::encode(block.get_prev_block_hash()));
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
            "getbalance" => {
                if args.len() != 4 {
                    println!("Usage: getbalance -address ADDRESS");
                    std::process::exit(1);
                }
                Cli::get_balance(args[3].clone());
            }
            "createblockchain" => {
                if args.len() != 4 {
                    println!("Usage: createblockchain -address ADDRESS");
                    std::process::exit(1);
                }
                Blockchain::create_blockchain(args[3].clone());
                println!("Done!");
            }
            "printchain" => {
                self.print_chain();
            }
            "send" => {
                if args[3].is_empty() || args[5].is_empty() || args[7].is_empty() {
                    println!("Usage: send -from FROM -to TO -amount AMOUNT - Send AMOUNT of coins from FROM address to TO");
                }
                Cli::send(
                    args[3].clone(),
                    args[5].clone(),
                    args[7].parse::<i64>().unwrap(),
                );
            }
            _ => {
                Cli::print_usage();
                std::process::exit(1);
            }
        }
    }

    pub fn get_balance(address: String) {
        let bc = Blockchain::create_blockchain(address.clone());
        let utxo = bc.find_utxo(address.clone());
        let mut balance = 0;
        for out in utxo {
            balance += out.get_value();
        }
        println!("Balance of {}: {}", address, balance);
    }

    pub fn send(from: String, to: String, amount: i64) {
        let mut blockchain = Blockchain::create_blockchain(from.clone());
        let transaction =
            transaction::new_utxo_transaction(from.clone(), to.clone(), amount, &blockchain);
        blockchain.mine_block(vec![transaction]);
        println!("Success!");
    }
}
