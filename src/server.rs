use crate::transaction;
use crate::Block;
use crate::Blockchain;
use crate::Transaction;
use crate::UtxoSet;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

const NODE_VERSION: usize = 1;
const COMMAND_LENGTH: usize = 12;

static mut NODE_ADDRESS: String = String::new();
static mut MINING_ADDRESS: String = String::new();
pub static mut KNOWN_NODES: Lazy<Vec<String>> = Lazy::new(|| vec![String::from("127.0.0.1:3000")]);
static mut BLOCKS_IN_TRANSIT: Vec<Vec<u8>> = Vec::new();
static mut MEMPOOL: Lazy<HashMap<String, Transaction>> = Lazy::new(|| HashMap::new());

/*When a new node is run, it gets several nodes from a DNS seed,
and sends them version message */
#[derive(Serialize, Deserialize)]
struct Version {
    version: usize,
    best_height: usize, // the length of the node's blockchain
    addr_from: String,  // the address of the sender
}

pub fn start_server(node_id: String, miner_address: String) {
    unsafe {
        NODE_ADDRESS = format!("127.0.0.1:{}", node_id);
        MINING_ADDRESS = miner_address;
        let ln = TcpListener::bind(&NODE_ADDRESS).unwrap();

        let mut bc = Blockchain::new_blockchain(node_id.clone()).unwrap();
        if NODE_ADDRESS != KNOWN_NODES[0] {
            send_version(KNOWN_NODES[0].clone(), &bc);
        }

        for stream in ln.incoming() {
            let stream = stream.unwrap();
            handle_connection(stream, &mut bc);
        }
    }
}

fn send_version(addr: String, bc: &Blockchain) {
    let best_height = bc.get_best_height();
    let payload = bincode::serialize(&Version {
        version: NODE_VERSION,
        best_height,
        addr_from: unsafe { NODE_ADDRESS.clone() },
    })
    .unwrap();

    let mut request = command_to_bytes("version");
    request.extend(payload);
    send_data(addr, request);
}

fn send_data(addr: String, data: Vec<u8>) {
    let stream = TcpStream::connect(addr.clone());
    match stream {
        Ok(mut stream) => {
            stream.write(&data).unwrap();
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            println!("{} is not available", addr);
            unsafe {
                KNOWN_NODES.retain(|node| *node != addr);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, bc: &mut Blockchain) {
    let mut request = Vec::new();
    stream.read_to_end(&mut request).unwrap();

    let command = bytes_to_command(&request[..COMMAND_LENGTH]);
    println!("Received command: {}", command);

    match command.as_str() {
        "addr" => handle_addr(&request),
        "block" => handle_block(&request, bc),
        "inv" => handle_inv(&request),
        "getblocks" => handle_get_blocks(&request, bc),
        "getdata" => handle_get_data(&request, bc),
        "tx" => handle_tx(&request, bc),
        "version" => handle_version(&request, bc),
        _ => println!("Unknown command!"),
    }

    stream.flush().unwrap();
}

fn handle_version(request: &Vec<u8>, bc: &Blockchain) {
    let payload: Version = bincode::deserialize(&request[COMMAND_LENGTH..]).unwrap();
    let my_best_height = bc.get_best_height();
    let foreigner_best_height = payload.best_height;

    if my_best_height < foreigner_best_height {
        send_get_blocks(payload.addr_from.clone());
    } else if my_best_height > foreigner_best_height {
        send_version(payload.addr_from.clone(), bc);
    }

    if !node_is_known(&payload.addr_from) {
        unsafe {
            KNOWN_NODES.push(payload.addr_from);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GetBlocks {
    addr_from: String,
}

fn send_get_blocks(addr: String) {
    let payload = bincode::serialize(&GetBlocks {
        addr_from: unsafe { NODE_ADDRESS.clone() },
    })
    .unwrap();

    let mut request = command_to_bytes("getblocks");
    request.extend(payload);
    send_data(addr, request);
}

fn handle_get_blocks(request: &Vec<u8>, bc: &Blockchain) {
    let payload: GetBlocks = bincode::deserialize(&request[COMMAND_LENGTH..]).unwrap();
    let blocks = bc.get_block_hashes();
    send_inv(payload.addr_from, "block", blocks);
}

/*Bitcoin uses inv to show other nodes what blocks or
transactions current node has.  */
#[derive(Serialize, Deserialize)]
struct Inv {
    addr_from: String,
    inv_type: String, //whether these are blocks or transactions.
    items: Vec<Vec<u8>>,
}

fn send_inv(address: String, kind: &str, items: Vec<Vec<u8>>) {
    let payload = bincode::serialize(&Inv {
        addr_from: unsafe { NODE_ADDRESS.clone() },
        inv_type: kind.to_string(),
        items,
    })
    .unwrap();

    let mut request = command_to_bytes("inv");
    request.extend(payload);
    send_data(address, request);
}

fn handle_inv(request: &Vec<u8>) {
    let payload: Inv = bincode::deserialize(&request[COMMAND_LENGTH..]).unwrap();
    if payload.inv_type == "block" {
        unsafe {
            BLOCKS_IN_TRANSIT = payload.items.clone();
        }
        let block_hash = payload.items[0].clone();
        send_get_data(payload.addr_from, "block", &block_hash);

        for (i, b) in payload.items.into_iter().enumerate() {
            if b.eq(&block_hash) {
                unsafe {
                    BLOCKS_IN_TRANSIT.remove(i);
                }
            }
        }
    } else if payload.inv_type == "tx" {
        let tx_id = payload.items[0].clone();
        unsafe {
            println!("get MEMPOOP key: {}", hex::encode(tx_id.clone()));
            if !MEMPOOL.contains_key(&hex::encode(tx_id.clone())) {
                send_get_data(payload.addr_from, "tx", &tx_id);
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GetData {
    addr_from: String,
    kind: String,
    id: Vec<u8>,
}

fn send_get_data(addr: String, kind: &str, id: &[u8]) {
    let payload = bincode::serialize(&GetData {
        addr_from: unsafe { NODE_ADDRESS.clone() },
        kind: kind.to_string(),
        id: id.to_vec(),
    })
    .unwrap();

    let mut request = command_to_bytes("getdata");
    request.extend(payload);
    send_data(addr, request);
}

fn handle_get_data(request: &Vec<u8>, bc: &Blockchain) {
    let payload: GetData = bincode::deserialize(&request[COMMAND_LENGTH..]).unwrap();
    if payload.kind == "block" {
        let block = bc.get_block(&payload.id);
        if block.is_none() {
            return;
        }
        send_block(payload.addr_from, &block.unwrap());
    } else if payload.kind == "tx" {
        let tx_id = hex::encode(payload.id);
        unsafe {
            let tx = MEMPOOL[&tx_id].clone();
            send_tx(payload.addr_from, &tx);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Tx {
    addr_from: String,
    transaction: Vec<u8>,
}

pub fn send_tx(addr: String, tx: &Transaction) {
    let payload = bincode::serialize(&Tx {
        addr_from: unsafe { NODE_ADDRESS.clone() },
        transaction: bincode::serialize(tx).unwrap(),
    })
    .unwrap();

    let mut request = command_to_bytes("tx");
    request.extend(payload);
    send_data(addr, request);
}

fn handle_tx(request: &Vec<u8>, bc: &mut Blockchain) {
    let payload: Tx = bincode::deserialize(&request[COMMAND_LENGTH..]).unwrap();
    let tx_data = payload.transaction;
    let tx: Transaction = bincode::deserialize(&tx_data).unwrap();
    unsafe {
        MEMPOOL.insert(hex::encode(tx.get_id()), tx.clone());
        println!("insert into MEMPOOP key: {}", hex::encode(tx.get_id()));

        if NODE_ADDRESS == KNOWN_NODES[0] {
            for node in KNOWN_NODES.clone() {
                if node != NODE_ADDRESS && node != payload.addr_from {
                    send_inv(node, "tx", vec![tx.get_id().clone()]);
                }
            }
        } else {
            while MEMPOOL.len() >= 2 && MINING_ADDRESS.len() > 0 {
                let mut txs: Vec<Transaction> = Vec::new();

                for (_, tx) in MEMPOOL.clone() {
                    if bc.verify_transaction(&tx) {
                        txs.push(tx.clone());
                    }
                }

                if txs.len() == 0 {
                    println!("All transactions are invalid! Waiting for new ones...");
                    return;
                }

                let cb_tx = transaction::new_coinbase_tx(MINING_ADDRESS.clone(), "".to_string());
                txs.push(cb_tx);

                let new_block = bc.mine_block(txs.clone());
                let utxo_set = UtxoSet::new(bc.clone());
                utxo_set.reindex();
                println!("New block is mined!");

                for tx in txs {
                    let tx_id = hex::encode(tx.get_id());
                    MEMPOOL.remove(&tx_id);
                }

                for node in KNOWN_NODES.clone() {
                    if node != NODE_ADDRESS {
                        send_inv(node, "block", vec![new_block.get_hash().clone()]);
                    }
                }

                if MEMPOOL.len() > 0 {
                    continue;
                } else {
                    break;
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct BlockSend {
    addr_from: String,
    block: Vec<u8>,
}

fn send_block(addr: String, block: &Block) {
    let payload = bincode::serialize(&BlockSend {
        addr_from: unsafe { NODE_ADDRESS.clone() },
        block: block.serialize(),
    })
    .unwrap();

    let mut request = command_to_bytes("block");
    request.extend(payload);
    send_data(addr, request);
}

fn handle_block(request: &Vec<u8>, bc: &mut Blockchain) {
    let payload: BlockSend = bincode::deserialize(&request[COMMAND_LENGTH..]).unwrap();
    let block_data = payload.block;
    let block = Block::deserialize_block(block_data);

    println!("Received a new block!");
    bc.add_block(block);

    unsafe {
        if BLOCKS_IN_TRANSIT.len() > 0 {
            let block_hash = BLOCKS_IN_TRANSIT[0].clone();
            send_get_data(payload.addr_from, "block", &block_hash);

            BLOCKS_IN_TRANSIT = BLOCKS_IN_TRANSIT[1..].to_vec();
        } else {
            let utxo_set = UtxoSet::new(bc.clone());
            utxo_set.reindex();
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Addr {
    addr_list: Vec<String>,
}

fn handle_addr(request: &Vec<u8>) {
    let payload: Addr = bincode::deserialize(&request[COMMAND_LENGTH..]).unwrap();
    unsafe {
        KNOWN_NODES.extend(payload.addr_list);
        println!("There are {} known nodes now", KNOWN_NODES.len());
    }
    request_blocks();
}

fn command_to_bytes(command: &str) -> Vec<u8> {
    let mut bytes = vec![0; COMMAND_LENGTH];
    for (i, c) in command.chars().enumerate() {
        bytes[i] = c as u8;
    }
    bytes
}

fn bytes_to_command(bytes: &[u8]) -> String {
    let mut command = String::new();
    for b in bytes {
        if *b != 0 {
            command.push(*b as char);
        }
    }
    command
}

fn node_is_known(addr: &String) -> bool {
    unsafe {
        for node in KNOWN_NODES.clone() {
            if node.eq(addr) {
                return true;
            }
        }
    }
    false
}

fn request_blocks() {
    unsafe {
        for node in KNOWN_NODES.clone() {
            send_get_blocks(node);
        }
    }
}
