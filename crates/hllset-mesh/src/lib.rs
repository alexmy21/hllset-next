//! Mesh networking for HLLSet — replaces ROS 2 pub/sub with Rust-native message bus.
//!
//! # Architecture
//!
//! The mesh crate provides an in-process message bus that mirrors the ROS 2
//! pub/sub model but runs entirely in Rust.  Each "node" (algebra, worker,
//! noether controller) is a plain Rust struct that spawns tokio tasks.
//!
//! # Future: mielin-mesh integration
//!
//! The `MeshBus` trait is designed so that a mielin-mesh backend can be
//! plugged in later — same trait, same message types, distributed transport.
//!
//! # Example
//!
//! ```rust,no_run
//! use hllset_mesh::{InProcessBus, MeshBus, Message};
//!
//! #[tokio::main]
//! async fn main() {
//!     let bus = InProcessBus::new(64);
//!     let mut rx = bus.subscribe("hllset/ingest_text").await.unwrap();
//!
//!     bus.publish("hllset/ingest_text", Message::text("test", "hello world")).await.unwrap();
//!
//!     let msg = rx.recv().await.unwrap();
//!     println!("Received: {:?}", msg);
//! }
//! ```

use serde::{Deserialize, Serialize};

pub mod algebra_node;
pub mod bus;
pub mod noether_controller;
pub mod worker_node;

pub use algebra_node::AlgebraNode;
pub use bus::{InProcessBus, MeshBus, MeshError};
pub use noether_controller::NoetherController;
pub use worker_node::WorkerNode;

/// A message on the mesh — equivalent to a ROS 2 topic message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Topic (e.g. "hllset/ingest_text", "hllset/hllset_key")
    pub topic: String,
    /// JSON payload
    pub payload: serde_json::Value,
}

impl Message {
    /// Create a message with a text payload.
    pub fn text(topic: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            payload: serde_json::Value::String(text.into()),
        }
    }

    /// Create a message from a JSON value.
    pub fn json(topic: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            topic: topic.into(),
            payload,
        }
    }
}

/// Compute request — equivalent to what ROS 2 WorkerNode receives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequest {
    pub op: String,
    pub args: serde_json::Value,
    pub request_id: String,
}

/// Compute result — equivalent to what ROS 2 WorkerNode publishes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResult {
    pub op: String,
    pub request_id: String,
    pub result: serde_json::Value,
    pub worker: String,
}
