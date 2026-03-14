//! Cryptographic functions: Argon2id, hashing, miner ID

use crate::constants::*;
use crate::types::{Hash32, MinerId, Target};
use argon2::{Argon2, Algorithm, Version, Params};
use sha2::{Sha256, Digest};
use ripemd::Ripemd160;

/// Argon2id hashing with protocol parameters (256MB memory, 2 iterations, 4 parallelism)
pub fn argon2id_hash(data: &[u8]) -> Hash32 {
    // Convert bytes to kilobytes (Params::new expects KB)
    let memory_kb = ARGON2_MEMORY / 1024; // 268_435_456 / 1024 = 262_144 KB
    
    let params = Params::new(
        memory_kb,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(ARGON2_HASH_LEN),
    ).expect("Invalid Argon2 parameters");
    
    let hasher = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        params,
    );
    
    let mut output = [0u8; 32];
    hasher
        .hash_password_into(data, b"accum_salt", &mut output)
        .expect("Argon2id failed");
    
    output
}

/// Compute miner ID from compressed public key (RIPEMD160(SHA256(pubkey)))
pub fn miner_id_from_pubkey(pubkey: &[u8]) -> MinerId {
    let sha256 = Sha256::digest(pubkey);
    let ripemd160 = Ripemd160::digest(&sha256);
    
    let mut miner_id = [0u8; 20];
    miner_id.copy_from_slice(&ripemd160);
    miner_id
}

/// Compare hash against target (little-endian)
pub fn hash_meets_target(hash: &Hash32, target: &Target) -> bool {
    for i in 0..32 {
        if hash[i] < target.0[i] {
            return true;
        }
        if hash[i] > target.0[i] {
            return false;
        }
    }
    true // equal also meets target
}

/// Quick pre-filter using SHA256 (rejects before Argon2)
pub fn quick_prefilter(header: &[u8], nonce: u64, target: u64) -> bool {
    let mut data = Vec::with_capacity(header.len() + 8);
    data.extend_from_slice(header);
    data.extend_from_slice(&nonce.to_le_bytes());
    
    let hash = Sha256::digest(&data);
    let first_bytes = u64::from_le_bytes(hash[0..8].try_into().unwrap());
    
    first_bytes <= target
}