//! Redis storage backend for HLLSet.
//!
//! Implements the `Storage` trait from `hllset-storage` using `redis-rs`.
//! Each trait method maps to a Redis command:
//!
//! | Method   | Redis command              |
//! |----------|----------------------------|
//! | `store`  | `SET key data` (binary)    |
//! | `load`   | `GET key`                  |
//! | `exists` | `EXISTS key`               |
//! | `delete` | `DEL key`                  |
//! | `list`   | `SCAN 0 MATCH prefix*`     |
//! | `pin`    | `SADD hllset:pins key`     |
//! | `unpin`  | `SREM hllset:pins key`     |
//! | `gc`     | SCAN + filter pinned + DEL |
//!
//! # Example
//!
//! ```rust,no_run
//! use hllset_storage_redis::RedisStorage;
//! use hllset_storage::Storage;
//!
//! let store = RedisStorage::connect("redis://127.0.0.1:6379").unwrap();
//! store.store("h:test", b"hello redis").unwrap();
//! assert!(store.exists("h:test").unwrap());
//! let data = store.load("h:test").unwrap();
//! assert_eq!(data, Some(b"hello redis".to_vec()));
//! ```

use hllset_storage::storage::{Result, Storage, StorageError};
use redis::Commands;
use std::cell::RefCell;
use std::collections::HashSet;

/// Redis-backed storage implementing the HLLSet `Storage` trait.
///
/// Uses `RefCell` for interior mutability because `redis::Connection`
/// requires `&mut self` but the `Storage` trait methods take `&self`.
pub struct RedisStorage {
    conn: RefCell<redis::Connection>,
    url: String,
}

impl RedisStorage {
    /// Connect to a Redis server at the given URL.
    pub fn connect(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)
            .map_err(|e| StorageError::Backend(format!("redis open: {e}")))?;
        let mut conn = client
            .get_connection()
            .map_err(|e| StorageError::Backend(format!("redis connect: {e}")))?;

        // Verify connection with PING
        redis::cmd("PING")
            .query::<String>(&mut conn)
            .map_err(|e| StorageError::Backend(format!("redis ping: {e}")))?;

        Ok(Self {
            conn: RefCell::new(conn),
            url: url.to_string(),
        })
    }

    /// Return the Redis connection URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Check if the Redis server is reachable.
    pub fn ping(&self) -> Result<String> {
        redis::cmd("PING")
            .query::<String>(&mut *self.conn.borrow_mut())
            .map_err(|e| StorageError::Backend(format!("redis ping: {e}")))
    }
}

impl Storage for RedisStorage {
    fn store(&self, key: &str, data: &[u8]) -> Result<()> {
        self.conn
            .borrow_mut()
            .set(key, data)
            .map_err(|e| StorageError::Backend(format!("redis SET: {e}")))
    }

    fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.conn
            .borrow_mut()
            .get(key)
            .map_err(|e| StorageError::Backend(format!("redis GET: {e}")))
    }

    fn exists(&self, key: &str) -> Result<bool> {
        let exists: bool = self
            .conn
            .borrow_mut()
            .exists(key)
            .map_err(|e| StorageError::Backend(format!("redis EXISTS: {e}")))?;
        Ok(exists)
    }

    fn delete(&self, key: &str) -> Result<bool> {
        let count: i32 = self
            .conn
            .borrow_mut()
            .del(key)
            .map_err(|e| StorageError::Backend(format!("redis DEL: {e}")))?;
        Ok(count > 0)
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let pattern = format!("{prefix}*");
        let mut cursor: u64 = 0;
        let mut keys = Vec::new();
        let mut conn = self.conn.borrow_mut();

        loop {
            let (next_cursor, batch): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query(&mut *conn)
                .map_err(|e| StorageError::Backend(format!("redis SCAN: {e}")))?;

            keys.extend(batch);
            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(keys)
    }

    fn pin(&self, key: &str) -> Result<()> {
        self.conn
            .borrow_mut()
            .sadd("hllset:pins", key)
            .map_err(|e| StorageError::Backend(format!("redis SADD pins: {e}")))
    }

    fn unpin(&self, key: &str) -> Result<()> {
        self.conn
            .borrow_mut()
            .srem("hllset:pins", key)
            .map_err(|e| StorageError::Backend(format!("redis SREM pins: {e}")))
    }

    fn gc(&self) -> Result<Vec<String>> {
        // Phase 1: get pinned keys in scoped borrow
        let pinned: HashSet<String> = {
            let mut conn = self.conn.borrow_mut();
            conn.smembers("hllset:pins")
                .map_err(|e| StorageError::Backend(format!("redis SMEMBERS pins: {e}")))?
        };

        // Phase 2: scan all keys (separate borrows via list())
        let mut all_keys = self.list("h:")?;
        let c_keys = self.list("c:")?;
        all_keys.extend(c_keys);

        // Phase 3: delete unpinned keys in new borrow
        let mut conn = self.conn.borrow_mut();
        let mut removed = Vec::new();
        for key in &all_keys {
            if !pinned.contains(key) {
                let count: i32 = conn
                    .del(key.as_str())
                    .map_err(|e| StorageError::Backend(format!("redis DEL gc: {e}")))?;
                if count > 0 {
                    removed.push(key.clone());
                }
            }
        }

        // Phase 4: clean up pins set
        if !removed.is_empty() {
            let _: () = conn
                .srem("hllset:pins", removed.clone())
                .map_err(|e| StorageError::Backend(format!("redis SREM cleanup: {e}")))?;
        }

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connect() -> Option<RedisStorage> {
        RedisStorage::connect("redis://127.0.0.1:6379").ok()
    }

    #[test]
    fn test_store_and_load() {
        let store = match connect() {
            Some(s) => s,
            None => {
                eprintln!("Redis not running — skipping test");
                return;
            }
        };
        let key = "h:test_redis_store";
        let data = b"hello redis backend";

        store.store(key, data).unwrap();
        assert!(store.exists(key).unwrap());

        let loaded = store.load(key).unwrap().unwrap();
        assert_eq!(loaded, data);

        store.delete(key).unwrap();
    }

    #[test]
    fn test_list_by_prefix() {
        let store = match connect() {
            Some(s) => s,
            None => return,
        };
        store.store("h:list_alpha", b"a").unwrap();
        store.store("h:list_beta", b"b").unwrap();
        store.store("c:list_gamma", b"c").unwrap();

        let h_keys = store.list("h:list_").unwrap();
        assert!(h_keys.contains(&"h:list_alpha".to_string()));
        assert!(h_keys.contains(&"h:list_beta".to_string()));

        let c_keys = store.list("c:list_").unwrap();
        assert!(c_keys.contains(&"c:list_gamma".to_string()));

        store.delete("h:list_alpha").unwrap();
        store.delete("h:list_beta").unwrap();
        store.delete("c:list_gamma").unwrap();
    }

    #[test]
    fn test_delete() {
        let store = match connect() {
            Some(s) => s,
            None => return,
        };
        store.store("h:test_redis_del", b"x").unwrap();
        assert!(store.exists("h:test_redis_del").unwrap());
        assert!(store.delete("h:test_redis_del").unwrap());
        assert!(!store.exists("h:test_redis_del").unwrap());
        assert!(!store.delete("h:test_redis_del").unwrap());
    }

    #[test]
    fn test_pin_and_gc() {
        let store = match connect() {
            Some(s) => s,
            None => return,
        };
        store.store("h:gc_keep", b"keep").unwrap();
        store.store("h:gc_toss", b"toss").unwrap();
        store.pin("h:gc_keep").unwrap();

        let removed = store.gc().unwrap();
        assert!(removed.contains(&"h:gc_toss".to_string()));
        assert!(!removed.contains(&"h:gc_keep".to_string()));
        assert!(store.exists("h:gc_keep").unwrap());
        assert!(!store.exists("h:gc_toss").unwrap());

        store.unpin("h:gc_keep").unwrap();
        store.delete("h:gc_keep").unwrap();
    }

    #[test]
    fn test_ping() {
        let store = match connect() {
            Some(s) => s,
            None => return,
        };
        let pong = store.ping().unwrap();
        assert!(pong.contains("PONG") || pong == "PONG");
    }
}
