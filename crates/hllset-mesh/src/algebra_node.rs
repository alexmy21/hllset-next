//! Algebra node — Rust-native replacement for ROS 2 `algebra_node.py`.
//!
//! Ingests text snippets, tokenizes them into HLLSets, and publishes keys
//! on the mesh. Uses hllset-dsl LatticeElement directly (no subprocess).

use crate::bus::MeshBus;
use crate::Message;
use hllset_dsl::LatticeElement;
use hllset_storage::MemoryStorage;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Algebra node that ingests text and produces HLLSet keys.
///
/// Equivalent to the ROS 2 `AlgebraNode` but:
/// - No Python subprocess — calls hllset-core/hllset-dsl directly
/// - No ROS topics — uses the mesh bus
/// - In-memory store for key→token lookup
pub struct AlgebraNode {
    bus: Arc<dyn MeshBus>,
    store: Arc<Mutex<Store>>,
}

struct Store {
    /// key → list of tokens
    entries: std::collections::HashMap<String, Vec<String>>,
    storage: MemoryStorage,
}

impl AlgebraNode {
    /// Create a new algebra node attached to the given mesh bus.
    pub fn new(bus: Arc<dyn MeshBus>) -> Self {
        Self {
            bus,
            store: Arc::new(Mutex::new(Store {
                entries: std::collections::HashMap::new(),
                storage: MemoryStorage::new(),
            })),
        }
    }

    /// Ingest raw text — tokenize, create HLLSet, publish key.
    pub async fn ingest_text(&self, text: &str) -> Result<String, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err("empty text".to_string());
        }

        // Tokenize: split by whitespace, lowercase
        let tokens: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let element = LatticeElement::from_tokens(&tokens);
        let key = element.key().to_string();

        // Store locally
        {
            let mut store = self.store.lock().await;
            store.entries.insert(key.clone(), tokens);
        }

        // Publish on mesh
        let msg = Message::text("hllset/hllset_key", &key);
        self.bus
            .publish("hllset/hllset_key", msg)
            .await
            .map_err(|e| format!("publish failed: {e}"))?;

        info!("Ingested text -> key={}", key);
        Ok(key)
    }

    /// Ingest pre-tokenized list — create HLLSet, publish key.
    pub async fn ingest_tokens(&self, tokens: &[String]) -> Result<String, String> {
        if tokens.is_empty() {
            return Err("empty tokens".to_string());
        }

        let element = LatticeElement::from_tokens(tokens);
        let key = element.key().to_string();

        {
            let mut store = self.store.lock().await;
            store.entries.insert(key.clone(), tokens.to_vec());
        }

        let msg = Message::text("hllset/hllset_key", &key);
        self.bus
            .publish("hllset/hllset_key", msg)
            .await
            .map_err(|e| format!("publish failed: {e}"))?;

        info!("Ingested tokens -> key={}", key);
        Ok(key)
    }

    /// Get stored keys.
    pub async fn stored_keys(&self) -> Vec<String> {
        let store = self.store.lock().await;
        store.entries.keys().cloned().collect()
    }
}
