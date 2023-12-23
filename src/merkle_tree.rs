use sha2::{Digest, Sha256};

pub struct MerkleTree {
    pub root_node: MerkleNode,
}

#[derive(Clone)]
pub struct MerkleNode {
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
    pub data: Vec<u8>,
}

fn new_merkle_node(
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
    data: Vec<u8>,
) -> MerkleNode {
    let mut m_node = MerkleNode {
        left: left.clone(),
        right: left.clone(),
        data: vec![],
    };
    if left.is_none() && right.is_none() {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hasher.finalize().to_vec();
        m_node.data = hash;
    } else {
        let prev_hashes = [left.unwrap().data, right.unwrap().data].concat();
        let mut hasher = Sha256::new();
        hasher.update(&prev_hashes);
        let hash = hasher.finalize().to_vec();
        m_node.data = hash;
    }
    m_node
}

pub fn new_merkle_tree(mut data: Vec<Vec<u8>>) -> MerkleTree {
    let mut nodes: Vec<MerkleNode> = vec![];

    if data.len() % 2 != 0 {
        data.push(data[data.len() - 1].clone());
    }

    for datum in data.clone() {
        let node = new_merkle_node(None, None, datum);
        nodes.push(node);
    }

    for _ in 0..data.len() / 2 {
        let mut new_level: Vec<MerkleNode> = vec![];
        for j in (0..nodes.len()).step_by(2) {
            let node = new_merkle_node(
                Some(Box::new(nodes[j].clone())),
                Some(Box::new(nodes[j + 1].clone())),
                vec![],
            );
            new_level.push(node);
        }

        nodes = new_level;
    }

    MerkleTree {
        root_node: nodes[0].clone(),
    }
}
