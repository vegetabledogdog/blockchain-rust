use crate::Blockchain;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::process;

const SUBSIDY: i64 = 10; // the amount of reward

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    id: Vec<u8>,
    vin: Vec<TXInput>,
    vout: Vec<TXOutput>,
}

impl Transaction {
    // sets ID of a transaction
    fn set_id(&mut self) {
        let encode = bincode::serialize(&self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(encode);
        self.id = hasher.finalize().to_vec();
    }

    // coinbase is the mining reward, so it only has inputs without outputs, and the
    // input address originates from 0
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.len() == 0 && self.vin[0].vout == -1
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

#[derive(Serialize, Deserialize, Clone)]
pub struct TXOutput {
    value: i64,
    // used to define outputs locking and unlocking logic
    // https://en.bitcoin.it/wiki/Script
    script_pub_key: String,
}

impl TXOutput {
    pub fn can_be_unlocked_with(&self, unlocking_data: String) -> bool {
        self.script_pub_key == unlocking_data
    }

    pub fn get_value(&self) -> i64 {
        self.value
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TXInput {
    txid: Vec<u8>,      // previous transaction id
    vout: i64,          //Vout stores an index of an output in the transaction.
    script_sig: String, // ScriptSig is a script which provides data to be used in an output’s ScriptPubKey
}

impl TXInput {
    pub fn can_unlock_output_with(&self, unlocking_data: String) -> bool {
        self.script_sig == unlocking_data
    }

    pub fn get_txid(&self) -> Vec<u8> {
        self.txid.clone()
    }

    pub fn get_vout(&self) -> i64 {
        self.vout
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
        script_sig: data,
    };
    let txout = TXOutput {
        value: SUBSIDY,
        script_pub_key: to,
    };
    let mut tx = Transaction {
        id: vec![],
        vin: vec![txin],
        vout: vec![txout],
    };
    tx.set_id();
    tx
}

//  a general transaction
pub fn new_utxo_transaction(from: String, to: String, amount: i64, bc: &Blockchain) -> Transaction {
    let mut txs_inputs = Vec::new();
    let mut txs_outputs = Vec::new();

    let (acc, valid_outputs) = bc.find_spendable_outputs(from.clone(), amount);

    if acc < amount {
        eprintln!("Error: Not enough funds");
        process::exit(-1);
    }

    for (txid, outs) in valid_outputs.iter() {
        // spending coins, writing into inputs indicates that the money has been spent
        for out in outs {
            let input = TXInput {
                txid: hex::decode(txid.clone()).unwrap(),
                vout: *out,
                script_sig: from.clone(),
            };
            txs_inputs.push(input);
        }
    }

    // transfer utxo to the "to" address
    txs_outputs.push(TXOutput {
        value: amount,
        script_pub_key: to.clone(),
    });

    // change coins
    if acc > amount {
        txs_outputs.push(TXOutput {
            value: acc - amount,
            script_pub_key: from.clone(),
        });
    }

    let mut tx = Transaction {
        id: Vec::new(),
        vin: txs_inputs,
        vout: txs_outputs,
    };
    tx.set_id();

    tx
}
