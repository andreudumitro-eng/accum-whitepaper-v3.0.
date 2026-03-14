//! Share structures and validation

use crate::block::BlockHeader;
use crate::constants::*;
use crate::crypto::{argon2id_hash, quick_prefilter};
use crate::error::Error;
use crate::types::{Hash32, MinerId, Target};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Share packet (180 bytes, little-endian)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePacket {
    pub miner_id: MinerId,
    pub header: BlockHeader,
    pub nonce: u64,
    pub hash: Hash32,
}

impl SharePacket {
    /// Create new share packet
    pub fn new(miner_id: MinerId, header: BlockHeader, nonce: u64) -> Self {
        let mut share_header = header.clone();
        share_header.nonce = nonce;
        let hash = argon2id_hash(&share_header.as_bytes());
        
        Self {
            miner_id,
            header: share_header,
            nonce,
            hash,
        }
    }

    /// Validate share against target
    pub fn validate(&self, target_share: &Target) -> Result<(), Error> {
        // Quick pre-filter
        let header_bytes = self.header.as_bytes();
        if !quick_prefilter(&header_bytes, self.nonce, 0) {
            return Err(Error::InvalidShare);
        }

        // Verify hash
        let computed = argon2id_hash(&header_bytes);
        if computed != self.hash {
            return Err(Error::InvalidShare);
        }

        // Check against target
        for i in 0..32 {
            if self.hash[i] < target_share.0[i] {
                return Ok(());
            }
            if self.hash[i] > target_share.0[i] {
                return Err(Error::InvalidShare);
            }
        }
        Ok(())
    }
}

/// Share pool for current epoch
#[derive(Debug, Default)]
pub struct SharePool {
    shares: HashMap<MinerId, Vec<SharePacket>>,
    share_count: usize,
    invalid_counts: HashMap<MinerId, (u32, u32)>, // (invalid, total)
}

impl SharePool {
    /// Create new empty share pool
    pub fn new() -> Self {
        Self::default()
    }

    /// Add share to pool
    pub fn add_share(&mut self, share: SharePacket) -> Result<(), Error> {
        let miner_shares = self.shares.entry(share.miner_id).or_default();
        
        // Check per-miner limit
        if miner_shares.len() >= MAX_SHARES_PER_MINER_PER_EPOCH as usize {
            return Err(Error::InvalidShare);
        }

        miner_shares.push(share);
        self.share_count += 1;
        Ok(())
    }

    /// Track invalid share
    pub fn track_invalid(&mut self, miner_id: MinerId) {
        let entry = self.invalid_counts.entry(miner_id).or_insert((0, 0));
        entry.0 += 1; // invalid
        entry.1 += 1; // total
    }

    /// Track valid share
    pub fn track_valid(&mut self, miner_id: MinerId) {
        let entry = self.invalid_counts.entry(miner_id).or_insert((0, 0));
        entry.1 += 1; // total
    }

    /// Check if miner should be banned (>30% invalid)
    pub fn should_ban(&self, miner_id: MinerId) -> bool {
        if let Some((invalid, total)) = self.invalid_counts.get(&miner_id) {
            if *total > 0 {
                let ratio = (*invalid as f64) / (*total as f64);
                return ratio > 0.3;
            }
        }
        false
    }

    /// Get shares for a miner
    pub fn get_miner_shares(&self, miner_id: &MinerId) -> Option<&Vec<SharePacket>> {
        self.shares.get(miner_id)
    }

    /// Get share count for miner
    pub fn miner_share_count(&self, miner_id: &MinerId) -> usize {
        self.shares.get(miner_id).map(|v| v.len()).unwrap_or(0)
    }

    /// Get total shares in pool
    pub fn total_shares(&self) -> usize {
        self.share_count
    }

    /// Get all miner IDs
    pub fn miners(&self) -> Vec<MinerId> {
        self.shares.keys().copied().collect()
    }

    /// Clear pool for new epoch
    pub fn clear(&mut self) {
        self.shares.clear();
        self.share_count = 0;
        self.invalid_counts.clear();
    }
}

/// Calculate target_share from target_block
pub fn target_share_from_block(target_block: &Target) -> Target {
    let mut result = [0u8; 32];
    
    // Convert to u16 to avoid overflow
    for i in 0..31 {
        let val = ((target_block.0[i] as u16) << 8) | (target_block.0[i + 1] as u16 >> 8);
        result[i] = (val >> 8) as u8;
    }
    result[31] = target_block.0[31];
    
    Target(result)
}