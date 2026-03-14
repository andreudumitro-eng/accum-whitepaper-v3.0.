//! Реальный P2P сетевой слой на libp2p

use crate::block::Block;
use crate::storage::Storage;
use futures::StreamExt;
use libp2p::{
    core::upgrade,
    gossipsub,
    identity,
    kad,
    noise,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, Multiaddr, PeerId, Transport,
};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc;

// Настройки сети
const NETWORK_NAME: &[u8] = b"accum-network-v1";

/// События сети, которые обрабатывает нода
#[derive(Debug)]
pub enum NetworkEvent {
    NewBlock(Block),
    NewPeer(PeerId),
    PeerDisconnected(PeerId),
}

/// Поведение нашей сети (комбинация протоколов)
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "AccumBehaviourEvent")]
pub struct AccumBehaviour {
    gossipsub: gossipsub::Behaviour,
    kad: kad::Behaviour<kad::store::MemoryStore>,
}

/// События от поведения
#[derive(Debug)]
pub enum AccumBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Kad(kad::Event),
}

impl From<gossipsub::Event> for AccumBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        AccumBehaviourEvent::Gossipsub(event)
    }
}

impl From<kad::Event> for AccumBehaviourEvent {
    fn from(event: kad::Event) -> Self {
        AccumBehaviourEvent::Kad(event)
    }
}

impl AccumBehaviour {
    pub fn new(local_peer_id: PeerId) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // Настройка Gossipsub (для распространения блоков)
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|e| format!("Gossipsub config error: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(identity::Keypair::generate_ed25519()),
            gossipsub_config,
        ).map_err(|e| format!("Gossipsub error: {}", e))?;

        // Настройка Kademlia (для поиска пиров)
        let kad = kad::Behaviour::new(
            local_peer_id,
            kad::store::MemoryStore::new(local_peer_id),
        );

        Ok(Self { gossipsub, kad })
    }
}

/// Наша P2P нода
pub struct P2PNode {
    swarm: Swarm<AccumBehaviour>,
    event_sender: mpsc::UnboundedSender<NetworkEvent>,
    storage: Arc<Storage>,
}

impl P2PNode {
    /// Создать новую P2P ноду
    pub async fn new(storage: Arc<Storage>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // Генерируем уникальный ключ для этой ноды
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        println!("Local peer id: {}", local_peer_id);

        // Настройка транспорта (TCP + шифрование)
        let transport = tcp::tokio::Transport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(libp2p::yamux::Config::default())
            .boxed();

        // Создаем поведение
        let behaviour = AccumBehaviour::new(local_peer_id)?;

        // Канал для событий
        let (event_sender, _) = mpsc::unbounded_channel();

        // Создаем Swarm (главный объект сети)
        let swarm = Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            libp2p::swarm::Config::with_tokio_executor()
        );

        Ok(Self {
            swarm,
            event_sender,
            storage,
        })
    }

    /// Запустить ноду и слушать на указанном адресе
    pub async fn start(&mut self, listen_addr: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let addr: Multiaddr = listen_addr.parse()?;
        self.swarm.listen_on(addr)?;
        println!("Listening on: {}", listen_addr);
        Ok(())
    }

    /// Подключиться к другой ноде
    pub async fn dial(&mut self, addr: Multiaddr) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.swarm.dial(addr)?;
        Ok(())
    }

    /// Опубликовать блок в сеть (Gossipsub)
    pub async fn publish_block(&mut self, block: &Block) -> Result<(), Box<dyn Error + Send + Sync>> {
        let topic = gossipsub::IdentTopic::new("blocks".to_string());
        let data = bincode::serialize(block)?;
        self.swarm.behaviour_mut().gossipsub.publish(topic, data)?;
        Ok(())
    }

    /// Запустить главный цикл обработки событий сети
    pub async fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let blocks_topic = gossipsub::IdentTopic::new("blocks".to_string());

        self.swarm.behaviour_mut().gossipsub.subscribe(&blocks_topic)?;

        println!("Node is running...");

        loop {
            match self.swarm.next().await {
                Some(SwarmEvent::NewListenAddr { address, .. }) => {
                    println!("Listening on {}", address);
                }
                Some(SwarmEvent::Behaviour(AccumBehaviourEvent::Gossipsub(gossipsub::Event::Message { 
                    propagation_source: _,
                    message_id: _,
                    message,
                }))) => {
                    // Получили новое сообщение (блок)
                    if let Ok(block) = bincode::deserialize::<Block>(&message.data) {
                        println!("Received new block: {:?}...", &block.header.hash()[0..4]);
                        
                        // Сохраняем блок в базу
                        let height = block.header.nonce;
                        let _ = self.storage.save_block(height as u64, &block);
                        
                        // Отправляем событие в основную программу
                        let _ = self.event_sender.send(NetworkEvent::NewBlock(block));
                    }
                }
                Some(SwarmEvent::Behaviour(AccumBehaviourEvent::Kad(_))) => {
                    // Обработка Kademlia событий
                }
                Some(SwarmEvent::ConnectionEstablished { peer_id, .. }) => {
                    println!("Connected to peer: {}", peer_id);
                    let _ = self.event_sender.send(NetworkEvent::NewPeer(peer_id));
                }
                Some(SwarmEvent::ConnectionClosed { peer_id, .. }) => {
                    println!("Disconnected from peer: {}", peer_id);
                    let _ = self.event_sender.send(NetworkEvent::PeerDisconnected(peer_id));
                }
                _ => {}
            }
        }
    }
}