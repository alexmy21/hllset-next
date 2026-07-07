# Migration Rationale and Feasibility Assessment

**hllset_dsl → hllset-next** | July 2026 | Experimental POC

---

## 1. Motivation

The original `hllset_dsl` project is a production-targeted Forth DSL for
content-addressed probabilistic set operations (HLLSet algebra, BSS morphisms,
R-link topology, temporal time pyramids, holographic memory). It works:
179 tests, 0 failures, clean architecture.

However, its infrastructure layer has a **language fragmentation** problem:

| Component | Language | Role |
| --------------------- | ------------ | ------------------------------- |
| HLLSet algebra | Rust | Core computation |
| Forth DSL + Lua VM | Rust | Canonical interface |
| Storage (HLPP) | Go | IPFS daemon via HTTP API |
| Messaging | Python | ROS 2 pub/sub via subprocess |

Three languages, three runtimes, three build systems. This is manageable
for a research project but creates friction for deployment, reproducibility,
and long-term maintenance. The vision: **unify everything under Rust.**

Two projects from the COOLJAPAN ecosystem (Team Kitasan) offer Rust-native
replacements:

| Replacement   | Replaces  | Description                                           |
|---------------|-----------|-------------------------------------------------------|
| **ipfrs**     | Go IPFS   | Content-addressed storage, CID/IPLD, P2P networking   |
| **mielin**    | ROS 2     | Distributed agent mesh, DHT, QUIC transport           |

---

## 2. Scale Assessment (First-Principles)

Before committing to integration, we measured the actual code surface:

| Project | Rust source lines | Crates | Maturity |
| -------------- | ------------------- | -------- | ---------- |
| hllset_dsl | ~6,500 | 7 | Stable (179 tests) |
| ipfrs | ~661,000 | 12 | v0.2.0 |
| mielin | ~199,000 | 10 | v0.1.0-rc.1 |

The ratio is stark: ipfrs is ~100x larger, mielin ~30x larger. Pulling in
the full dependency trees would be architectural overreach. The question
became: **what's the minimum viable subset?**

### What hllset_dsl Actually Needs

**From IPFS (→ ipfrs):** The `Storage` trait is 6 methods on 48 lines:

```rust
fn store(&self, key: &str, data: &[u8]) -> Result<()>;
fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;
fn exists(&self, key: &str) -> Result<bool>;
fn delete(&self, key: &str) -> Result<bool>;
fn list(&self, prefix: &str) -> Result<Vec<String>>;
fn pin/unpin/gc(...)  // optional
```

The current implementation is a 150-line HTTP wrapper calling a Go IPFS
daemon. **ipfrs-core alone** (~17,600 lines) provides everything needed:
`Block`, `Cid`, `Ipld`, chunking, streaming, CAR format — without pulling
in ipfrs-network (147K lines), ipfrs-semantic (140K), or ipfrs-tensorlogic
(150K).

**From ROS 2 (→ mielin):** The ROS 2 integration is ~400 lines of Python
that spawn the hllset CLI binary as a subprocess and use ROS topics for
pub/sub. It's a thin message bus wrapper. **mielin-mesh** (~41K lines:
core + wire) provides DHT + QUIC transport, but for the initial POC we
chose an even lighter approach — an in-process tokio broadcast channel
bus with a trait abstraction (`MeshBus`) that mielin-mesh can plug into
later.

---

## 3. Two Migration Strategies Considered

### Option A: "Surgical" — Minimal integration, thin adapters

Depend on `ipfrs-core` for content-addressing primitives. Write a
~150-line adapter implementing the `Storage` trait. Replace ROS 2 with
a trait-based mesh bus, starting in-process, with a mielin-mesh backend
as a future step. Keep all existing crates unchanged.

**Pros:** Minimal dependency footprint. Fast compilation. Low risk.
Keeps the Forth DSL and existing architecture intact. 189 tests pass.

**Cons:** Doesn't yet achieve distributed networking. ipfrs-core is
still a v0.x external dependency.

### Option B: "Greenfield" — Full rewrite, cherry-pick core

Extract HLLSet algebra, Forth DSL, materializer. Build new storage and
networking from scratch. Drop the Lua VM in favor of pure Forth→Rust
compilation. Use mielin-mesh for distributed transport and ipfrs-storage
for production-grade block storage.

**Pros:** Cleanest possible result. Full control over dependency graph.

**Cons:** High up-front cost. Discards 179 working tests. Risk of losing
subtle invariants encoded in the existing codebase. Both ipfrs and mielin
are pre-1.0, creating a fragile triple-dependency.

### Decision: Option A (Surgical)

The POC is experimental ground. Production stays on the original project.
The goal is to prove feasibility and create a foundation for future
iteration — not to ship tomorrow.

Option A achieves:

1. **Proof of concept**: Storage works with ipfrs-core. Mesh bus works
   without ROS 2. The core algebra is resilient to infrastructure changes.

2. **Ground for future development**: The `MeshBus` trait and `Storage`
   trait are designed for swap-in distributed backends. When mielin-mesh
   and ipfrs-storage stabilize, integration is a thin adapter away.

3. **Conceptual clarity**: The migration confirms that the integration
   surface is well-designed. The core (HLLSet algebra, Forth DSL, Lua VM)
   has zero dependency on IPFS or ROS — they're pure computation behind
   clean trait boundaries.

---

## 4. What This POC Achieves

| Objective | Status |
| ----------------------------------------------- | -------- |
| Replace Go IPFS daemon with Rust-native storage | Done — `IpfrsNativeStorage` (sled + ipfrs-core) |
| Replace ROS 2 Python with Rust-native messaging | Done — `hllset-mesh` (InProcessBus + MeshBus trait) |
| Keep Forth DSL unchanged | Done — all 15 Forth tests pass |
| Keep Lua VM unchanged | Done — all 93 DSL tests pass |
| Zero changes to core algebra | Done — 47 core tests pass |
| Pure Rust build (`cargo build` only) | Done |
| Original project untouched | Done |

---

## 5. Architecture of the New Crates

### hllset-storage (modified)

```text
┌─────────────────────────────────────┐
│         Storage trait (unchanged)   │
│  store / load / exists / delete     │
│  list / pin / unpin / gc            │
├─────────────────────────────────────┤
│  MemoryStorage (unchanged)          │  ← HashMap, dev/testing
│  IpfrsNativeStorage (NEW)           │  ← sled::Db + ipfrs-core::Block/Cid
│  CacheStorage (unchanged)           │  ← LRU wrapper
└─────────────────────────────────────┘

Dependencies: ipfrs-core, sled (replaces ureq for HTTP→Go IPFS)
```

### hllset-mesh (new)

```text
┌─────────────────────────────────────┐
│         MeshBus trait               │
│  publish(topic, msg) / subscribe(topic)
├─────────────────────────────────────┤
│  InProcessBus                       │  ← tokio::broadcast (single process)
│  (future) MielinMeshBus             │  ← mielin-mesh DHT + QUIC
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│  AlgebraNode                        │  ← Ingests text → HLLSet keys
│  WorkerNode                         │  ← Stateless op execution
│  NoetherController                  │  ← Flux monitoring + stability
└─────────────────────────────────────┘

Replaces: hllset_ros/ (Python package, ~400 lines)
```

---

## 6. What's NOT Done (Future Work)

### 6.1 Distributed Mesh Networking

The `InProcessBus` is single-process. The `MeshBus` trait is designed so
`mielin-mesh` (`MeshService`, DHT, QUIC) can be plugged in as a drop-in
replacement. This enables:

- Multi-node algebra clusters
- Remote worker dispatch
- Distributed Noether flux monitoring

### 6.2 Distributed Content-Addressed Storage

`IpfrsNativeStorage` is a single-node sled database. The `Storage` trait
is ready for a replicated backend:

- mielin-mesh-replicated sled instances (RAFT or gossip-based)
- ipfrs-storage's `SledBlockStore` with its pin/GC/compression features
- Full HLPP protocol reimplementation on ipfrs primitives

### 6.3 CID-Based Deduplication

Currently `IpfrsNativeStorage` stores data by HLLSet key (`h:<sha1>`)
but doesn't yet use ipfrs-core's CID for content-aware deduplication
or DAG construction. The `Block` and `Cid` types are imported and
available — the deduplication logic is the next step.

### 6.4 Production Hardening

- Remove POC warnings (unused fields, imports)
- Add integration tests for mesh nodes
- Benchmark storage backend vs original HTTP-to-Go-IPFS
- Profile Lua VM overhead vs direct Rust calls in mesh nodes

---

## 7. Dependency Map

```text
hllset-cli ──────────────┐
    │                    │
    ├── hllset-dsl ──────┤
    │     ├── hllset-core
    │     ├── hllset-storage ── ipfrs-core, sled
    │     └── hllset-materialize ── hllset-duckdb
    │
    ├── hllset-forth ──── hllset-core
    │
    └── hllset-mesh ───── hllset-dsl, hllset-storage, tokio
         (MeshBus trait ── ready for mielin-mesh swap-in)
```

---

## 8. Conclusions

1. **The migration is feasible.** The core algebra, Forth DSL, and Lua VM
   are completely isolated from infrastructure concerns behind clean trait
   boundaries. The integration layer is thin and well-designed.

2. **Surgical integration is the right approach for now.** ipfrs (~661K
   lines) and mielin (~199K lines) are massive, pre-1.0 projects. Pulling
   them in fully would create fragile dependencies. Using only ipfrs-core
   (~17K lines) for content-addressing and a trait-based mesh bus with
   future mielin-mesh swap-in keeps the POC lean and testable.

3. **The Forth DSL remains the canonical interface.** ipfrs and mielin
   have their own CLIs, but those are infrastructure management tools
   (init, add, gateway, mesh start, agent deploy). hllset's CLI is a DSL
   compiler/interpreter — a different layer entirely. No conflict.

4. **189 tests, 0 failures.** The migration proves that the original
   project's architecture is resilient. Infrastructure changes did not
   touch the core — exactly what well-designed trait boundaries enable.

5. **This POC is ground for future development, not a replacement.**
   The original `hllset_dsl` remains the production target. This project
   exists to explore, prove, and derisk the path toward a unified Rust
   platform.
