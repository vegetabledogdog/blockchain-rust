use crate::wallet;
use crate::wallets;
use crate::UtxoSet;
use bs58;
use ring::signature;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::process;

const SUBSIDY: i64 = 10; // the amount of reward

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    id: Vec<u8>,
    vin: Vec<TXInput>,
    vout: Vec<TXOutput>,
}

impl Transaction {
    // hash of a transaction
    fn hash(&mut self) -> Vec<u8> {
        let encode = bincode::serialize(&self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(encode);
        hasher.finalize().to_vec()
    }

    // coinbase is the mining reward, so it only has inputs without outputs, and the
    // input address originates from 0
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.len() == 0 && self.vin[0].vout == -1
    }

    pub fn sign(&mut self, private_key: &Vec<u8>, prev_txs: &HashMap<String, Transaction>) {
        if self.is_coinbase() {
            return;
        }
        for vin in &mut self.vin {
            if prev_txs.get(&hex::encode(&vin.txid)).is_none() {
                eprintln!("ERROR: Previous transaction is not correct");
            }
        }

        let mut tx_copy = self.trimmed_copy();
        for (in_id, vin) in self.vin.iter_mut().enumerate() {
            let prev_tx = prev_txs.get(&hex::encode(&vin.txid)).unwrap();
            tx_copy.vin[in_id].signature = Vec::new();
            tx_copy.vin[in_id].pub_key = prev_tx.vout[vin.vout as usize].pub_key_hash.clone();
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[in_id].pub_key = Vec::new();

            let tx_bytes = bincode::serialize(&tx_copy).unwrap();
            let signature = ecdsa_sign(private_key, &tx_bytes);
            vin.signature = signature;
        }
    }
    // creates a trimmed copy of Transaction to be used in signing
    pub fn trimmed_copy(&self) -> Transaction {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        for vin in &self.vin {
            inputs.push(TXInput {
                txid: vin.txid.clone(),
                vout: vin.vout,
                signature: vec![],
                pub_key: vec![],
            });
        }
        for vout in &self.vout {
            outputs.push(TXOutput {
                value: vout.value,
                pub_key_hash: vout.pub_key_hash.clone(),
            });
        }
        Transaction {
            id: self.id.clone(),
            vin: inputs,
            vout: outputs,
        }
    }

    pub fn verify(&self, prev_txs: &HashMap<String, Transaction>) -> bool {
        if self.is_coinbase() {
            return true;
        }
        for vin in &self.vin {
            if prev_txs.get(&hex::encode(&vin.txid)).is_none() {
                eprintln!("ERROR: Previous transaction is not correct");
            }
        }

        let mut tx_copy = self.trimmed_copy();
        for (in_id, vin) in self.vin.iter().enumerate() {
            let prev_tx = prev_txs.get(&hex::encode(&vin.txid)).unwrap();
            tx_copy.vin[in_id].signature = Vec::new();
            tx_copy.vin[in_id].pub_key = prev_tx.vout[vin.vout as usize].pub_key_hash.clone();
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[in_id].pub_key = Vec::new();

            let tx_bytes = bincode::serialize(&tx_copy).unwrap();
            let verify = ecdsa_sign_verify(&vin.pub_key, &tx_bytes, &vin.signature);
            if verify == false {
                return false;
            }
        }
        true
    }

    pub fn get_id(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn get_vout(&self) -> Vec<TXOutput> {
        self.vout.clone()
    }

    pub fn get_vin(&self) -> Vec<TXInput> {
        self.vin.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TXOutput {
    value: i64,
    pub_key_hash: Vec<u8>,
}

impl TXOutput {
    // simply locks an output
    pub fn lock(&mut self, address: Vec<u8>) {
        let pub_key_hash = bs58::decode(address).into_vec().unwrap();
        self.pub_key_hash =
            pub_key_hash[1..pub_key_hash.len() - wallet::ADDRESS_CHECK_SUM_LEN].to_vec();
    }

    pub fn is_locked_with_key(&self, pub_key_hash: &Vec<u8>) -> bool {
        self.pub_key_hash.eq(pub_key_hash)
    }

    pub fn get_value(&self) -> i64 {
        self.value
    }

    pub fn new_tx_output(value: i64, address: String) -> TXOutput {
        let mut tx_output = TXOutput {
            value,
            pub_key_hash: Vec::new(),
        };
        tx_output.lock(address.into_bytes());
        tx_output
    }

    pub fn get_pub_key_hash(&self) -> Vec<u8> {
        self.pub_key_hash.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TXInput {
    txid: Vec<u8>,      // previous transaction id
    vout: i64,          //Vout stores an index of an output in the transaction.
    signature: Vec<u8>, // signature
    pub_key: Vec<u8>,   // public key
}

impl TXInput {
    //  checks that an input uses a specific key to unlock an output
    pub fn uses_key(&self, pub_key_hash: &Vec<u8>) -> bool {
        let locking_hash = wallet::hash_pub_key(&self.pub_key);
        locking_hash.eq(pub_key_hash)
    }

    pub fn get_txid(&self) -> Vec<u8> {
        self.txid.clone()
    }

    pub fn get_vout(&self) -> i64 {
        self.vout
    }

    pub fn get_pub_key(&self) -> Vec<u8> {
        self.pub_key.clone()
    }
}

// creates a new coinbase transaction
pub fn new_coinbase_tx(to: String, mut data: String) -> Transaction {
    if data == "" {
        data = format!("Reward to '{}'", to);
    }
    let txin = TXInput {
        txid: vec![],
        vout: -1,
        signature: vec![],
        pub_key: data.into_bytes(),
    };
    let txout = TXOutput::new_tx_output(SUBSIDY, to);
    let mut tx = Transaction {
        id: vec![],
        vin: vec![txin],
        vout: vec![txout],
    };
    tx.id = tx.hash();
    tx
}

//  a general transaction
pub fn new_utxo_transaction(
    from: String,
    to: String,
    amount: i64,
    utxo_set: &UtxoSet,
) -> Transaction {
    let mut txs_inputs = Vec::new();
    let mut txs_outputs = Vec::new();

    let wallets = wallets::new_wallets();
    let wallet = wallets.get_wallet(from.as_str()).unwrap();
    let pub_key_hash = wallet::hash_pub_key(&wallet.public_key);

    let (acc, valid_outputs) = utxo_set.find_spendable_outputs(&pub_key_hash, amount);

    if acc < amount {
        eprintln!("Error: Not enough funds");
        process::exit(-1);
    }

    for (txid, outs) in valid_outputs.iter() {
        // spending coins, writing into inputs indicates that the money has been spent
        for out in outs {
            let input = TXInput {
                txid: hex::decode(txid.clone()).unwrap(),
                vout: *out as i64,
                signature: vec![],
                pub_key: wallet.public_key.clone(),
            };
            txs_inputs.push(input);
        }
    }

    // transfer utxo to the "to" address
    txs_outputs.push(TXOutput::new_tx_output(amount, to.clone()));

    // change coins
    if acc > amount {
        txs_outputs.push(TXOutput::new_tx_output(acc - amount, from.clone()));
    }

    let mut tx = Transaction {
        id: Vec::new(),
        vin: txs_inputs,
        vout: txs_outputs,
    };
    tx.id = tx.hash();
    utxo_set
        .get_blockchain()
        .sign_transaction(&mut tx, &wallet.get_private_key());
    tx
}

pub fn ecdsa_sign(private_key: &Vec<u8>, data: &Vec<u8>) -> Vec<u8> {
    let key_pair = signature::EcdsaKeyPair::from_pkcs8(
        &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
        private_key,
    )
    .unwrap();
    let rng = ring::rand::SystemRandom::new();
    key_pair.sign(&rng, data).unwrap().as_ref().to_vec()
}

pub fn ecdsa_sign_verify(public_key: &Vec<u8>, data: &Vec<u8>, signature: &Vec<u8>) -> bool {
    let public_key =
        signature::UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, public_key);
    public_key.verify(data, signature).is_ok()
}
