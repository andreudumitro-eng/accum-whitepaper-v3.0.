mod constants;
mod error;
mod types;
mod crypto;
mod block;
mod share;
mod miner;
mod consensus;
mod pool;
mod difficulty;
mod p2p;
mod epoch;
mod genesis;
mod storage;
mod network;

use constants::MINIMUM_BOND_LYT;
use storage::Storage;
use std::sync::Arc;

struct NodeState {
    db: Arc<Storage>,
    current_height: u64,
    current_epoch: u32,
}

impl NodeState {
    fn new(db: Arc<Storage>) -> Self {
        Self {
            db,
            current_height: 0,
            current_epoch: 1,
        }
    }

    fn load_chain(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Loading chain from database ===\n");

        let mut height = 0;
        loop {
            match self.db.get_block(height) {
                Ok(Some(block)) => {
                    println!("  Block {} found: {:?}...", height, &block.header.hash()[0..4]);
                    height += 1;
                }
                Ok(None) => break,
                Err(e) => {
                    println!("  Error reading block {}: {}", height, e);
                    break;
                }
            }
        }

        if height == 0 {
            println!("  No blocks found, starting from genesis");
            self.current_height = 0;
        } else {
            self.current_height = height - 1;
            println!("\n✅ Chain loaded: {} blocks total", height);
            println!("   Current height: {}", self.current_height);

            if let Ok(Some(last_block)) = self.db.get_block(self.current_height) {
                println!("   Last block hash: {:?}...", &last_block.header.hash()[0..4]);
                println!("   Last block timestamp: {}", last_block.header.timestamp);
                self.current_epoch = last_block.header.epoch_index;
                println!("   Current epoch: {}", self.current_epoch);
            }
        }

        Ok(())
    }

    fn get_last_block(&self) -> Option<block::Block> {
        self.db.get_block(self.current_height).ok().flatten()
    }

    fn get_last_hash(&self) -> [u8; 32] {
        if let Ok(Some(block)) = self.db.get_block(self.current_height) {
            block.header.hash()
        } else {
            [0u8; 32]
        }
    }

    fn validate_pow(&self, header: &block::BlockHeader) -> bool {
        let hash = header.hash();
        let target = header.difficulty;
        
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

    fn add_block(&mut self, block: block::Block) -> Result<(), Box<dyn std::error::Error>> {
        let new_height = self.current_height + 1;
        
        println!("\n=== Adding new block at height {} ===\n", new_height);
        
        let expected_prev = self.get_last_hash();
        if block.header.prev_hash != expected_prev {
            println!("❌ Invalid previous hash");
            return Err("Invalid previous hash".into());
        }
        
        if !self.validate_pow(&block.header) {
            println!("❌ Invalid Proof of Work");
            return Err("Invalid PoW".into());
        }
        
        match self.db.save_block(new_height, &block) {
            Ok(_) => {
                println!("✅ Block saved to database");
                println!("   Hash: {:?}...", &block.header.hash()[0..4]);
                
                self.current_height = new_height;
                self.current_epoch = block.header.epoch_index;
                
                // Проверяем нужен ли adjustment
                if self.current_height % 120 == 0 {
                    self.check_difficulty_adjustment();
                }
                
                Ok(())
            }
            Err(e) => {
                println!("❌ Failed to save block: {}", e);
                Err(e.into())
            }
        }
    }

    fn create_block(&self) -> block::Block {
        let last_hash = self.get_last_hash();
        let new_height = self.current_height + 1;
        
        let difficulty = self.get_last_block()
            .map(|b| b.header.difficulty)
            .unwrap_or_else(|| {
                use genesis::create_genesis_block;
                create_genesis_block().header.difficulty
            });
        
        let header = block::BlockHeader {
            version: 1,
            prev_hash: last_hash,
            merkle_root: [0u8; 32],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            difficulty,
            nonce: new_height,
            epoch_index: 1,
        };
        
        block::Block {
            header,
            transactions: vec![],
        }
    }

    fn current_header(&self) -> block::BlockHeader {
        let last_hash = self.get_last_hash();
        let new_height = self.current_height + 1;
        
        let difficulty = self.get_last_block()
            .map(|b| b.header.difficulty)
            .unwrap_or_else(|| {
                use genesis::create_genesis_block;
                create_genesis_block().header.difficulty
            });
        
        block::BlockHeader {
            version: 1,
            prev_hash: last_hash,
            merkle_root: [0u8; 32],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            difficulty,
            nonce: 0,
            epoch_index: self.current_epoch,
        }
    }

    // =========================================
    // Difficulty Adjustment (по спецификации)
    // =========================================
    
    fn get_last_timestamps(&self, count: usize) -> Vec<u64> {
        let mut timestamps = Vec::new();
        let start = if self.current_height > count as u64 {
            self.current_height - count as u64 + 1
        } else {
            0
        };
        
        for height in start..=self.current_height {
            if let Ok(Some(block)) = self.db.get_block(height) {
                timestamps.push(block.header.timestamp);
            }
        }
        timestamps
    }

    fn calculate_new_difficulty(&self) -> types::Target {
        if self.current_height < 120 {
            return self.get_last_block()
                .map(|b| b.header.difficulty)
                .unwrap_or_else(|| {
                    use genesis::create_genesis_block;
                    create_genesis_block().header.difficulty
                });
        }
        
        let timestamps = self.get_last_timestamps(121);
        if timestamps.len() < 121 {
            return self.get_last_block().unwrap().header.difficulty;
        }
        
        let first = timestamps[0];
        let last = timestamps[timestamps.len() - 1];
        let actual_time = last - first;
        
        println!("\n📊 Difficulty Adjustment:");
        println!("   Time for last 120 blocks: {}s (target: 7200s)", actual_time);
        
        let old_target = self.get_last_block().unwrap().header.difficulty;
        let factor = actual_time as f64 / 7200.0;
        
        let factor = if factor > 1.25 {
            println!("   ⚠️  Clamping +25% (was +{:.1}%)", (factor - 1.0) * 100.0);
            1.25
        } else if factor < 0.75 {
            println!("   ⚠️  Clamping -25% (was {:.1}%)", (factor - 1.0) * 100.0);
            0.75
        } else {
            factor
        };
        
        println!("   Adjustment factor: {:.3}", factor);
        
        old_target.scaled(factor)
    }

    fn check_difficulty_adjustment(&mut self) {
        if self.current_height > 0 && self.current_height % 120 == 0 {
            let new_target = self.calculate_new_difficulty();
            println!("   New difficulty: {:?}...", &new_target.0[0..4]);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ACCUM node starting...\n");

    // =========================================
    // 1. Открываем базу
    // =========================================
    let db = Arc::new(Storage::open("./data")?);
    println!("✅ Database opened successfully");

    // =========================================
    // 2. Создаём состояние ноды
    // =========================================
    let mut node = NodeState::new(db.clone());

    // =========================================
    // 3. Загружаем цепочку
    // =========================================
    node.load_chain()?;

    // =========================================
    // 4. Если цепочка пуста - создаём Genesis
    // =========================================
    if node.current_height == 0 {
        println!("\n=== Creating Genesis Block ===\n");

        use genesis::create_genesis_block;
        let genesis = create_genesis_block();

        match node.db.get_block(0) {
            Ok(Some(_)) => {
                println!("✅ Genesis block already exists");
            }
            _ => {
                match node.db.save_block(0, &genesis) {
                    Ok(_) => {
                        println!("✅ Genesis block saved");
                        node.current_height = 0;
                    }
                    Err(e) => {
                        println!("❌ Failed to save genesis: {}", e);
                        return Ok(());
                    }
                }
            }
        }
    }

    // =========================================
    // 5. Показываем текущее состояние
    // =========================================
    println!("\n=== Current Node State ===");
    println!("Current height: {}", node.current_height);

    // =========================================
    // 6. Добавляем тестовый блок
    // =========================================
    println!("\n=== Adding test block ===\n");
    
    let new_block = node.create_block();
    let _ = node.add_block(new_block);

    // =========================================
    // 7. Запускаем сеть
    // =========================================
    println!("\n=== Starting P2P Network ===\n");
    
    let mut p2p_node = match network::P2PNode::new(db.clone()).await {
        Ok(node) => node,
        Err(e) => {
            println!("❌ Failed to create P2P node: {}", e);
            return Ok(());
        }
    };

    if let Err(e) = p2p_node.start("/ip4/0.0.0.0/tcp/9000").await {
        println!("❌ Failed to start P2P: {}", e);
        return Ok(());
    }

    let network_handle = tokio::spawn(async move {
        if let Err(e) = p2p_node.run().await {
            eprintln!("Network error: {}", e);
        }
    });

    // =========================================
    // 8. Запускаем майнер в отдельном потоке
    // =========================================
    println!("\n=== Starting Miner ===\n");
    
    let miner_db = db.clone();
    let miner_handle = tokio::spawn(async move {
        let mut miner_node = NodeState::new(miner_db);
        
        if let Err(e) = miner_node.load_chain() {
            println!("❌ Miner failed to load chain: {}", e);
            return;
        }
        
        println!("⚡ Mining started with real difficulty from chain");
        
        loop {
            let mut header = miner_node.current_header();
            let target = header.difficulty;
            let start_time = std::time::Instant::now();
            
            for nonce in 0..1_000_000 {
                header.nonce = nonce;
                let hash = header.hash();
                
                let mut is_block = true;
                for i in 0..32 {
                    if hash[i] >= target.0[i] {
                        is_block = false;
                        break;
                    }
                }
                
                if is_block {
                    let elapsed = start_time.elapsed();
                    println!("\n🎉 FOUND BLOCK! nonce: {} (time: {:.2}s)", nonce, elapsed.as_secs_f64());
                    println!("   Hash: {:?}...", &hash[0..4]);
                    println!("   Difficulty: {:?}...", &target.0[0..4]);
                    
                    let block = block::Block {
                        header: header.clone(),
                        transactions: vec![],
                    };
                    
                    if let Err(e) = miner_node.add_block(block.clone()) {
                        println!("   ❌ Failed to save block: {}", e);
                    } else {
                        println!("   ✅ Block saved");
                        
                        if let Some(last) = miner_node.get_last_block() {
                            let time_between = last.header.timestamp - header.timestamp;
                            println!("   Time since last block: {}s", time_between);
                        }
                    }
                    
                    break;
                }
                
                if nonce % 10000 == 0 && nonce > 0 {
                    let elapsed = start_time.elapsed();
                    println!("   Nonce: {}, hashrate: {:.0} H/s, difficulty: {:?}...", 
                             nonce, 
                             nonce as f64 / elapsed.as_secs_f64(),
                             &target.0[0..2]);
                }
                
                if nonce % 1000 == 0 {
                    tokio::task::yield_now().await;
                }
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    // =========================================
    // 9. Ждём остановки
    // =========================================
    println!("\nNode is running. Press Ctrl+C to stop.\n");
    tokio::signal::ctrl_c().await?;
    
    println!("\n✅ Node shutdown complete");
    
    let _ = network_handle.await;
    let _ = miner_handle.await;
    
    Ok(())
}