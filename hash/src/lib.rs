use sha2::{Sha256, Digest};
use rand;
use rand::RngCore;

pub fn random_bytes() -> [u8; 32] {
    let mut data = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut data);

    return data;
}


pub fn hash_bytes(bytes: [u8; 32]) -> String {
    let mut hasher = Sha256::new();

    hasher.update(bytes);

    let result = format!("{:x}", hasher.finalize());

    return result;
}
