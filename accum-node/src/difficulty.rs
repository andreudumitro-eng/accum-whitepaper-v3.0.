//! Difficulty adjustment algorithm

use crate::constants::*;
use crate::types::{Hash32, Target};
use std::cmp::Ordering;

/// Difficulty adjustment every 120 blocks
pub fn adjust_difficulty(
    old_target: &Target,
    actual_time_span: u64,
) -> Target {
    // new_target = old_target × (actual_time_span / 7200)
    let factor = actual_time_span as f64 / TARGET_ADJUSTMENT_TIME as f64;
    
    // Clamp to ±25%
    let factor = if factor > 1.25 {
        1.25
    } else if factor < 0.75 {
        0.75
    } else {
        factor
    };
    
    old_target.scaled(factor)
}

/// Calculate actual time span from block timestamps
pub fn calculate_time_span(
    timestamps: &[u64],
    interval: usize,
) -> Option<u64> {
    if timestamps.len() < interval + 1 {
        return None;
    }
    
    let start = timestamps[timestamps.len() - interval - 1];
    let end = timestamps[timestamps.len() - 1];
    
    Some(end - start)
}

/// Compact target representation
pub fn compact_from_target(target: &Target) -> u32 {
    let bytes = target.0;
    
    // Find first non-zero byte
    let mut size = 32;
    for (i, &b) in bytes.iter().enumerate() {
        if b != 0 {
            size = i;
            break;
        }
    }
    
    if size == 32 {
        return 0;
    }
    
    let exponent = (32 - size) as u32;
    let mantissa = ((bytes[size] as u32) << 16) |
                   ((bytes[size + 1] as u32) << 8) |
                   (bytes[size + 2] as u32);
    
    (exponent << 24) | mantissa
}

/// Target from compact representation
pub fn target_from_compact(compact: u32) -> Target {
    let exponent = (compact >> 24) & 0xFF;
    let mantissa = compact & 0x00FFFFFF;
    
    let mut bytes = [0u8; 32];
    
    if exponent <= 3 {
        bytes[31 - exponent as usize] = (mantissa >> 16) as u8;
        if exponent > 1 {
            bytes[32 - exponent as usize] = (mantissa >> 8) as u8;
        }
        if exponent > 2 {
            bytes[33 - exponent as usize] = mantissa as u8;
        }
    } else {
        let pos = 32 - exponent as usize;
        if pos < 32 {
            bytes[pos] = (mantissa >> 16) as u8;
            if pos + 1 < 32 {
                bytes[pos + 1] = (mantissa >> 8) as u8;
            }
            if pos + 2 < 32 {
                bytes[pos + 2] = mantissa as u8;
            }
        }
    }
    
    Target(bytes)
}

/// Check if hash meets target (PoW valid)
pub fn hash_meets_target(hash: &Hash32, target: &Target) -> bool {
    for i in 0..32 {
        match hash[i].cmp(&target.0[i]) {
            Ordering::Less => return true,
            Ordering::Greater => return false,
            Ordering::Equal => continue,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_difficulty_adjustment() {
        let target = Target([0xFF; 32]);
        
        // Faster blocks (half time) -> harder
        let adjusted = adjust_difficulty(&target, 3600);
        assert!(adjusted.0[0] < target.0[0]);
        
        // Slower blocks (double time) -> easier
        let adjusted = adjust_difficulty(&target, 14400);
        assert!(adjusted.0[0] > target.0[0]);
        
        // Clamp test
        let adjusted = adjust_difficulty(&target, 36000); // 5x slower
        assert!(adjusted.0[0] <= (target.0[0] as f64 * 1.25) as u8);
    }
    
    #[test]
    fn test_compact_conversion() {
        let target = Target([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                             0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF]);
        
        let compact = compact_from_target(&target);
        let recovered = target_from_compact(compact);
        
        assert_eq!(target.0[29], recovered.0[29]);
        assert_eq!(target.0[30], recovered.0[30]);
        assert_eq!(target.0[31], recovered.0[31]);
    }
}