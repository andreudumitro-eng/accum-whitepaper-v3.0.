//! Epoch lifecycle management

use crate::block::BlockHeader;
use crate::constants::*;
use crate::consensus::PoCICalculator;
use crate::difficulty::adjust_difficulty;
use crate::error::Error;
use crate::miner::MinerRegistry;
use crate::pool::PersistentSharePool;
use crate::share::target_share_from_block;
use crate::types::{Amount, EpochIndex, MinerId, Target};
use std::collections::HashMap;

/// Epoch state
#[derive(Debug)]
pub struct Epoch {
    pub index: EpochIndex,
    pub start_block: u64,
    pub end_block: u64,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub target_block: Target,
    pub target_share: Target,
    pub total_shares: u64,
    pub active_miners: usize,
}

impl Epoch {
    /// Create new epoch
    pub fn new(index: EpochIndex, start_block: u64, start_time: u64, target_block: Target) -> Self {
        let target_share = target_share_from_block(&target_block);
        
        Self {
            index,
            start_block,
            end_block: start_block + EPOCH_BLOCKS - 1,
            start_time,
            end_time: None,
            target_block,
            target_share,
            total_shares: 0,
            active_miners: 0,
        }
    }

    /// Check if block belongs to this epoch
    pub fn contains_block(&self, height: u64) -> bool {
        height >= self.start_block && height <= self.end_block
    }

    /// Mark epoch as ended
    pub fn end(&mut self, end_time: u64) {
        self.end_time = Some(end_time);
    }

    /// Update stats
    pub fn update_stats(&mut self, total_shares: u64, active_miners: usize) {
        self.total_shares = total_shares;
        self.active_miners = active_miners;
    }
}

/// Epoch manager
#[derive(Debug)]
pub struct EpochManager {
    epochs: HashMap<EpochIndex, Epoch>,
    current_epoch: EpochIndex,
    block_timestamps: Vec<u64>,
    block_heights: Vec<u64>,
    share_pool: PersistentSharePool,
    miner_registry: MinerRegistry,
}

impl EpochManager {
    /// Create new epoch manager
    pub fn new(initial_target: Target) -> Self {
        let mut manager = Self {
            epochs: HashMap::new(),
            current_epoch: 1,
            block_timestamps: Vec::new(),
            block_heights: Vec::new(),
            share_pool: PersistentSharePool::new(500),
            miner_registry: MinerRegistry::new(),
        };
        
        // Create genesis epoch
        let genesis_epoch = Epoch::new(1, 0, 1741353600, initial_target);
        manager.epochs.insert(1, genesis_epoch);
        
        manager
    }

    /// Get current epoch
    pub fn current(&self) -> Option<&Epoch> {
        self.epochs.get(&self.current_epoch)
    }

    /// Get current epoch mutably
    pub fn current_mut(&mut self) -> Option<&mut Epoch> {
        self.epochs.get_mut(&self.current_epoch)
    }

    /// Add a new block
    pub fn add_block(&mut self, header: &BlockHeader) -> Result<(), Error> {
        let height = header.epoch_index as u64 * EPOCH_BLOCKS + header.nonce as u64; // Simplified
        
        // Verify epoch
        if header.epoch_index != self.current_epoch {
            return Err(Error::InvalidEpoch);
        }
        
        // Verify timestamp
        if !self.verify_timestamp(header.timestamp) {
            return Err(Error::InvalidTimestamp);
        }
        
        // Store timestamp
        self.block_timestamps.push(header.timestamp);
        self.block_heights.push(height);
        
        // Check if epoch ended
        if self.block_timestamps.len() as u64 >= EPOCH_BLOCKS {
            self.end_current_epoch()?;
        }
        
        Ok(())
    }

    /// Verify timestamp against median of last 11 blocks
    fn verify_timestamp(&self, timestamp: u64) -> bool {
        if self.block_timestamps.len() < 11 {
            return true;
        }
        
        let mut last_11 = self.block_timestamps[self.block_timestamps.len() - 11..].to_vec();
        last_11.sort();
        let median = last_11[5];
        
        timestamp > median
    }

    /// End current epoch and start next
    fn end_current_epoch(&mut self) -> Result<(), Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Mark current epoch as ended
        if let Some(epoch) = self.epochs.get_mut(&self.current_epoch) {
            epoch.end(now);
            
            // Update stats from share pool
            epoch.update_stats(
                self.share_pool.total_shares() as u64,
                self.share_pool.miners().len(),
            );
        }
        
        // Calculate PoCI and rewards
        let rewards = self.calculate_epoch_rewards()?;
        
        // Adjust difficulty for next epoch
        let new_target = self.calculate_next_target()?;
        
        // Create next epoch
        let next_epoch = self.current_epoch + 1;
        let next_start_block = self.block_heights.last().unwrap_or(&0) + 1;
        let next_epoch_obj = Epoch::new(next_epoch, next_start_block, now, new_target);
        self.epochs.insert(next_epoch, next_epoch_obj);
        
        // Clear share pool for next epoch
        self.share_pool.new_epoch();
        
        // Update miner registry for next epoch
        self.miner_registry.next_epoch();
        
        self.current_epoch = next_epoch;
        
        println!("Epoch {} ended, {} started", self.current_epoch - 1, self.current_epoch);
        println!("Rewards calculated: {} miners", rewards.len());
        
        Ok(())
    }

    /// Calculate PoCI and rewards for current epoch
    fn calculate_epoch_rewards(&self) -> Result<HashMap<MinerId, Amount>, Error> {
        let calculator = PoCICalculator::new(
            // Need to clone or reference - simplified
            crate::share::SharePool::new(),
            self.miner_registry.clone(), // Need Clone impl
        );
        
        Ok(calculator.calculate_epoch_rewards())
    }

    /// Calculate next epoch target based on last 120 blocks
    fn calculate_next_target(&self) -> Result<Target, Error> {
        if self.block_timestamps.len() < 120 {
            // Not enough blocks, use current target
            return Ok(self.current().unwrap().target_block);
        }
        
        let start = self.block_timestamps[self.block_timestamps.len() - 120];
        let end = self.block_timestamps.last().unwrap();
        let time_span = end - start;
        
        let current_target = self.current().unwrap().target_block;
        Ok(adjust_difficulty(&current_target, time_span))
    }

    /// Get epoch by index
    pub fn get_epoch(&self, index: EpochIndex) -> Option<&Epoch> {
        self.epochs.get(&index)
    }

    /// Get share pool
    pub fn share_pool(&self) -> &PersistentSharePool {
        &self.share_pool
    }

    /// Get share pool mutably
    pub fn share_pool_mut(&mut self) -> &mut PersistentSharePool {
        &mut self.share_pool
    }

    /// Get miner registry
    pub fn miner_registry(&self) -> &MinerRegistry {
        &self.miner_registry
    }

    /// Get miner registry mutably
    pub fn miner_registry_mut(&mut self) -> &mut MinerRegistry {
        &mut self.miner_registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockHeader;
    use crate::types::Target;
    
    #[test]
    fn test_epoch_creation() {
        let target = Target([0xFF; 32]);
        let epoch = Epoch::new(1, 0, 1741353600, target);
        
        assert_eq!(epoch.index, 1);
        assert_eq!(epoch.start_block, 0);
        assert_eq!(epoch.end_block, 1439);
        assert!(epoch.contains_block(1000));
        assert!(!epoch.contains_block(1500));
    }
    
    #[test]
    fn test_timestamp_verification() {
        let target = Target([0xFF; 32]);
        let mut manager = EpochManager::new(target);
        
        // Add 11 blocks with increasing timestamps
        for i in 0..11 {
            let header = BlockHeader {
                version: 1,
                prev_hash: [0u8; 32],
                merkle_root: [0u8; 32],
                timestamp: 1741353600 + i * 60,
                difficulty: target,
                nonce: i,
                epoch_index: 1,
            };
            let _ = manager.add_block(&header);
        }
        
        // Timestamp should be > median of last 11
        assert!(manager.verify_timestamp(1741353600 + 11 * 60 + 1));
        assert!(!manager.verify_timestamp(1741353600)); // Too old
    }
}