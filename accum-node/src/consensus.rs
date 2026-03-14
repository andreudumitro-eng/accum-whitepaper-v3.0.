//! Proof-of-Contribution Index (PoCI) and reward distribution

use crate::constants::*;
use crate::miner::MinerRegistry;
use crate::share::SharePool;
use crate::types::{MinerId, Amount};
use std::collections::HashMap;

/// PoCI calculator for an epoch
pub struct PoCICalculator {
    share_pool: SharePool,
    miner_registry: MinerRegistry,
}

impl PoCICalculator {
    /// Create new PoCI calculator
    pub fn new(share_pool: SharePool, miner_registry: MinerRegistry) -> Self {
        Self {
            share_pool,
            miner_registry,
        }
    }

    /// Get all active miners in this epoch
    pub fn active_miners(&self) -> Vec<MinerId> {
        self.share_pool.miners()
    }

    /// Calculate normalized shares (sqrt)
    pub fn normalized_shares(&self) -> HashMap<MinerId, f64> {
        let mut result = HashMap::new();
        let miners = self.active_miners();
        
        if miners.is_empty() {
            return result;
        }

        // Calculate sqrt for each miner
        let mut sqrt_values = Vec::new();
        for miner_id in &miners {
            let shares = self.share_pool.miner_share_count(miner_id) as f64;
            sqrt_values.push(shares.sqrt());
        }

        // Find max sqrt
        let mut max_sqrt = 0.0_f64;
        for &val in &sqrt_values {
            if val > max_sqrt {
                max_sqrt = val;
            }
        }
        
        if max_sqrt == 0.0 {
            return result;
        }

        // Normalize
        for (i, miner_id) in miners.iter().enumerate() {
            result.insert(*miner_id, sqrt_values[i] / max_sqrt);
        }

        result
    }

    /// Calculate normalized loyalty
    pub fn normalized_loyalty(&self) -> HashMap<MinerId, f64> {
        let mut result = HashMap::new();
        let miners = self.active_miners();
        
        if miners.is_empty() {
            return result;
        }

        // Get loyalty values
        let mut loyalty_values = Vec::new();
        for miner_id in &miners {
            let loyalty = self.miner_registry.loyalty_for_poci(miner_id);
            loyalty_values.push(loyalty);
        }

        // Find max loyalty
        let mut max_loyalty = 0.0_f64;
        for &val in &loyalty_values {
            if val > max_loyalty {
                max_loyalty = val;
            }
        }
        
        if max_loyalty == 0.0 {
            return result;
        }

        // Normalize
        for (i, miner_id) in miners.iter().enumerate() {
            result.insert(*miner_id, loyalty_values[i] / max_loyalty);
        }

        result
    }

    /// Calculate normalized bond (sqrt)
    pub fn normalized_bond(&self) -> HashMap<MinerId, f64> {
        let mut result = HashMap::new();
        let miners = self.active_miners();
        
        if miners.is_empty() {
            return result;
        }

        // Calculate sqrt bond for each miner
        let mut sqrt_values = Vec::new();
        for miner_id in &miners {
            let bond = self.miner_registry.bond_for_poci(miner_id);
            sqrt_values.push(bond);
        }

        // Find max sqrt bond
        let mut max_sqrt = 0.0_f64;
        for &val in &sqrt_values {
            if val > max_sqrt {
                max_sqrt = val;
            }
        }
        
        if max_sqrt == 0.0 {
            return result;
        }

        // Normalize
        for (i, miner_id) in miners.iter().enumerate() {
            result.insert(*miner_id, sqrt_values[i] / max_sqrt);
        }

        result
    }

    /// Calculate PoCI for all miners
    pub fn calculate_poci(&self) -> HashMap<MinerId, f64> {
        let norm_shares = self.normalized_shares();
        let norm_loyalty = self.normalized_loyalty();
        let norm_bond = self.normalized_bond();

        let mut poci = HashMap::new();
        let miners = self.active_miners();

        for miner_id in miners {
            let shares = norm_shares.get(&miner_id).unwrap_or(&0.0);
            let loyalty = norm_loyalty.get(&miner_id).unwrap_or(&0.0);
            let bond = norm_bond.get(&miner_id).unwrap_or(&0.0);

            // PoCI = 0.6*shares + 0.2*loyalty + 0.2*bond
            let value = POCI_WEIGHT_SHARES * shares
                + POCI_WEIGHT_LOYALTY * loyalty
                + POCI_WEIGHT_BOND * bond;

            poci.insert(miner_id, value);
        }

        poci
    }

    /// Calculate rewards based on PoCI
    pub fn calculate_rewards(&self, poci: &HashMap<MinerId, f64>) -> HashMap<MinerId, Amount> {
        let mut rewards = HashMap::new();
        
        // Sum all PoCI values
        let mut sum_poci = 0.0_f64;
        for &val in poci.values() {
            sum_poci += val;
        }
        
        if sum_poci == 0.0 {
            return rewards;
        }

        // Calculate rewards
        for (miner_id, &value) in poci {
            let reward = ((value / sum_poci) * (EPOCH_REWARD_LYT as f64)) as Amount;
            rewards.insert(*miner_id, reward);
        }

        rewards
    }

    /// Full epoch reward calculation
    pub fn calculate_epoch_rewards(&self) -> HashMap<MinerId, Amount> {
        let poci = self.calculate_poci();
        self.calculate_rewards(&poci)
    }
}

/// Example from specification (section 7)
pub fn example_from_spec() {
    println!("PoCI calculation example:");
    println!("Miner A: shares=10000, loyalty=100, bond=50M LYT");
    println!("Miner B: shares=2500, loyalty=50, bond=10M LYT");
    println!("Miner C: shares=1600, loyalty=10, bond=0 LYT");
    
    // Calculate manually
    let sqrt_a = (10000.0_f64).sqrt();
    let sqrt_b = (2500.0_f64).sqrt();
    let sqrt_c = (1600.0_f64).sqrt();
    let max_sqrt = sqrt_a.max(sqrt_b).max(sqrt_c);
    
    let norm_shares_a = sqrt_a / max_sqrt;
    let norm_shares_b = sqrt_b / max_sqrt;
    let norm_shares_c = sqrt_c / max_sqrt;
    
    let max_loyalty = 100.0_f64.max(50.0).max(10.0);
    let norm_loyalty_a = 100.0 / max_loyalty;
    let norm_loyalty_b = 50.0 / max_loyalty;
    let norm_loyalty_c = 10.0 / max_loyalty;
    
    let sqrt_bond_a = (50_000_000.0_f64).sqrt();
    let sqrt_bond_b = (10_000_000.0_f64).sqrt();
    let max_sqrt_bond = sqrt_bond_a.max(sqrt_bond_b);
    
    let norm_bond_a = sqrt_bond_a / max_sqrt_bond;
    let norm_bond_b = sqrt_bond_b / max_sqrt_bond;
    let norm_bond_c = 0.0;
    
    let poci_a = 0.6 * norm_shares_a + 0.2 * norm_loyalty_a + 0.2 * norm_bond_a;
    let poci_b = 0.6 * norm_shares_b + 0.2 * norm_loyalty_b + 0.2 * norm_bond_b;
    let poci_c = 0.6 * norm_shares_c + 0.2 * norm_loyalty_c + 0.2 * norm_bond_c;
    
    let sum_poci = poci_a + poci_b + poci_c;
    
    println!("\nResults:");
    println!("PoCI A: {:.4}", poci_a);
    println!("PoCI B: {:.4}", poci_b);
    println!("PoCI C: {:.4}", poci_c);
    println!("Sum PoCI: {:.4}", sum_poci);
    
    let reward_a = (poci_a / sum_poci) * (EPOCH_REWARD_LYT as f64);
    let reward_b = (poci_b / sum_poci) * (EPOCH_REWARD_LYT as f64);
    let reward_c = (poci_c / sum_poci) * (EPOCH_REWARD_LYT as f64);
    
    println!("\nRewards:");
    println!("Miner A: {:.0} LYT ({:.2} ACM)", reward_a, reward_a / 10_000_000.0);
    println!("Miner B: {:.0} LYT ({:.2} ACM)", reward_b, reward_b / 10_000_000.0);
    println!("Miner C: {:.0} LYT ({:.2} ACM)", reward_c, reward_c / 10_000_000.0);
}