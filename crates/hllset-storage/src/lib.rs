//! Content-addressed storage for HLLSets.
//!
//! Provides a sync `Storage` trait with:
//! - `MemoryStorage` — in-memory HashMap (dev/testing)
//! - `IpfrsNativeStorage` — sled-backed with ipfrs-core content-addressing
//!
//! # Example
//!
//! ```rust
//! use hllset_storage::{MemoryStorage, Storage};
//!
//! let store = MemoryStorage::new();
//! store.store("h:abc123", b"hello").unwrap();
//! let data = store.load("h:abc123").unwrap();
//! assert_eq!(data, Some(b"hello".to_vec()));
//! ```

pub mod cache;
pub mod ipfs;
pub mod memory;
pub mod storage;

pub use cache::CacheStorage;
pub use ipfs::IpfrsNativeStorage;
pub use memory::MemoryStorage;
pub use storage::{Result, Storage, StorageError};
