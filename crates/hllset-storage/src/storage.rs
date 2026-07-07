//! Sync storage trait for HLLSet data.

/// Storage errors.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("backend error: {0}")]
    Backend(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("serialization: {0}")]
    Serialization(String),
}

/// Convenience result type.
pub type Result<T> = std::result::Result<T, StorageError>;

/// Content-addressed storage backend.
///
/// Keys follow HLLSet conventions: `h:<sha1>` for heterogeneous data,
/// `c:<sha1>` for homogeneous/catalog data.
pub trait Storage {
    /// Store raw bytes under a key. Returns the key on success.
    fn store(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Load raw bytes by key. Returns `None` if not found.
    fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Check whether a key exists in storage.
    fn exists(&self, key: &str) -> Result<bool>;

    /// Delete a key and its data.
    fn delete(&self, key: &str) -> Result<bool>;

    /// List keys matching a prefix (e.g., "h:" or "c:").
    fn list(&self, prefix: &str) -> Result<Vec<String>>;

    /// Pin a key — prevent garbage collection. Idempotent.
    /// Default: no-op (backends that don't support GC don't need pins).
    fn pin(&self, _key: &str) -> Result<()> { Ok(()) }

    /// Unpin a key — allow garbage collection. Idempotent.
    fn unpin(&self, _key: &str) -> Result<()> { Ok(()) }

    /// Garbage collect: remove all unpinned keys. Returns removed keys.
    /// Default: returns empty (backends without GC support).
    fn gc(&self) -> Result<Vec<String>> { Ok(Vec::new()) }
}
