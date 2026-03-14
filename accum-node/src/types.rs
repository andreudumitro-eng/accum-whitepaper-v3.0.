use serde::{Deserialize, Serialize};
use core::cmp::Ordering;

pub type Hash32 = [u8; 32];
pub type MinerId = [u8; 20];
pub type Amount = u64;
pub type Height = u64;
pub type EpochIndex = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target(pub Hash32);

impl Target {
    /// Check if hash meets target
    pub fn is_met_by(&self, hash: &Hash32) -> bool {
        for i in 0..32 {
            match hash[i].cmp(&self.0[i]) {
                Ordering::Less => return true,
                Ordering::Greater => return false,
                Ordering::Equal => continue,
            }
        }
        true
    }

    /// Scale target by factor (for difficulty adjustment)
    pub fn scaled(&self, factor: f64) -> Self {
        if factor <= 0.0 {
            return Target([0u8; 32]);
        }
        if factor == 1.0 {
            return *self;
        }

        // Use floating point approximation for difficulty adjustment
        // This is sufficient for the ±25% range
        let mut result = [0u8; 32];
        
        // Simple approximation: scale first few bytes
        for i in 0..4 {
            let val = ((self.0[i] as f64) * factor).round() as u8;
            result[i] = val.min(0xFF);
        }
        
        // Copy remaining bytes
        for i in 4..32 {
            result[i] = self.0[i];
        }
        
        Target(result)
    }

    /// Convert to compact representation (for block header)
    pub fn compact(&self) -> u32 {
        let bytes = self.0;
        
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

    /// Create target from compact representation
    pub fn from_compact(compact: u32) -> Self {
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
}