//! Block structures and validation

use crate::constants::*;
use crate::error::Error;
use crate::types::{Hash32, Target};
use crate::crypto::argon2id_hash;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Block header (120 bytes, little-endian)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_hash: Hash32,
    pub merkle_root: Hash32,
    pub timestamp: u64,
    pub difficulty: Target,
    pub nonce: u64,
    pub epoch_index: u32,
}

impl BlockHeader {
    /// Convert header to bytes for hashing
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(120);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.prev_hash);
        bytes.extend_from_slice(&self.merkle_root);
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.difficulty.0);
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes.extend_from_slice(&self.epoch_index.to_le_bytes());
        bytes
    }

    /// Compute block hash using Argon2id
    pub fn hash(&self) -> Hash32 {
        argon2id_hash(&self.as_bytes())
    }

    /// Check if hash meets target (PoW valid)
    pub fn meets_target(&self, target: &Target) -> bool {
        let hash = self.hash();
        for i in 0..32 {
            if hash[i] < target.0[i] {
                return true;
            }
            if hash[i] > target.0[i] {
                return false;
            }
        }
        true
    }

    /// Basic timestamp validation
    pub fn validate_timestamp(&self, prev_timestamp: Option<u64>) -> Result<(), Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Not too far in future
        if self.timestamp > now + 7200 {
            return Err(Error::InvalidTimestamp);
        }

        // Must be after previous block timestamp
        if let Some(prev) = prev_timestamp {
            if self.timestamp <= prev {
                return Err(Error::InvalidTimestamp);
            }
        }

        Ok(())
    }
}

/// Full block with transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

/// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub locktime: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxIn {
    pub prev_txid: Hash32,
    pub prev_index: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOut {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}