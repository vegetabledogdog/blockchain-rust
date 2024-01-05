use bs58;
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};
use ripemd::Ripemd160;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/* Bitcoin Address
Version  Public key hash                           Checksum
00       62E907B15CBF27D5425399EBF6F0FB50EBB88F18  C29B7D93
*/

const VERSION: u8 = 0x00;
pub const WALLET_FILE: &str = "wallet_{}.dat";
pub const ADDRESS_CHECK_SUM_LEN: usize = 4;

#[derive(Clone, Serialize, Deserialize)]
pub struct Wallet {
    private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl Wallet {
    pub fn new_wallet() -> Wallet {
        let private_key = new_key_pair();
        let key_pair =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &private_key).unwrap();
        let public_key = key_pair.public_key().as_ref().to_vec();
        Wallet {
            private_key,
            public_key,
        }
    }

    pub fn get_address(&self) -> Vec<u8> {
        let hash_pub_key = hash_pub_key(&self.public_key);
        let mut payload = [VERSION].to_vec();
        payload.extend(hash_pub_key);
        let check_sum = check_sum(&payload);
        payload.extend(check_sum);
        bs58::encode(payload).into_vec()
    }

    pub fn get_private_key(&self) -> Vec<u8> {
        self.private_key.clone()
    }
}

pub fn new_key_pair() -> Vec<u8> {
    let rng = SystemRandom::new();
    let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();
    pkcs8.as_ref().to_vec()
}

pub fn hash_pub_key(pub_key: &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(pub_key);
    let public_sha256 = hasher.finalize().to_vec();
    ripe_md_160(public_sha256)
}

// generates a checksum for a public key
pub fn check_sum(payload: &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let mut hash = hasher.finalize_reset().to_vec();
    hasher.update(hash);
    hash = hasher.finalize().to_vec();
    return hash[..ADDRESS_CHECK_SUM_LEN].to_vec();
}

pub fn ripe_md_160(payload: Vec<u8>) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(payload);
    hasher.finalize().to_vec()
}

// check if address if valid
pub fn validate_address(address: String) -> bool {
    let pub_key_hash = bs58::decode(address).into_vec().unwrap();
    let actual_check_sum = &pub_key_hash[pub_key_hash.len() - ADDRESS_CHECK_SUM_LEN..];
    let version = pub_key_hash[0];
    let pub_key_hash = &pub_key_hash[1..pub_key_hash.len() - ADDRESS_CHECK_SUM_LEN].to_vec();

    let mut target_vec = vec![];
    target_vec.push(version);
    target_vec.extend(pub_key_hash);
    let target_checksum = check_sum(&target_vec);
    actual_check_sum == target_checksum
}

// calculate address from public key hash
pub fn calc_address(pub_hash_key: &Vec<u8>) -> String {
    let mut payload = [VERSION].to_vec();
    payload.extend(pub_hash_key);
    let check_sum = check_sum(&payload);
    payload.extend(check_sum);
    bs58::encode(payload).into_string()
}
