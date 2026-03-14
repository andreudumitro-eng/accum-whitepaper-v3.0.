//! P2P protocol messages and handling

use crate::block::Block;
use crate::error::Error;
use crate::share::SharePacket;
use crate::types::{Hash32, MinerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Version handshake message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMessage {
    pub version: u32,
    pub capabilities: u64,
    pub timestamp: u64,
    pub user_agent: String,
    pub start_height: u64,
    pub nonce: u64,
}

/// Inventory type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InvType {
    Block = 0,
    Transaction = 1,
    Share = 2,
}

/// Inventory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvItem {
    pub inv_type: InvType,
    pub hash: Hash32,
}

/// Inventory message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvMessage {
    pub items: Vec<InvItem>,
}

/// GetData message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDataMessage {
    pub items: Vec<InvItem>,
}

/// Block message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMessage {
    pub block: Block,
}

/// Transaction message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxMessage {
    pub tx: Vec<u8>, // Simplified
}

/// Share message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareMessage {
    pub share: SharePacket,
}

/// Epoch commit message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochCommitMessage {
    pub epoch_index: u32,
    pub root: Hash32,
    pub timestamp: u64,
}

/// GetShares request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSharesMessage {
    pub epoch_index: u32,
    pub miner_id_list: Vec<MinerId>,
    pub offset: u32,
}

/// Shares reply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharesReplyMessage {
    pub shares_batch: Vec<SharePacket>,
}

/// Compact block (BIP152 style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactBlockMessage {
    pub header_hash: Hash32,
    pub nonce: u64,
    pub short_ids: Vec<u64>,
    pub prefilled_txs: Vec<usize>,
}

/// Ping message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingMessage {
    pub nonce: u64,
}

/// Pong message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongMessage {
    pub nonce: u64,
}

/// All possible P2P messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PMessage {
    Version(VersionMessage),
    Verack,
    Inv(InvMessage),
    GetData(GetDataMessage),
    Block(BlockMessage),
    Tx(TxMessage),
    Share(ShareMessage),
    EpochCommit(EpochCommitMessage),
    GetShares(GetSharesMessage),
    SharesReply(SharesReplyMessage),
    CompactBlock(CompactBlockMessage),
    Ping(PingMessage),
    Pong(PongMessage),
}

/// Peer connection state
#[derive(Debug, Clone)]
pub struct Peer {
    pub address: String,
    pub version: Option<VersionMessage>,
    pub connected_at: u64,
    pub last_seen: u64,
    pub share_count: u32,
    pub last_share_time: u64,
    pub ban_until: u64,
    pub misbehavior_count: u32,
}

impl Peer {
    /// Create new peer
    pub fn new(address: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            address,
            version: None,
            connected_at: now,
            last_seen: now,
            share_count: 0,
            last_share_time: 0,
            ban_until: 0,
            misbehavior_count: 0,
        }
    }

    /// Check if peer is banned
    pub fn is_banned(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.ban_until > now
    }

    /// Ban peer for minutes
    pub fn ban(&mut self, minutes: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.ban_until = now + minutes * 60;
    }

    /// Record share reception
    pub fn record_share(&mut self) -> Result<(), Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Reset counter if minute passed
        if now - self.last_share_time >= 60 {
            self.share_count = 0;
        }
        
        // Check rate limit
        if self.share_count >= 100 {
            self.ban(5); // 5 minute ban
            return Err(Error::P2p);
        }
        
        self.share_count += 1;
        self.last_share_time = now;
        self.last_seen = now;
        
        Ok(())
    }
}

/// P2P network manager
#[derive(Debug)]
pub struct P2PManager {
    peers: HashMap<String, Peer>,
    max_peers: usize,
    message_history: Vec<(u64, String, P2PMessage)>,
    max_history: usize,
}

impl P2PManager {
    /// Create new P2P manager
    pub fn new(max_peers: usize) -> Self {
        Self {
            peers: HashMap::new(),
            max_peers,
            message_history: Vec::new(),
            max_history: 1000,
        }
    }

    /// Add or update peer
    pub fn add_peer(&mut self, address: String) {
        if self.peers.len() >= self.max_peers {
            // Remove oldest inactive peer
            if let Some(oldest) = self.find_oldest_peer() {
                self.peers.remove(&oldest);
            }
        }
        
        self.peers.entry(address.clone())
            .or_insert_with(|| Peer::new(address));
    }

    /// Find oldest peer by last_seen
    fn find_oldest_peer(&self) -> Option<String> {
        let mut oldest = None;
        let mut oldest_time = u64::MAX;
        
        for (addr, peer) in &self.peers {
            if peer.last_seen < oldest_time {
                oldest_time = peer.last_seen;
                oldest = Some(addr.clone());
            }
        }
        
        oldest
    }

    /// Process incoming message
    pub fn process_message(&mut self, from: String, msg: P2PMessage) -> Result<Option<P2PMessage>, Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Check if peer is banned
        if let Some(peer) = self.peers.get(&from) {
            if peer.is_banned() {
                return Err(Error::P2p);
            }
        }
        
        // Record message
        self.message_history.push((now, from.clone(), msg.clone()));
        if self.message_history.len() > self.max_history {
            self.message_history.remove(0);
        }
        
        // Handle message
        match msg {
            P2PMessage::Version(version) => {
                self.handle_version(from, version)
            }
            P2PMessage::Verack => {
                Ok(None)
            }
            P2PMessage::Inv(inv) => {
                self.handle_inv(from, inv)
            }
            P2PMessage::GetData(getdata) => {
                self.handle_getdata(from, getdata)
            }
            P2PMessage::Share(share) => {
                self.handle_share(from, share)
            }
            P2PMessage::Ping(ping) => {
                Ok(Some(P2PMessage::Pong(PongMessage { nonce: ping.nonce })))
            }
            P2PMessage::Pong(_) => {
                Ok(None)
            }
            _ => {
                // Other messages handled elsewhere
                Ok(None)
            }
        }
    }

    /// Handle version message
    fn handle_version(&mut self, from: String, version: VersionMessage) -> Result<Option<P2PMessage>, Error> {
        if let Some(peer) = self.peers.get_mut(&from) {
            peer.version = Some(version);
            peer.last_seen = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        Ok(Some(P2PMessage::Verack))
    }

    /// Handle inventory message
    /// Handle inventory message
fn handle_inv(&mut self, _from: String, inv: InvMessage) -> Result<Option<P2PMessage>, Error> {
    // Request unknown items
    let mut to_request = Vec::new();
    for item in inv.items {
        // In real implementation: check if we have it
        to_request.push(item);
    }
    
    if !to_request.is_empty() {
        Ok(Some(P2PMessage::GetData(GetDataMessage { items: to_request })))
    } else {
        Ok(None)
    }
}

    /// Handle getdata message
fn handle_getdata(&mut self, _from: String, _getdata: GetDataMessage) -> Result<Option<P2PMessage>, Error> {
    // In real implementation: lookup and return items
    Ok(None)
}

    /// Handle share message
fn handle_share(&mut self, from: String, _share: ShareMessage) -> Result<Option<P2PMessage>, Error> {
    if let Some(peer) = self.peers.get_mut(&from) {
        peer.record_share()?;
    }
    
    // Share validated elsewhere
    Ok(None)
}

    /// Get peer count
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Get banned peers
    pub fn banned_peers(&self) -> Vec<String> {
        self.peers.iter()
            .filter(|(_, p)| p.is_banned())
            .map(|(addr, _)| addr.clone())
            .collect()
    }

    /// Clean up old peers
    pub fn cleanup(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Remove peers inactive for > 1 hour
        self.peers.retain(|_, peer| now - peer.last_seen < 3600);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_peer_rate_limit() {
        let mut peer = Peer::new("127.0.0.1:8333".to_string());
        
        // 100 shares should be OK
        for i in 0..100 {
            assert!(peer.record_share().is_ok());
        }
        
        // 101st share should ban
        assert!(peer.record_share().is_err());
        assert!(peer.is_banned());
    }
    
    #[test]
    fn test_p2p_manager() {
        let mut manager = P2PManager::new(10);
        
        manager.add_peer("127.0.0.1:8333".to_string());
        assert_eq!(manager.peer_count(), 1);
        
        let version = VersionMessage {
            version: 1,
            capabilities: 0,
            timestamp: 1741353600,
            user_agent: "ACCUM node".to_string(),
            start_height: 0,
            nonce: 12345,
        };
        
        let response = manager.process_message(
            "127.0.0.1:8333".to_string(),
            P2PMessage::Version(version)
        ).unwrap();
        
        assert!(matches!(response, Some(P2PMessage::Verack)));
    }
}