//! In-process message bus — the Rust-native replacement for ROS 2 pub/sub.
//!
//! Uses `tokio::sync::broadcast` channels internally.  Each topic gets its
//! own channel.  The trait `MeshBus` defines the interface so a distributed
//! transport (mielin-mesh) can be swapped in later.

use crate::Message;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, Mutex};

#[derive(Debug, Error)]
pub enum MeshError {
    #[error("channel send error: {0}")]
    Send(String),
    #[error("channel recv error: {0}")]
    Recv(String),
}

/// Abstract mesh message bus.
///
/// Implementations:
/// - `InProcessBus` — tokio broadcast channels (single process)
/// - (future) `MielinMeshBus` — mielin-mesh distributed transport
#[async_trait::async_trait]
pub trait MeshBus: Send + Sync {
    /// Publish a message to a topic.
    async fn publish(&self, topic: &str, msg: Message) -> Result<(), MeshError>;

    /// Subscribe to a topic. Returns a receiver.
    async fn subscribe(&self, topic: &str) -> Result<broadcast::Receiver<Message>, MeshError>;
}

/// In-process message bus backed by tokio broadcast channels.
///
/// Clone is cheap — all clones share the same channels.
#[derive(Clone)]
pub struct InProcessBus {
    channels: Arc<Mutex<HashMap<String, broadcast::Sender<Message>>>>,
    capacity: usize,
}

impl InProcessBus {
    /// Create a new bus with the given per-topic buffer capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
            capacity,
        }
    }

    /// Get or create a broadcast sender for the given topic.
    async fn get_or_create_tx(&self, topic: &str) -> broadcast::Sender<Message> {
        let mut channels = self.channels.lock().await;
        if let Some(tx) = channels.get(topic) {
            tx.clone()
        } else {
            let (tx, _) = broadcast::channel(self.capacity);
            channels.insert(topic.to_string(), tx.clone());
            tx
        }
    }
}

#[async_trait::async_trait]
impl MeshBus for InProcessBus {
    async fn publish(&self, topic: &str, msg: Message) -> Result<(), MeshError> {
        let tx = self.get_or_create_tx(topic).await;
        tx.send(msg).map_err(|e| MeshError::Send(e.to_string()))?;
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<broadcast::Receiver<Message>, MeshError> {
        let tx = self.get_or_create_tx(topic).await;
        Ok(tx.subscribe())
    }
}
