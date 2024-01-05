use crate::server;
use crate::transaction;
use crate::utxo_set;
use crate::wallet;
use crate::wallets;
use crate::wallets::new_wallets;
use crate::Blockchain;
use crate::BlockchainIterator;
use crate::ProofOfWork;
use crate::Transaction;
use std::env;

pub struct Cli {}

impl Cli {
    fn print_usage() {
        println!("Usage:");
        println!(" getbalance -address ADDRESS - Get balance of ADDRESS");
        println!(" createblockchain -address ADDRESS - Create a blockchain and send genesis block reward to ADDRESS");
        println!(" createwallet - Generates a new key-pair and saves it into the wallet file");
        println!(" listaddresses - Lists all addresses from the wallet file");
        println!(" printchain - Print all the blocks of the blockchain");
        println!(" reindexutxo - Rebuilds the UTXO set");
        println!(
"  send -from FROM -to TO -amount AMOUNT -mine - Send AMOUNT of coins from FROM address to TO. Mine on the same node, when -mine is set."
        );
        println!(" startnode -miner ADDRESS - Start a node with ID specified in NODE_ID env. var. -miner enables mining");
    }

    fn validate_args() {
        if std::env::args().len() < 2 {
            Cli::print_usage();
            std::process::exit(1);
        }
    }

    fn print_chain(&self, node_id: String) {
        let bc = Blockchain::new_blockchain(node_id).unwrap();
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
        let node_id_var = env::var("NODE_ID");
        if node_id_var == Err(std::env::VarError::NotPresent) {
            println!("NODE_ID env is not set!");
            std::process::exit(1);
        }
        let node_id = node_id_var.unwrap();

        match args[1].as_str() {
            "getbalance" => {
                if args.len() != 4 {
                    println!("Usage: getbalance -address ADDRESS");
                    std::process::exit(1);
                }
                Cli::get_balance(args[3].clone(), node_id);
            }
            "createblockchain" => {
                if args.len() != 4 {
                    println!("Usage: createblockchain -address ADDRESS");
                    std::process::exit(1);
                }
                Cli::create_blockchain(args[3].clone(), node_id);
            }
            "createwallet" => {
                Cli::create_wallet(node_id);
            }
            "listaddresses" => {
                Cli::list_address(node_id);
            }
            "printchain" => {
                self.print_chain(node_id);
            }
            "send" => {
                if args[3].is_empty() || args[5].is_empty() || args[7].is_empty() {
                    println!("  send -from FROM -to TO -amount AMOUNT -mine");
                }
                let mine: bool;
                if args.len() == 9 && args[8] == "-mine" {
                    mine = true;
                } else {
                    mine = false;
                }
                Cli::send(
                    args[3].clone(),
                    args[5].clone(),
                    args[7].parse::<i64>().unwrap(),
                    node_id,
                    mine,
                );
            }
            "reindexutxo" => {
                Cli::reindex_utxo(node_id);
            }
            "startnode" => {
                let mut miner_address = String::new();
                if args.len() == 4 {
                    miner_address = args[3].clone();
                }
                Cli::start_node(node_id, miner_address);
            }
            _ => {
                Cli::print_usage();
                std::process::exit(1);
            }
        }
    }

    pub fn create_blockchain(address: String, node_id: String) {
        if !wallet::validate_address(address.clone()) {
            eprintln!("Error: Address is not valid");
            std::process::exit(1);
        }
        let bc = Blockchain::create_blockchain(address.clone(), node_id);
        let utxo_set = utxo_set::UtxoSet::new(bc);
        utxo_set.reindex();
        println!("Done!");
    }

    pub fn get_balance(address: String, node_id: String) {
        if !wallet::validate_address(address.clone()) {
            eprintln!("Error: Address is not valid");
            std::process::exit(1);
        }
        let bc = Blockchain::new_blockchain(node_id.clone()).unwrap();
        let utxo_set = utxo_set::UtxoSet::new(bc);

        let mut pub_key_hash = bs58::decode(&address).into_vec().unwrap();
        pub_key_hash = pub_key_hash[1..pub_key_hash.len() - wallet::ADDRESS_CHECK_SUM_LEN].to_vec();
        let utxo = utxo_set.find_utxo(&pub_key_hash);

        let mut balance = 0;
        for out in utxo {
            balance += out.get_value();
        }
        println!("Balance of '{}' : {}", address, balance);
    }

    pub fn create_wallet(node_id: String) {
        let mut wallets = wallets::new_wallets(node_id.clone());
        let address = wallets.create_wallet();
        wallets.save_to_file(node_id);
        println!("Your new address is: {}", address);
    }

    pub fn list_address(node_id: String) {
        let wallets = wallets::new_wallets(node_id);
        let addresses = wallets.get_addresses();
        for address in addresses {
            println!("{}", address);
        }
    }

    pub fn send(from: String, to: String, amount: i64, node_id: String, mine_now: bool) {
        if !wallet::validate_address(from.clone()) {
            eprintln!("Error: sender Address is not valid");
            std::process::exit(1);
        }
        if !wallet::validate_address(to.clone()) {
            eprintln!("Error: receiver Address is not valid");
            std::process::exit(1);
        }

        let mut blockchain = Blockchain::new_blockchain(node_id.clone()).unwrap();
        let utxo_set = utxo_set::UtxoSet::new(blockchain.clone());

        let wallets = new_wallets(node_id);
        let wallet = wallets.get_wallet(&from).unwrap();

        let transaction = transaction::new_utxo_transaction(&wallet, to.clone(), amount, &utxo_set);

        if mine_now {
            let cbtx = transaction::new_coinbase_tx(from.clone(), "".to_string());
            let transactions = vec![cbtx, transaction];
            let block = blockchain.mine_block(transactions);
            utxo_set.update(block);
        } else {
            unsafe {
                server::send_tx(server::KNOWN_NODES[0].clone(), &transaction);
            }
        }

        println!("Success!");
    }

    pub fn reindex_utxo(node_id: String) {
        let bc = Blockchain::new_blockchain(node_id).unwrap();
        let utxo_set = utxo_set::UtxoSet::new(bc);
        utxo_set.reindex();
        let count = utxo_set.count_transactions();
        println!("Done! There are {} transactions in the UTXO set.", count);
    }

    pub fn start_node(node_id: String, miner_address: String) {
        println!("Starting node {}", node_id);
        if miner_address.len() > 0 {
            if wallet::validate_address(miner_address.clone()) {
                println!(
                    "Mining is on. Address to receive rewards: {}",
                    miner_address
                );
            } else {
                panic!("Wrong miner address!");
            }
        }
        server::start_server(node_id, miner_address);
    }
}
