use crate::wallet;
use crate::Wallet;
use bincode;
use std::collections::HashMap;
use std::env::current_dir;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

//creates Wallets and fills it from a file if it exists
pub fn new_wallets(node_id: String) -> Wallets {
    let mut wallets = Wallets {
        wallets: HashMap::new(),
    };
    wallets.load_from_file(node_id);
    wallets
}

impl Wallets {
    fn load_from_file(&mut self, node_id: String) {
        let wallet_file = wallet::WALLET_FILE.replace("{}", &node_id);
        let path = current_dir().unwrap().join(wallet_file);
        if !path.exists() {
            println!("No wallet file found. Please create a new wallet first.");
            return;
        }
        let mut file = File::open(path).unwrap();
        let metadata = file.metadata().unwrap();
        let mut buf = vec![0; metadata.len() as usize];
        file.read(&mut buf).unwrap();
        let wallets: HashMap<String, Wallet> = bincode::deserialize(&buf).unwrap();
        self.wallets = wallets;
    }
    // returns an array of addresses stored in the wallet file
    pub fn get_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();
        for (address, _) in &self.wallets {
            addresses.push(address.to_string());
        }
        addresses
    }

    // returns a Wallet by its address
    pub fn get_wallet(&self, address: &str) -> Option<Wallet> {
        if let Some(wallet) = self.wallets.get(address) {
            // let w = wallet.clone();
            return Some(wallet.clone());
        }
        None
    }

    // adds a Wallet to Wallets
    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new_wallet();
        let address = String::from_utf8(wallet.get_address()).unwrap();
        self.wallets.insert(address.clone(), wallet);
        address
    }

    // saves wallets to a file
    pub fn save_to_file(&self, node_id: String) {
        let wallet_file = wallet::WALLET_FILE.replace("{}", &node_id);
        let path = current_dir().unwrap().join(wallet_file);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .unwrap();
        let wallets = bincode::serialize(&self.wallets).unwrap();
        file.write_all(&wallets).unwrap();
    }
}
