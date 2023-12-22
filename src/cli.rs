use crate::transaction;
use crate::wallet;
use crate::wallets;
use crate::Blockchain;
use crate::BlockchainIterator;
use crate::ProofOfWork;
use crate::Transaction;

pub struct Cli {}

impl Cli {
    fn print_usage() {
        println!("Usage:");
        println!(" getbalance -address ADDRESS - Get balance of ADDRESS");
        println!(" createblockchain -address ADDRESS - Create a blockchain and send genesis block reward to ADDRESS");
        println!(" createwallet - Generates a new key-pair and saves it into the wallet file");
        println!(" listaddresses - Lists all addresses from the wallet file");
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
                    println!("Prev. block: {}", hex::encode(block.get_prev_block_hash()));
                    println!("Hash: {}", hex::encode(block.get_hash()));
                    let pow = ProofOfWork::new_proof_of_work(block.clone());
                    println!("PoW: {}", pow.validate());
                    for tx in block.get_transactions() {
                        Cli::print_transaction(&tx);
                    }
                    println!();
                }
                None => break,
            }
        }
    }

    pub fn print_transaction(tx: &Transaction) {
        for tx_in in tx.get_vin() {
            println!("TXInput:");
            println!("  TXID: {}", hex::encode(tx_in.get_txid()));
            println!("  Out: {}", tx_in.get_vout());
            let pub_key_hash = wallet::hash_pub_key(&tx_in.get_pub_key());
            println!("address:{}", wallet::calc_address(&pub_key_hash));
        }

        for tx_out in tx.get_vout() {
            println!("TXOutput:");
            println!("  TXID: {}", hex::encode(tx.get_id()));
            println!("  Value: {}", tx_out.get_value());
            let pub_key_hash = tx_out.get_pub_key_hash();
            println!("address:{}", wallet::calc_address(&pub_key_hash));
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
            "createwallet" => {
                Cli::create_wallet();
            }
            "listaddresses" => {
                Cli::list_address();
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
        if !wallet::validate_address(address.clone()) {
            eprintln!("Error: Address is not valid");
            std::process::exit(1);
        }
        let bc = Blockchain::create_blockchain(address.clone());

        let mut pub_key_hash = bs58::decode(&address).into_vec().unwrap();
        pub_key_hash = pub_key_hash[1..pub_key_hash.len() - wallet::ADDRESS_CHECK_SUM_LEN].to_vec();
        let utxo = bc.find_utxo(&pub_key_hash);

        let mut balance = 0;
        for out in utxo {
            balance += out.get_value();
        }
        println!("Balance of '{}' : {}", address, balance);
    }

    pub fn create_wallet() {
        let mut wallets = wallets::new_wallets();
        let address = wallets.create_wallet();
        wallets.save_to_file();
        println!("Your new address is: {}", address);
    }

    pub fn list_address() {
        let wallets = wallets::new_wallets();
        let addresses = wallets.get_addresses();
        for address in addresses {
            println!("{}", address);
        }
    }

    pub fn send(from: String, to: String, amount: i64) {
        if !wallet::validate_address(from.clone()) {
            eprintln!("Error: sender Address is not valid");
            std::process::exit(1);
        }
        if !wallet::validate_address(to.clone()) {
            eprintln!("Error: receiver Address is not valid");
            std::process::exit(1);
        }

        let mut blockchain = Blockchain::create_blockchain(from.clone());
        let transaction =
            transaction::new_utxo_transaction(from.clone(), to.clone(), amount, &blockchain);
        blockchain.mine_block(vec![transaction]);
        println!("Success!");
    }
}
