# Guru HLLSet Lattice — Self-Programming Architecture & Design Document

> **Application:** Guru HLLSet Lattice (`guru-hllset-lattice`)  
> **Core Library:** [hllset-next](https://github.com/alexmy21/hllset-next)  
> **Paradigm:** Self-Programming System — the lattice writes, rewrites, and evolves its own logic  
> **Storage Backends:** Redis (temporal/key-value) or IPFS (content-addressed)  
> **Date:** July 14, 2026  
> **Status:** Design Proposal

---

## Table of Contents

1. [Overview](#1-overview)
2. [Self-Programming Paradigm](#2-self-programming-paradigm)
3. [Architecture](#3-architecture)
4. [Core Concepts — HLLSet & Lattice](#4-core-concepts--hllset--lattice)
5. [Getting hllset-next from GitHub](#5-getting-hllset-next-from-github)
6. [Generating HLLSets](#6-generating-hllsets)
7. [Lattice Storage Design](#7-lattice-storage-design)
8. [Backend 1: Redis Storage](#8-backend-1-redis-storage)
9. [Backend 2: IPFS Storage (HLPP)](#9-backend-2-ipfs-storage-hlpp)
10. [User Choice — Backend Selection](#10-user-choice--backend-selection)
11. [API Design](#11-api-design)
12. [CLI Usage](#12-cli-usage)
13. [Deployment](#13-deployment)
14. [Appendix: Project Structure](#14-appendix-project-structure)

---

## 1. Overview

This document describes the design of a **self-programming application** that:

1. **Pulls `hllset-next`** from [github.com/alexmy21/hllset-next](https://github.com/alexmy21/hllset-next) as its core computational engine.
2. **Generates HLLSets** — HyperLogLog probabilistic fingerprints that compactly represent sets of tokens (text, data, byte sequences).
3. **Stores HLLSets in lattice form** — the lattice is a bounded distributive algebraic structure where HLLSets are elements with join (union, ∪) and meet (intersection, ∩) operations.
4. **Persists the lattice** in either **Redis** (low-latency key-value, ideal for temporal/transient state) or **IPFS** (content-addressed, immutable, ideal for permanent canonical storage), based on user choice.
5. **Self-programs** — the lattice becomes a living codebase where patterns detected through rank algebra, Noether steering, and BSS morphisms feed back into the system, generating new logic, rewiring existing structures, and evolving its own behaviour without external intervention.

HLLSets form a **bounded distributive lattice**:

| Property | Definition | HLLSet Realization |
|---|---|---|
| **Join (∪)** | Least upper bound | Bitwise OR of registers — union |
| **Meet (∩)** | Greatest lower bound | Bitwise AND of registers — intersection |
| **Associative** | (a ∪ b) ∪ c = a ∪ (b ∪ c) | OR is associative |
| **Commutative** | a ∪ b = b ∪ a | OR is commutative |
| **Idempotent** | a ∪ a = a | OR is idempotent |
| **Bottom (⊥)** | Empty element | HLLSet with no bits set |
| **Top (⊤)** | Universal element | HLLSet with all bits set |

---

## 3. Architecture

```
                          ┌──────────────────────────────────────┐
                          │      Self-Programming Loop           │
                          │                                      │
                          │  ┌─────────┐    ┌──────────────┐    │
                          │  │ Noether │    │   Fisher     │    │
                          │  │ Steering│───►│   Matrix     │    │
                          │  │(drift   │    │(co-occurrence)│    │
                          │  │ detect) │    └──────┬───────┘    │
                          │  └────┬────┘           │            │
                          │       │                │            │
                          │       ▼                ▼            │
                          │  ┌────────────────────────────┐     │
                          │  │   Rank Algebra Engine      │     │
                          │  │   F→G→H→K→L→M hierarchy   │     │
                          │  └────────────┬───────────────┘     │
                          │               │                    │
                          │               ▼                    │
                          │  ┌────────────────────────────┐     │
                          │  │   Program Synthesizer      │     │
                          │  │   (Lua code generation     │     │
                          │  │    from lattice patterns)  │     │
                          │  └────────────┬───────────────┘     │
                          │               │                    │
                          └───────────────┼────────────────────┘
                                          │
            ┌─────────────────────────────┼─────────────────────┐
            │         Application Layer   │                     │
            │  ┌─────────┐  ┌──────────┐  ▼  ┌──────────────┐  │
            │  │ Ingest  │  │ Lattice  │  ┌──────────┐       │  │
            │  │ Pipeline│  │ Algebra  │  │  Query   │       │  │
            │  └────┬────┘  └────┬─────┘  │  Engine  │       │  │
            └───────┼────────────┼────────└────┬─────┘       ┘  │
                    │            │               │               │
            ┌───────▼────────────▼───────────────▼───────────────┐
            │              Core Layer (hllset-next)              │
            │  ┌──────────────┐  ┌───────────────┐  ┌─────────┐ │
            │  │ hllset-core  │  │  hllset-dsl   │  │hllset-  │ │
            │  │ (algebra)    │  │  (Lua VM)     │  │material.│ │
            │  └──────────────┘  └───────────────┘  └─────────┘ │
            └───────┼────────────────────┼──────────────────────┘
                    │                    │
            ┌───────▼────────────────────▼──────────────────────┐
            │              Storage Abstraction Layer            │
            │  ┌───────────────────────────────────────────┐    │
            │  │     Storage Trait + HLPP Protocol         │    │
            │  └──────────┬───────────────────────────┬────┘    │
            │             │                           │         │
            │  ┌──────────▼──────────┐  ┌────────────▼────────┐ │
            │  │  RedisStorage       │  │  IpfsStorage (HLPP) │ │
            │  │  (temporal KV)      │  │  (content-addressed)│ │
            │  └─────────────────────┘  └─────────────────────┘ │
            └───────────────────────────────────────────────────┘
```

### Key Components

| Component | Crate / Module | Responsibility |
|---|---|---|
| **Ingest Pipeline** | `hllset-dsl` / `app::ingest` | Tokenize input data → create HLLSet → store in lattice |
| **Lattice Algebra** | `hllset-core` | ∪ ∩ \ ⊕ lattice operations, BSS morphisms |
| **Query Engine** | `app::query` | Fetch HLLSets by key, compute lattice operations, cardinality |
| **Materialize Engine** | `hllset-materialize` | Reverse-map HLLSet bits back to known tokens via LUT |
| **Noether Steering** | `hllset-mesh` / `app::noether` | Pattern drift detection — monitors rank flux and divergence over time |
| **Fisher Matrix** | `hllset-core` / `app::fisher` | Temporal co-occurrence analysis — builds co-occurrence matrices from token frequency vectors |
| **Rank Algebra Engine** | `hllset-core` / `app::ranks` | Five-level rank hierarchy: token-R → bit-R → reg-R → hllset-R → compound-R |
| **Program Synthesizer** | `app::synth` | Generates Lua scripts and lattice structures from detected patterns — the system writes its own logic |
| **Self-Programming Loop** | `app::feedback` | Orchestrates the detect → analyse → synthesise → apply cycle |
| **Storage Trait** | `hllset-storage` | Abstract `Storage` trait with multiple backends |
| **Redis Backend** | `app::storage::redis` | Redis key-value store for temporal/cached lattice data |
| **IPFS Backend** | `hllset-storage::ipfs` | `ipfrs-core` + `sled` for content-addressed storage |

---

## 2. Self-Programming Paradigm

### 2.1 What Self-Programming Means

A **self-programming system** is one that can modify its own behaviour based on patterns it detects in its own operation. Unlike traditional programs that follow fixed logic, a self-programming system:

- **Observes itself** through lattice operations (union, intersection, BSS morphisms)
- **Detects drift** via Noether steering — monitoring rank flux and divergence over time
- **Identifies co-occurrence patterns** through the Fisher matrix — building temporal correlation structures from token frequency vectors
- **Generates new logic** by synthesising Lua scripts from lattice patterns — the system writes code that becomes part of the running system
- **Evolves its lattice** automatically — adding, merging, and pruning lattice elements to optimise for observed behaviour

### 2.2 The Self-Programming Loop

```
                      ┌─────────────────────┐
                      │   Ingest Data       │
                      │   (tokens → HLLSet) │
                      └──────────┬──────────┘
                                 │
                                 ▼
                      ┌─────────────────────┐
                      │   Observe Lattice   │
                      │   (BSS, subset,     │
                      │    rank analysis)   │
                      └──────────┬──────────┘
                                 │
                    ┌────────────┼────────────┐
                    ▼            ▼            ▼
             ┌──────────┐ ┌──────────┐ ┌──────────┐
             │ Noether  │ │  Fisher  │ │  Rank    │
             │ Steering │ │  Matrix  │ │ Algebra  │
             │(drift)   │ │(co-occur)│ │(F→G→H→K)│
             └────┬─────┘ └────┬─────┘ └────┬─────┘
                  │            │            │
                  └────────────┼────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │   Synthesise Logic  │
                    │   (generate Lua     │
                    │    scripts, new     │
                    │    lattice ops)     │
                    └──────────┬──────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │   Apply & Store     │
                    │   (persist new      │
                    │    elements,        │
                    │    rewire lattice)  │
                    └──────────┬──────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │   Repeat (loop)     │◄──── self-programming
                    └─────────────────────┘
```

### 2.3 How the Lattice Enables Self-Programming

The lattice is not merely a storage structure — it is the **program itself**. Because:

| Property | Self-Programming Role |
|---|---|
| **Immutable elements** | Past decisions are preserved as a permanent record. The system can inspect its own history. |
| **Idempotent operations** | Safe to reapply — the system can experiment without corrupting state. |
| **Content-addressed keys** | Every lattice element is addressable by its content. The system can reference its own parts by what they mean, not where they are stored. |
| **BSS morphisms** | The system can measure how confidently one pattern contains another — enabling analogical reasoning and code reuse. |
| **Rank hierarchy** | Token-R → bit-R → reg-R → hllset-R → compound-R. The system quantifies what matters and what fades, steering its own attention. |
| **Noether steering** | Monitors divergence between observed and expected rank distributions — the system detects when its model of itself is drifting. |
| **Fisher matrix** | Captures temporal co-occurrence of sub-patterns — the system learns which patterns tend to appear together and synthesises new composite operations. |

### 2.4 Self-Programming Outputs

The system generates these **self-programming artefacts** as Rust library files:

| Artefact | Format | Description |
|---|---|---|
| **Lua scripts** | `.lua` files | Generated logic that runs inside the `hllset-dsl` VM — new tokenizers, materialisers, and lattice traversal strategies |
| **Lattice configuration** | Rust `LatticeConfig` | Automatically derived lattice topology — which operations to chain, what thresholds to use |
| **Rank profiles** | Rust `RankProfile` | Learned rank hierarchies that optimise pattern detection for observed data distributions |
| **Noether schedules** | Rust `NoetherSchedule` | Drift detection schedules — when and how aggressively to re-evaluate the system's self-model |
| **Fisher eigenpatterns** | Rust `FisherPattern` | Dominant co-occurrence modes extracted from the Fisher matrix — the system's learned "grammar" of patterns |

### 2.5 Example: Self-Programmed Pattern Detection

```text
1. System ingests code tokens → stores as HLLSet "h:abc..."
2. Noether steering detects rising rank flux in register class 400-500
3. Fisher matrix reveals co-occurrence between tokens "union" and "intersect"
4. Rank algebra elevates these tokens to high token-R
5. Program synthesizer generates a new Lua script:
     function auto_union_intersect(a, b)
       local u = a + b
       local i = a * b
       if #i / #u > 0.7 then
         return u   -- high overlap → prefer union
       else
         return i   -- low overlap → prefer intersection
       end
     end
6. Script is stored as a new lattice element "c:def..."
7. System applies the script to future ingestions
8. Loop repeats — behaviour evolves continuously
```

---

## 4. Core Concepts — HLLSet & Lattice

### 4.1 HLLSet

An **HLLSet** is a HyperLogLog probabilistic fingerprint stored as a compressed Roaring bitmap. It compactly represents a set of tokens with:

- **Configurable precision**: 1024 registers (P=10), each 32 bits wide
- **Set operations**: union (OR), intersection (AND), difference, symmetric difference (XOR)
- **Cardinality estimation**: Horvitz-Thompson estimator
- **Content-addressable key**: Deterministic SHA-1 hash of the serialized HLLSet
- **Idempotent insertion**: The same token always sets the same bits

```rust
// Core HLLSet structure (from hllset-core)
pub struct HLLSet {
    bitmap: RoaringBitmap, // Compressed bitmap: (reg * 32 + tz) positions
}
```

### 4.2 LatticeElement

The **LatticeElement** wraps an HLLSet with its content-addressable key:

```rust
pub struct LatticeElement {
    hllset: HLLSet,
    key: String,  // "h:<sha1>" or "c:<sha1>"
}
```

Every operation on a `LatticeElement` produces a **new** element (immutable lattice semantics):

| Operation | Symbol | Method | Description |
|---|---|---|---|
| **Join** | ∪ | `a.union(&b)` | Bitwise OR — elements in either set |
| **Meet** | ∩ | `a.intersection(&b)` | Bitwise AND — elements in both sets |
| **Difference** | \ | `a.difference(&b)` | A AND NOT B |
| **Symmetric diff** | ⊕ | `a.xor(&b)` | XOR |
| **BSS inclusion** | ⊑ | `a.bss_inclusion(&b)` | Bell State Similarity (confidence) |

### 4.3 The Lattice Structure

The lattice is defined by the **partially ordered set** of all possible HLLSet bit vectors under the **subset relation** (⊆):

```
         ┌──⊤──┐           ← All bits set (universal set)
        /   |   \
      H1   H2   H3          ← Individual HLLSets (lattice elements)
       \   / \   /
        H4   H5             ← Union/intersection results
         \   /
          └─⊥─┘             ← Empty HLLSet (bottom)
```

When HLLSets are persisted in a store (Redis or IPFS), the **stored collection** is a snapshot of the lattice — each key represents a lattice element, and the keyspace organizes the partial order through content-addressing.

---

## 5. Getting hllset-next from GitHub

### 4.1 Clone the Repository

```bash
git clone https://github.com/alexmy21/hllset-next.git
cd hllset-next
```

### 4.2 Project Structure

```
hllset-next/
├── Cargo.toml                     # Workspace manifest
├── crates/
│   ├── hllset-core/               # Core HLLSet algebra engine
│   │   └── src/core/
│   │       ├── hllset.rs          # HLLSet struct (Roaring bitmap)
│   │       ├── operations.rs      # ∪ ∩ \ ⊕ lattice operations
│   │       ├── cardinality.rs     # Horvitz-Thompson estimator
│   │       ├── hashing.rs         # MurmurHash3 token→hash
│   │       ├── bss.rs             # Bell State Similarity morphisms
│   │       ├── serialization.rs   # to_bytes / from_bytes / content_key
│   │       └── content_addr.rs    # SHA-1 content addressing
│   ├── hllset-dsl/                # Lua DSL runtime + LatticeElement
│   │   └── src/
│   │       ├── lattice.rs         # LatticeElement wrapper
│   │       ├── runtime.rs         # Lua VM with hllset bindings
│   │       ├── worker.rs          # Stateless Worker (inscribe/load/materialize)
│   │       ├── materialize.rs     # LUT-based token recovery
│   │       └── tokenizer.rs       # Text tokenization
│   ├── hllset-storage/            # Storage trait + backends
│   │   └── src/
│   │       ├── storage.rs         # Storage trait definition
│   │       ├── memory.rs          # MemoryStorage (HashMap)
│   │       ├── ipfs.rs            # IpfrsNativeStorage (sled + ipfrs-core)
│   │       └── cache.rs           # CacheStorage (LRU wrapper)
│   ├── hllset-forth/              # Forth DSL parser/compiler
│   ├── hllset-materialize/        # Pluggable materialization engines
│   ├── hllset-duckdb/             # DuckDB-backed chunked LUT
│   ├── hllset-mesh/               # Rust-native pub/sub mesh bus
│   └── hllset-cli/                # CLI entry point
├── _DOCS/
│   ├── dev/HLPP.md                # HLLSet Lattice Persistence Protocol
│   └── notebooks/                 # Jupyter notebooks (Rust + Python)
└── scripts/                       # Python helper scripts
```

### 4.3 Build

```bash
cargo build --release
# Binary at: target/release/hllset
```

### 4.4 Key Dependencies

| Dependency | Purpose |
|---|---|
| `roaring` | Compressed bitmap storage for HLLSet registers |
| `serde` / `bincode` | Serialization/deserialization |
| `sha1` | Content-addressed key derivation |
| `mlua` | Lua scripting runtime |
| `ipfrs-core` | IPFS content-addressing (CID computation) |
| `sled` | Embedded database for IPFS-native storage |
| `redis` | Redis client (proposed for this application) |

---

## 6. Generating HLLSets

### 5.1 From Tokens (Direct Rust API)

```rust
use hllset_core::HLLSet;

// Create from byte tokens
let h = HLLSet::from_tokens(&["hello", "world", "test"]);

// Add tokens incrementally
let mut h = HLLSet::new();
h.add_token(b"hello");
h.add_token(b"world");

// Cardinality estimation
let card = h.cardinality();  // ≈ 2.0

// Content-addressable key
let key = h.content_key();   // "h:a3f82c..."
```

### 5.2 From Hashes

```rust
// From pre-computed 64-bit hashes
let h = HLLSet::from_hashes(&[0xDEADBEEF, 0xCAFEBABE]);
```

### 5.3 Via Lua DSL

```lua
-- via hllset CLI
local h = hllset.inscribe({"hello", "world", "test"})
return h:key()  -- content-addressed key
```

### 5.4 LatticeElement (with key tracking)

```rust
use hllset_dsl::LatticeElement;

let elem = LatticeElement::from_tokens(&["hello", "world"]);
println!("Key: {}", elem.key());         // "h:<sha1>"
println!("Cardinality: {}", elem.cardinality());  // ≈ 2.0

// Lattice operations
let a = LatticeElement::from_tokens(&["apple", "banana"]);
let b = LatticeElement::from_tokens(&["banana", "cherry"]);
let union = a.union(&b);        // ∪
let inter = a.intersection(&b); // ∩
let diff = a.difference(&b);    // \
```

### 5.5 Via the Worker (Store-aware)

```rust
use hllset_dsl::Worker;
use hllset_storage::MemoryStorage;

let worker = Worker::new(MemoryStorage::new());
let key = worker.inscribe(&["hello", "world"]);
// HLLSet is automatically stored under content key

let loaded = worker.load_hllset(&key);
// loaded: Option<HLLSet>
```

---

## 7. Lattice Storage Design

The **HLLSet Lattice Persistence Protocol (HLPP)** defines how lattice elements are stored and retrieved. The key design principles are:

### 6.1 IICA — Immutable, Idempotent, Content-Addressed

| Principle | Meaning | Benefit |
|---|---|---|
| **Immutable** | Once stored, an HLLSet never changes | Safe caching, no locking |
| **Idempotent** | Same tokens → same bytes → same key | No duplicates, safe retry |
| **Content-Addressed** | Key = hash of content | Self-validating, no conflicts |

### 6.2 Namespace Convention

| Prefix | Type | Purpose |
|---|---|---|
| `h:` | HLLSet | Standard HLLSet (any operation result) |
| `o:` | HLLSet | Original HLLSet from tokenizer |
| `r:` | HLLSet | Retained intersection (R-link) |
| `d:` | HLLSet | Departed elements (difference) |
| `n:` | HLLSet | New elements (difference) |
| `t:` | Commit | Commit object (temporal chain) |
| `u:` | User-assigned | UUID-based temporal key |
| `system:` | Named | Global state (TF vector, head, layers) |

### 6.3 The `Storage` Trait

```rust
pub trait Storage {
    /// Store raw bytes under a key.
    fn store(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Load raw bytes by key. Returns None if not found.
    fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Check whether a key exists.
    fn exists(&self, key: &str) -> Result<bool>;

    /// Delete a key and its data.
    fn delete(&self, key: &str) -> Result<bool>;

    /// List keys matching a prefix (e.g., "h:" or "c:").
    fn list(&self, prefix: &str) -> Result<Vec<String>>;

    /// Pin a key — prevent garbage collection. Idempotent.
    fn pin(&self, key: &str) -> Result<()>;

    /// Unpin a key — allow garbage collection. Idempotent.
    fn unpin(&self, key: &str) -> Result<()>;

    /// Garbage collect: remove all unpinned keys.
    fn gc(&self) -> Result<Vec<String>>;
}
```

### 6.4 Storage Flow

```
                   ┌───────────────────┐
Input Data         │   Tokenize        │
  "hello world"    │   (split/embed)   │
                   └────────┬──────────┘
                            │ tokens: ["hello", "world"]
                            ▼
                   ┌───────────────────┐
                   │  Hash & Inscribe  │
                   │  MurmurHash3 →    │
                   │  set bits in      │
                   │  RoaringBitmap    │
                   └────────┬──────────┘
                            │ HLLSet (bitmap)
                            ▼
                   ┌───────────────────┐
                   │  Serialize        │
                   │  (bincode)        │
                   └────────┬──────────┘
                            │ bytes
                            ▼
                   ┌───────────────────┐
                   │  Compute Key      │
                   │  SHA1(bytes) →    │
                   │  "h:<hex>"        │
                   └────────┬──────────┘
                            │ key + bytes
                            ▼
              ┌─────────────────────────┐
              │   Storage Backend       │
              │   (User Choice)         │
              │                         │
              │  ┌───┐     ┌─────┐     │
              │  │Redis│   │ IPFS │     │
              │  └───┘     └─────┘     │
              └─────────────────────────┘
```

---

## 8. Backend 1: Redis Storage

### 7.1 When to Use Redis

- Low-latency read/write required (< 1ms)
- Temporal/transient lattice state
- Session-scoped HLLSets
- High-throughput ingestion pipeline
- Cache layer in front of IPFS

### 7.2 Key Design

Redis stores HLLSets as key-value pairs with **two key types**:

| Key Pattern | Type | Value | TTL |
|---|---|---|---|
| `hllset:{key}` | String | Binary serialized HLLSet (bincode) | None (persistent) or configurable |
| `hllset:idx:{prefix}` | Set | Set of all keys with given prefix | None |
| `hllset:meta:{key}` | Hash | Metadata (cardinality, popcount, timestamp) | Same as data |

### 7.3 RedisStorage Implementation

```rust
use redis::{Client, Connection, cmd};
use hllset_storage::{Storage, Result, StorageError};

pub struct RedisStorage {
    client: Client,
}

impl RedisStorage {
    pub fn new(url: &str) -> Result<Self> {
        let client = Client::open(url)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(Self { client })
    }
}

impl Storage for RedisStorage {
    fn store(&self, key: &str, data: &[u8]) -> Result<()> {
        let mut conn = self.client.get_connection()
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        let redis_key = format!("hllset:{}", key);
        let prefix = &key[..2]; // e.g., "h:"

        redis::pipe()
            .cmd("SET")
            .arg(&redis_key)
            .arg(data)
            .ignore()
            .cmd("SADD")
            .arg(format!("hllset:idx:{}", prefix))
            .arg(&redis_key)
            .ignore()
            .query(&mut conn)
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let mut conn = self.client.get_connection()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let redis_key = format!("hllset:{}", key);
        cmd("GET")
            .arg(&redis_key)
            .query(&mut conn)
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.client.get_connection()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let redis_key = format!("hllset:{}", key);
        cmd("EXISTS")
            .arg(&redis_key)
            .query(&mut conn)
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    fn delete(&self, key: &str) -> Result<bool> {
        let mut conn = self.client.get_connection()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let redis_key = format!("hllset:{}", key);
        let prefix = &key[..2];
        redis::pipe()
            .cmd("DEL")
            .arg(&redis_key)
            .ignore()
            .cmd("SREM")
            .arg(format!("hllset:idx:{}", prefix))
            .arg(&redis_key)
            .ignore()
            .query(&mut conn)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(true)
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let mut conn = self.client.get_connection()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        let members: Vec<String> = cmd("SMEMBERS")
            .arg(format!("hllset:idx:{}", prefix))
            .query(&mut conn)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        // Strip the "hllset:" prefix from each key
        Ok(members.into_iter().map(|k| {
            k.strip_prefix("hllset:").unwrap_or(&k).to_string()
        }).collect())
    }

    fn pin(&self, key: &str) -> Result<()> {
        let mut conn = self.client.get_connection()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        cmd("SADD")
            .arg("hllset:pinned")
            .arg(format!("hllset:{}", key))
            .query(&mut conn)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(())
    }

    fn unpin(&self, key: &str) -> Result<()> {
        let mut conn = self.client.get_connection()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        cmd("SREM")
            .arg("hllset:pinned")
            .arg(format!("hllset:{}", key))
            .query(&mut conn)
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(())
    }

    fn gc(&self) -> Result<Vec<String>> {
        // Not implemented for Redis — use TTL-based eviction instead.
        Ok(Vec::new())
    }
}
```

### 7.4 Redis Configuration

```yaml
# config/redis.yaml
redis:
  url: "redis://127.0.0.1:6379/0"
  ttl_seconds: 0            # 0 = no expiry, or set e.g. 86400 (24h)
  key_prefix: "hllset:"
  pool_size: 10
```

### 7.5 Redis Lattice Commands

```bash
# Store an HLLSet
redis-cli SET hllset:h:a3f82c... <binary_data>

# List all HLLSets with prefix
redis-cli SMEMBERS hllset:idx:h:

# List all HLLSets (full lattice)
redis-cli KEYS hllset:h:*

# Check existence
redis-cli EXISTS hllset:h:a3f82c...

# Delete
redis-cli DEL hllset:h:a3f82c...

# Pin management
redis-cli SADD hllset:pinned hllset:h:a3f82c...
redis-cli SREM hllset:pinned hllset:h:a3f82c...
```

### 7.6 Redis as Temporal Cache (Redis + IPFS)

For production deployments, Redis can serve as a **hot cache** in front of IPFS:

```
            ┌──────────┐
Request ──► │  Redis   │── Cache hit? ──► Return data
            │  (LRU)   │
            └────┬─────┘
                 │ Cache miss
                 ▼
            ┌──────────┐
            │   IPFS   │── Fetch ──► Store in Redis ──► Return
            │(canonical│
            │ storage) │
            └──────────┘
```

---

## 9. Backend 2: IPFS Storage (HLPP)

### 8.1 When to Use IPFS

- Permanent, immutable lattice storage
- Content-addressable canonical records
- Distributed/replicated deployments
- Long-term archival with integrity verification
- Multi-node lattice synchronization

### 8.2 Implementation via ipfrs-native

The `hllset-next` project already provides `IpfrsNativeStorage` — a pure Rust IPFS-native backend using `ipfrs-core` for content-addressing and `sled` for local persistence. No external Go IPFS daemon is needed.

```rust
use hllset_storage::IpfrsNativeStorage;

// Open (or create) a persistent sled database
let storage = IpfrsNativeStorage::open("/data/hllset-lattice")?;

// Store with automatic CID computation
let data = b"hello world";
let cid = IpfrsNativeStorage::compute_cid(data)?;  // ipfrs-core CID
storage.store("h:abc123", data)?;

// Load by key
let result = storage.load("h:abc123")?;

// List by prefix
let hllsets = storage.list("h:")?;
```

### 8.3 Full HLPP Protocol Support

For the complete HLLSet Lattice Persistence Protocol, implement the full `HlppStorage` trait:

```rust
pub trait HlppStorage {
    // CA operations
    fn put(&self, cid: &str, bytes: &[u8]) -> Result<(), HlppError>;
    fn get(&self, cid: &str) -> Result<Option<Vec<u8>>, HlppError>;
    fn has(&self, cid: &str) -> Result<bool, HlppError>;
    fn list(&self, prefix: &str) -> Result<Vec<String>, HlppError>;
    fn pin(&self, cid: &str) -> Result<(), HlppError>;
    fn unpin(&self, cid: &str) -> Result<(), HlppError>;
    fn gc(&self) -> Result<Vec<String>, HlppError>;

    // Temporal operations
    fn get_tmp(&self, key: &str) -> Result<Option<Vec<u8>>, HlppError>;
    fn put_tmp(&self, key: &str, bytes: &[u8]) -> Result<(), HlppError>;
    fn cas_tmp(&self, key: &str, old: &[u8], new: &[u8]) -> Result<bool, HlppError>;
}
```

### 8.4 IPFS Wire Format

HLLSets are serialized in a fixed-size binary format:

```
Offset  Size    Field
0       8       Magic: "HLLSET\0\0"
8       4       Version: uint32 LE
12      4       M (register count): uint32 LE (= 1024)
16      4       B (bits per register): uint32 LE (= 32)
20      4096    Register array: 1024 × uint32 LE
4116    *       Optional metadata (reserved)
```

**CID**: `h:SHA1(bytes[0..4116])`

### 8.5 IPFS Configuration

```yaml
# config/ipfs.yaml
ipfs:
  storage_path: "/data/hllset-lattice"
  backend: "ipfrs-native"    # Pure Rust, no external daemon
  # For external Go IPFS daemon:
  # backend: "http"
  # api_url: "http://127.0.0.1:5001"
  # gateway_url: "http://127.0.0.1:8080"
  cache_enabled: true
  cache_capacity: 1000
  cache_ttl_seconds: 3600
```

### 8.6 IPFS Lattice Commands

```bash
# Store an HLLSet (via HLPP)
hllset -e 'local e = hllset.inscribe({"hello","world"}); hllset.store(e); return e:key()'

# Load an HLLSet
hllset -e 'return hllset.load("h:a3f82c..."):card()'

# List all HLLSets in the lattice
hllset -e 'return hllset.list("h:")'

# Check existence
hllset -e 'return hllset.exists("h:a3f82c...")'

# Lattice operations with persistence
hllset -e '
  local a = hllset.inscribe({"apple", "banana"})
  local b = hllset.inscribe({"banana", "cherry"})
  hllset.store(a)
  hllset.store(b)
  local u = a + b   -- union
  local i = a * b   -- intersection
  hllset.store(u)
  hllset.store(i)
  return {union=u:key(), inter=i:key(), card_u=#u, card_i=#i}
'
```

---

## 10. User Choice — Backend Selection

### 9.1 Selection Logic

The user chooses the storage backend at application startup or per-operation:

```rust
pub enum StorageBackend {
    Redis(RedisStorage),
    Ipfs(IpfrsNativeStorage),
    Hybrid {
        hot: RedisStorage,     // Read-through cache
        cold: IpfrsNativeStorage, // Canonical persistence
    },
}
```

### 9.2 Factory Pattern

```rust
pub fn create_storage(config: &StorageConfig) -> Box<dyn Storage> {
    match config.backend {
        BackendType::Redis => {
            Box::new(RedisStorage::new(&config.redis_url)
                .expect("Failed to connect to Redis"))
        }
        BackendType::Ipfs => {
            Box::new(IpfrsNativeStorage::open(&config.ipfs_path)
                .expect("Failed to open IPFS storage"))
        }
        BackendType::Hybrid => {
            let redis = RedisStorage::new(&config.redis_url)
                .expect("Failed to connect to Redis");
            let ipfs = IpfrsNativeStorage::open(&config.ipfs_path)
                .expect("Failed to open IPFS storage");
            Box::new(HybridStorage::new(redis, ipfs))
        }
    }
}
```

### 9.3 Decision Matrix

| Criterion | Choose Redis | Choose IPFS |
|---|---|---|
| **Latency** | ✅ < 1ms reads | ⬜ ~5-50ms (sled) |
| **Durability** | ⬜ Depends on config | ✅ Content-addressed, immutable |
| **Persistence** | ⬜ Volatile (configurable) | ✅ Permanent |
| **Distribution** | ⬜ Redis Cluster | ✅ IPFS multi-node |
| **Cache layer** | ✅ Excellent | ⬜ N/A |
| **Peer-to-peer** | ❌ No | ✅ Yes |
| **Throughput** | ✅ 100K+ ops/sec | ⬜ ~10K ops/sec |
| **Temporal state** | ✅ Great fit | ⬜ Overkill |
| **Use case** | Transient lattice, hot cache, real-time queries | Canonical lattice, archival, cross-node sync |

### 9.4 Hybrid Mode

Best practice: use **Redis as hot cache + IPFS as cold store**:

```
Write path:
  Input → HLLSet → store in Redis (immediate) → background sync to IPFS

Read path:
  Read from Redis → Cache hit? → Return
  Cache miss? → Read from IPFS → populate Redis → Return
```

---

## 11. API Design

### 10.1 Rust Library API

```rust
use hllset_core::HLLSet;
use hllset_dsl::LatticeElement;
use hllset_storage::Storage;

/// High-level lattice application API.
pub struct LatticeApp<S: Storage> {
    storage: S,
}

impl<S: Storage> LatticeApp<S> {
    /// Create a new lattice application with the given storage backend.
    pub fn new(storage: S) -> Self;

    /// Ingest tokens: tokenize → HLLSet → store → return key.
    pub fn ingest(&self, tokens: &[&str]) -> Result<String>;

    /// Load an HLLSet from the lattice by key.
    pub fn load(&self, key: &str) -> Result<Option<LatticeElement>>;

    /// Compute union of two stored HLLSets and optionally store result.
    pub fn union(&self, ka: &str, kb: &str, store_result: bool) -> Result<LatticeElement>;

    /// Compute intersection of two stored HLLSets.
    pub fn intersect(&self, ka: &str, kb: &str, store_result: bool) -> Result<LatticeElement>;

    /// Compute difference (A \ B) of two stored HLLSets.
    pub fn difference(&self, ka: &str, kb: &str, store_result: bool) -> Result<LatticeElement>;

    /// Get cardinality estimate for a stored HLLSet.
    pub fn cardinality(&self, key: &str) -> Result<f64>;

    /// List all keys in the lattice with a given prefix.
    pub fn list(&self, prefix: &str) -> Result<Vec<String>>;

    /// Materialize an HLLSet back to likely tokens using a LUT.
    pub fn materialize(&self, key: &str, lut: &TokenLUT) -> Result<MaterializedResult>;

    /// Compute BSS inclusion (confidence that B ⊆ A).
    pub fn bss_inclusion(&self, ka: &str, kb: &str) -> Result<f64>;

    /// Delete a key from the lattice.
    pub fn delete(&self, key: &str) -> Result<bool>;
}
```

### 10.2 REST API (Optional HTTP Layer)

| Method | Endpoint | Description |
|---|---|---|
| `POST` | `/api/v1/lattice/ingest` | Ingest tokens → return HLLSet key |
| `GET` | `/api/v1/lattice/{key}` | Load HLLSet by key |
| `GET` | `/api/v1/lattice/{key}/cardinality` | Get cardinality estimate |
| `POST` | `/api/v1/lattice/union` | Union of two HLLSets |
| `POST` | `/api/v1/lattice/intersect` | Intersection of two HLLSets |
| `POST` | `/api/v1/lattice/diff` | Difference of two HLLSets |
| `GET` | `/api/v1/lattice?prefix={prefix}` | List keys by prefix |
| `DELETE` | `/api/v1/lattice/{key}` | Delete a key |
| `POST` | `/api/v1/lattice/materialize` | Materialize HLLSet to tokens |

### 10.3 REST Request/Response Examples

**Ingest tokens:**
```bash
curl -X POST http://localhost:8080/api/v1/lattice/ingest \
  -H "Content-Type: application/json" \
  -d '{"tokens": ["hello", "world", "test"]}'
```

```json
{
  "key": "h:a3f82c91d4e8f7b6a5c4d3e2f1a0b9c8d7e6f5a4",
  "cardinality": 3.0,
  "popcount": 42,
  "backend": "redis"
}
```

**Union:**
```bash
curl -X POST http://localhost:8080/api/v1/lattice/union \
  -H "Content-Type: application/json" \
  -d '{"key_a": "h:abc...", "key_b": "h:def...", "store_result": true}'
```

```json
{
  "key": "h:789...",
  "cardinality": 5.0,
  "popcount": 78
}
```

---

## 12. CLI Usage

### 11.1 Starting the Application

```bash
# With Redis backend
cargo run -- --backend redis --redis-url redis://127.0.0.1:6379

# With IPFS backend
cargo run -- --backend ipfs --ipfs-path /data/hllset-lattice

# Hybrid mode (Redis cache + IPFS persistence)
cargo run -- --backend hybrid \
  --redis-url redis://127.0.0.1:6379 \
  --ipfs-path /data/hllset-lattice

# REPL mode
cargo run -- --repl --backend redis
```

### 11.2 Interactive Commands

```
hllset-lattice> ingest hello world test
→ Key: h:a3f82c... | Cardinality: 3.0

hllset-lattice> load h:a3f82c...
→ Key: h:a3f82c... | Cardinality: 3.0 | Popcount: 42

hllset-lattice> union h:a3f82c... h:b4e5d6...
→ Key: h:789abc... | Cardinality: 5.0

hllset-lattice> intersect h:a3f82c... h:b4e5d6...
→ Key: h:def012... | Cardinality: 2.0

hllset-lattice> cardinality h:a3f82c...
→ 3.0

hllset-lattice> list h:
→ h:a3f82c..., h:b4e5d6..., h:789abc..., h:def012...

hllset-lattice> backend
→ Current backend: redis (redis://127.0.0.1:6379)

hllset-lattice> switch ipfs --path /data/hllset-lattice
→ Switched to IPFS backend
```

### 11.3 One-shot Mode

```bash
# Ingest and store
cargo run -- --backend redis --eval '
  local e = hllset.inscribe({"hello", "world"})
  hllset.store(e)
  return e:key()
'

# Query lattice
cargo run -- --backend ipfs --eval '
  return hllset.list("h:")
'
```

---

## 13. Deployment

### 12.1 Docker Compose (Redis Backend)

```yaml
# docker-compose.redis.yml
version: "3.8"
services:
  lattice-app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - STORAGE_BACKEND=redis
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    command: redis-server --appendonly yes

volumes:
  redis-data:
```

### 12.2 Docker Compose (IPFS Backend)

```yaml
# docker-compose.ipfs.yml
version: "3.8"
services:
  lattice-app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - STORAGE_BACKEND=ipfs
      - IPFS_PATH=/data/hllset-lattice
    volumes:
      - lattice-data:/data

volumes:
  lattice-data:
```

### 12.3 Environment Configuration

```bash
# Backend selection
STORAGE_BACKEND=redis|ipfs|hybrid

# Redis config
REDIS_URL=redis://127.0.0.1:6379/0
REDIS_TTL_SECONDS=0

# IPFS config
IPFS_PATH=/data/hllset-lattice
IPFS_CACHE_ENABLED=true
IPFS_CACHE_CAPACITY=1000
IPFS_CACHE_TTL_SECONDS=3600

# Server config
HTTP_PORT=8080
LOG_LEVEL=info
```

---

## 14. Appendix: Project Structure

### 13.1 Recommended Application Directory Layout

```
guru-hllset-lattice/
├── Cargo.toml
├── src/
│   ├── main.rs                    # CLI entry point
│   ├── config.rs                  # Configuration parsing
│   ├── app.rs                     # LatticeApp implementation
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── redis.rs               # RedisStorage backend
│   │   ├── ipfs.rs                # IpfsStorage wrapper
│   │   └── hybrid.rs              # HybridStorage (Redis + IPFS)
│   ├── api/
│   │   ├── mod.rs
│   │   └── routes.rs              # REST API handlers
│   └── cli/
│       ├── mod.rs
│       └── commands.rs            # CLI command handlers
├── config/
│   ├── default.yaml
│   ├── redis.yaml
│   └── ipfs.yaml
├── hllset-next/                   # Git submodule or workspace member
│   └── (cloned from github.com/alexmy21/hllset-next)
├── docker-compose.redis.yml
├── docker-compose.ipfs.yml
├── Dockerfile
└── README.md
```

### 13.2 Cargo.toml Dependencies

```toml
[package]
name = "guru-hllset-lattice"
version = "0.1.0"
edition = "2021"

[dependencies]
# hllset-next workspace crates
hllset-core = { path = "hllset-next/crates/hllset-core" }
hllset-dsl = { path = "hllset-next/crates/hllset-dsl" }
hllset-storage = { path = "hllset-next/crates/hllset-storage" }
hllset-materialize = { path = "hllset-next/crates/hllset-materialize" }

# Storage backends
redis = { version = "0.25", optional = true }
ipfrs-core = { path = "hllset-next/crates/ipfrs-core", optional = true }
sled = { version = "0.34", optional = true }

# Web server (optional)
actix-web = { version = "4", optional = true }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# CLI
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"

[features]
default = ["redis-backend"]
redis-backend = ["redis"]
ipfs-backend = ["ipfrs-core", "sled"]
all-backends = ["redis-backend", "ipfs-backend"]
http-api = ["actix-web"]
```

### 13.3 Quick Start Commands

```bash
# 1. Clone the application
git clone <your-app-repo> guru-hllset-lattice
cd guru-hllset-lattice

# 2. Initialize hllset-next submodule
git submodule add https://github.com/alexmy21/hllset-next.git
git submodule update --init --recursive

# 3. Build with Redis backend
cargo build --release --features redis-backend

# 4. Start Redis (if using)
docker run -d --name redis -p 6379:6379 redis:7-alpine

# 5. Run the application
cargo run --release -- --backend redis --redis-url redis://127.0.0.1:6379
```

---

> **References**
>
> - [hllset-next GitHub Repository](https://github.com/alexmy21/hllset-next)
> - [HLLSet Lattice Persistence Protocol (HLPP)](https://github.com/alexmy21/hllset-next/blob/main/_DOCS/dev/HLPP.md)
> - [HLPP Design Notes](https://github.com/alexmy21/hllset-next/blob/main/_DOCS/dev/HLPP_NOTES.md)
> - [Migration Rationale](https://github.com/alexmy21/hllset-next/blob/main/_DOCS/MIGRATION.md)
> - HyperLogLog: Flajolet et al., "HyperLogLog: the analysis of a near-optimal cardinality estimation algorithm" (2007)
> - Roaring Bitmaps: Chambi et al., "Better bitmap performance with Roaring bitmaps" (2016)
