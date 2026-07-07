//! In-memory storage backend for development and testing.

use crate::storage::{Result, Storage, StorageError};
use std::collections::{BTreeMap, HashSet};

/// Content-addressed in-memory storage.
///
/// All data lives in a `BTreeMap<String, Vec<u8>>`. Keys follow
/// HLLSet naming conventions (`h:`, `c:`).
#[derive(Clone, Debug, Default)]
pub struct MemoryStorage {
    data: std::rc::Rc<std::cell::RefCell<BTreeMap<String, Vec<u8>>>>,
    pinned: std::rc::Rc<std::cell::RefCell<HashSet<String>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for MemoryStorage {
    fn store(&self, key: &str, data: &[u8]) -> Result<()> {
        self.data
            .borrow_mut()
            .insert(key.to_string(), data.to_vec());
        Ok(())
    }

    fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.data.borrow().get(key).cloned())
    }

    fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.data.borrow().contains_key(key))
    }

    fn delete(&self, key: &str) -> Result<bool> {
        Ok(self.data.borrow_mut().remove(key).is_some())
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let keys: Vec<String> = self
            .data
            .borrow()
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }

    fn pin(&self, key: &str) -> Result<()> {
        self.pinned.borrow_mut().insert(key.to_string());
        Ok(())
    }

    fn unpin(&self, key: &str) -> Result<()> {
        self.pinned.borrow_mut().remove(key);
        Ok(())
    }

    fn gc(&self) -> Result<Vec<String>> {
        let pinned = self.pinned.borrow();
        let mut data = self.data.borrow_mut();
        let to_remove: Vec<String> = data
            .keys()
            .filter(|k| !pinned.contains(*k))
            .cloned()
            .collect();
        for k in &to_remove {
            data.remove(k);
        }
        Ok(to_remove)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_load() {
        let s = MemoryStorage::new();
        s.store("h:abc", b"hello").unwrap();
        assert!(s.exists("h:abc").unwrap());
        assert_eq!(s.load("h:abc").unwrap(), Some(b"hello".to_vec()));
    }

    #[test]
    fn test_missing_returns_none() {
        let s = MemoryStorage::new();
        assert_eq!(s.load("h:nonexistent").unwrap(), None);
        assert!(!s.exists("h:nonexistent").unwrap());
    }

    #[test]
    fn test_delete() {
        let s = MemoryStorage::new();
        s.store("h:x", b"test").unwrap();
        assert!(s.delete("h:x").unwrap());
        assert!(!s.exists("h:x").unwrap());
    }

    #[test]
    fn test_list_by_prefix() {
        let s = MemoryStorage::new();
        s.store("h:a", b"1").unwrap();
        s.store("h:b", b"2").unwrap();
        s.store("c:x", b"3").unwrap();

        let h_keys = s.list("h:").unwrap();
        assert_eq!(h_keys.len(), 2);
        let c_keys = s.list("c:").unwrap();
        assert_eq!(c_keys.len(), 1);
    }
}
