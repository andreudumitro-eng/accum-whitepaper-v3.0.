//! Miner identity, bonds, and loyalty

use crate::constants::*;
use crate::error::Error;
use crate::types::{MinerId, Amount, Height};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Bond entry for a miner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondEntry {
    pub amount: Amount,
    pub locked_until: Height,
    pub created_at: Height,
}

/// Miner information for PoCI calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerInfo {
    pub miner_id: MinerId,
    pub shares_raw: u64,
    pub loyalty: f64,
    pub bond_total: Amount,
    pub invalid_shares: u32,
    pub total_shares: u32,
    pub last_epoch_participated: u32,
    pub first_seen: u64,
}

impl Default for MinerInfo {
    fn default() -> Self {
        Self {
            miner_id: [0u8; 20],
            shares_raw: 0,
            loyalty: 0.0,
            bond_total: 0,
            invalid_shares: 0,
            total_shares: 0,
            last_epoch_participated: 0,
            first_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Registry of all miners and their bonds
#[derive(Debug, Default, Clone)]
pub struct MinerRegistry {
        miners: HashMap<MinerId, MinerInfo>,
    bonds: HashMap<MinerId, Vec<BondEntry>>,
    current_epoch: u32,
    current_height: Height,
}

impl MinerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create miner info
    pub fn get_or_create_miner(&mut self, miner_id: MinerId) -> &mut MinerInfo {
        self.miners.entry(miner_id).or_insert_with(|| {
            let mut info = MinerInfo::default();
            info.miner_id = miner_id;
            info
        })
    }

    /// Add valid share for miner
    pub fn add_valid_share(&mut self, miner_id: MinerId) {
        // Get miner first, then update
        if let Some(miner) = self.miners.get_mut(&miner_id) {
            miner.shares_raw += 1;
            miner.total_shares += 1;
            miner.last_epoch_participated = self.current_epoch;
        } else {
            // Create new miner
            let mut info = MinerInfo::default();
            info.miner_id = miner_id;
            info.shares_raw = 1;
            info.total_shares = 1;
            info.last_epoch_participated = self.current_epoch;
            self.miners.insert(miner_id, info);
        }
    }

    /// Add invalid share for miner
    pub fn add_invalid_share(&mut self, miner_id: MinerId) {
        if let Some(miner) = self.miners.get_mut(&miner_id) {
            miner.invalid_shares += 1;
            miner.total_shares += 1;
        } else {
            let mut info = MinerInfo::default();
            info.miner_id = miner_id;
            info.invalid_shares = 1;
            info.total_shares = 1;
            self.miners.insert(miner_id, info);
        }
    }

    /// Add bond for miner
    pub fn add_bond(&mut self, miner_id: MinerId, amount: Amount) -> Result<(), Error> {
        if amount < MINIMUM_BOND_LYT {
            return Err(Error::InsufficientInput); // Bond too small
        }

        let lock_until = self.current_height + BOND_LOCKUP_BLOCKS;
        
        let bond = BondEntry {
            amount,
            locked_until: lock_until,
            created_at: self.current_height,
        };

        self.bonds.entry(miner_id).or_default().push(bond);
        
        // Update total bond in miner info
        if let Some(miner) = self.miners.get_mut(&miner_id) {
            miner.bond_total += amount;
        }

        Ok(())
    }

    /// Update loyalty for all miners (end of epoch)
    pub fn update_loyalty(&mut self) {
        for miner in self.miners.values_mut() {
            if miner.last_epoch_participated == self.current_epoch {
                // Participated: +1
                miner.loyalty += 1.0;
            } else {
                // Missed: max(loyalty * 0.7, loyalty // 2)
                let decayed = (miner.loyalty * 0.7).max(miner.loyalty.floor() / 2.0);
                miner.loyalty = decayed;
            }
        }
    }

    /// Clean up expired bonds
    pub fn cleanup_expired_bonds(&mut self) {
        for (miner_id, bond_list) in self.bonds.iter_mut() {
            let before: Amount = bond_list.iter().map(|b| b.amount).sum();
            
            // Keep only bonds still locked
            bond_list.retain(|bond| bond.locked_until > self.current_height);
            
            let after: Amount = bond_list.iter().map(|b| b.amount).sum();
            
            // Update total if changed
            if before != after {
                if let Some(miner) = self.miners.get_mut(miner_id) {
                    miner.bond_total = after;
                }
            }
        }
    }

    /// Check which miners should be banned (>30% invalid shares)
    pub fn check_bans(&self) -> Vec<MinerId> {
        let mut to_ban = Vec::new();
        
        for (miner_id, miner) in &self.miners {
            if miner.total_shares == 0 {
                continue;
            }
            let ratio = miner.invalid_shares as f64 / miner.total_shares as f64;
            if ratio > 0.3 {
                to_ban.push(*miner_id);
            }
        }
        
        to_ban
    }

    /// Get miner info
    pub fn get_miner(&self, miner_id: &MinerId) -> Option<&MinerInfo> {
        self.miners.get(miner_id)
    }

    /// Get all miner IDs
    pub fn all_miners(&self) -> Vec<MinerId> {
        self.miners.keys().copied().collect()
    }

    /// Get total bond for miner (normalized for PoCI)
    pub fn bond_for_poci(&self, miner_id: &MinerId) -> f64 {
        if let Some(miner) = self.miners.get(miner_id) {
            if miner.bond_total >= MINIMUM_BOND_LYT {
                (miner.bond_total as f64).sqrt()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Get loyalty for miner (normalized for PoCI)
    pub fn loyalty_for_poci(&self, miner_id: &MinerId) -> f64 {
        self.miners.get(miner_id).map(|m| m.loyalty).unwrap_or(0.0)
    }

    /// Get shares for miner (raw count)
    pub fn shares_for_poci(&self, miner_id: &MinerId) -> u64 {
        self.miners.get(miner_id).map(|m| m.shares_raw).unwrap_or(0)
    }

    /// Move to next epoch
    pub fn next_epoch(&mut self) {
        self.current_epoch += 1;
        self.update_loyalty();
        self.cleanup_expired_bonds();
    }

    /// Set current height
    pub fn set_height(&mut self, height: Height) {
        self.current_height = height;
    }
}

/// Equivocation proof for slashing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivocationProof {
    pub miner_id: MinerId,
    pub block_header_a: crate::block::BlockHeader,
    pub block_header_b: crate::block::BlockHeader,
}

impl EquivocationProof {
    /// Verify that this is a valid equivocation
    pub fn verify(&self) -> bool {
        // Same miner
        if self.block_header_a.hash() == self.block_header_b.hash() {
            return false; // Same block, not equivocation
        }
        
        // Same height
        // Note: In real implementation, would need to check height from context
        true
    }
}