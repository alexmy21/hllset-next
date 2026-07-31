# HLLSet DSL — Next Iteration (hllset-next)

**Unified Rust platform for HLLSet Algebra.** Implements the full
[STANDARD.md](_DOCS/dev/STANDARD.md) specification: IICA-gated operations,
five-level rank algebra, temporal pyramid, universal bridge, and
content-addressed namespace taxonomy.

**Developer tip.** HLLSet Algebra is pattern matching in a bitmask lattice.
The notation is precise, but the concepts are familiar:
`tokenize` = compile text into a bitmask; `union` = OR; `intersection` = AND;
`bss_inclusion` = confidence that pattern B matches within pattern A;
`materialize` = which known tokens match these bits?;
`Noether steering` = pattern drift detector;
`Fisher matrix` = temporal co-occurrence of sub-patterns;
`O(θ)` = which patterns exceed a relevance threshold;
`TemporalPyramid` = L0–L6 sliding window with automatic carry cascade;
`UniversalBridge` = cross-domain re-representation with Spearman ranking.
Same code, same FPGA, just different names for the same operations.

## Why This Exists

The original [`hllset_dsl`](../hllset_dsl/) project is a production-targeted
Forth DSL for content-addressed probabilistic set operations. It works, but
its infrastructure dependencies are non-Rust:

| Dependency | Language | Role |
| -------------- | ------------ | -------------------------- |
| IPFS daemon | Go | Content-addressed storage |
| ROS 2 | Python/C++ | Pub/sub messaging |

This project proves that both can be replaced with Rust-native equivalents
without changing the core algebra, Forth DSL, or Lua runtime. The result is
a **single-language platform** — build, test, and deploy with `cargo` alone.

The original project is untouched. This is experimental ground.

## Architecture

```text
hllset-next/
├── Cargo.toml                 # Workspace manifest (13 crates)
├── crates/
│   ├── hllset-core/           # HLLSet bitmap, hashing, BSS, TFVec, Commit, CID taxonomy
│   ├── hllset-dsl/            # Lua VM, tokenizer, materializer, De Bruijn
│   ├── hllset-forth/          # Forth parser → AST → Lua compiler, colon-definitions
│   ├── hllset-ranks/          # Five-level rank algebra, derivatives, Fisher, mask, TfRegisterRanker
│   ├── hllset-materialize/    # Pluggable materialization trait
│   ├── hllset-storage/        # HLPP Storage trait + MemoryStorage + IpfrsNative (sled)
│   ├── hllset-storage-redis/  # RedisStorage backend (enterprise)
│   ├── hllset-temporal/       # NEW: L0–L6 configurable temporal pyramid
│   ├── hllset-bridge/         # NEW: Universal cross-domain re-representation
│   ├── hllset-mesh/           # In-process pub/sub bus + Noether controller
│   ├── hllset-duckdb/         # Chunked LUT engine
│   └── hllset-cli/            # CLI: Lua -e, Forth --forth, REPL, mesh commands
├── _DOCS/
│   ├── dev/STANDARD.md        # Authoritative architecture specification
│   └── notebooks/             # 14 Jupyter notebooks (Rust kernel)
└── redis/                     # Redis container with Roaring Bitmap modules
```

## Quick Start

```bash
# Build
cargo build

# Lua evaluation
cargo run -- -e 'return hllset.tokenize("hello world"):key()'

# Forth DSL (with colon-definitions)
cargo run -- --forth '"neural" "network" 2 INSCRIBE KEY'
cargo run -- --forth ': DOUBLE 2 * ; 5 DOUBLE'

# Interactive REPL
cargo run -- --repl

# Mesh: start algebra node
cargo run -- --mesh-algebra

# Mesh: start Noether flux controller (integer threshold)
cargo run -- --mesh-noether 5
```

## Test Suite

```bash
cargo test
# 291 tests, 0 failures (13 crates)
```

## Notebooks

All 14 notebooks execute in the Rust (evcxr) Jupyter kernel.

| # | Notebook | Description |
| --- | ---------- | ------------- |
| 01 | `hllset_core` | HLLSet basics, BSS morphisms, lattice operations |
| 02 | `tokenizer_materialization` | Tokenizer pipeline, LUT construction, materialization |
| 03 | `client_demo` | External client: ingestion, comparison, storage |
| 04 | `algebraic_chunk_space` | IICA: chunked LUT, closure, BSS vector, Merkle tree |
| 05 | `iica_forth` | Forth DSL: immutable, idempotent, content-addressed |
| 06 | `fpga_self_reprogram` | FPGA self-reprogramming, DRN evolution, temporal layers |
| 07 | `secure_exchange` | Secure HLLSet exchange protocol |
| 08 | `holographic_memory` | Holographic lattice memory, TF time lens |
| 09 | `rank_algebra` | Five-level rank algebra |
| 10 | `multi_lattice_dimensions` | Multi-perceptron world model, swarm, time travel |
| 11 | `caal_llm_demo` | CAAL-LLM: Content-addressed Chinese LLM + I Ching |
| 11b | `redis_bridge` | Redis storage backend validation |
| **12** | **`dsl_user_guide`** | **DSL User Guide: tokenization, algebra, storage, temporal, Forth, IICA** |
| **13** | **`advanced_algebra`** | **Advanced: TFVec, Commit, ranks, temporal pyramid, bridge, CID taxonomy** |

## CAAL-LLM: Content-Addressed LLM Proof

**Notebook 11** demonstrates a content-addressed Chinese LLM. The result:

```text
Training:  10 Chinese sentences (~100 characters, driving rules)
Questions: 5 driving scenario questions
Correct:   4/5 (80%)
```

No gradient descent. No weight matrices. No GPU. No transformer.
Just murmurhash3 + bitwise AND + popcount. MS-DOS capable.

This validates two principles:

1. **Chinese as assembly language** — characters ARE tokens, fixed set, deterministic
2. **Context (HLLSet) based LLM** — learning = accumulating HLLSets; inference = BSS retrieval

A GPT needs billions of tokens. This needs 10 sentences and a hash function.

The same notebook runs the I Ching pipeline: scene → BSS consultation → hexagram
→ R-link navigation → strategic guidance. See `_DOCS/dev/CAAL_ICHING_ARCHITECTURE.md`
and `../caal-llm/` for the standalone Rust crate.

## Storage Backends — The Trait-Boundary Design

Every storage backend implements the `Storage` trait (11 methods: `put`, `get`,
`has`, `delete`, `list`, `pin`, `unpin`, `gc` for content-addressed operations;
`put_tmp`, `get_tmp`, `cas_tmp` for temporal state). Legacy aliases
(`store`/`load`/`exists`) delegate to the canonical HLPP names.

Switching backends is a one-line change — everything above the trait
(Lua runtime, materializer, DuckDB LUT, ingest pipeline, mesh nodes,
rank algebra, temporal pyramid) works identically.

| Backend | Crate | Use case | Tests |
| --------- | ------- | ---------- | --- |
| `MemoryStorage` | `hllset-storage` | Development, testing (full temporal support) | 19 |
| `IpfrsNativeStorage` | `hllset-storage` | Local (sled, no daemon) | 13 |
| `RedisStorage` | `hllset-storage-redis` | Enterprise (Redis + Roaring Bitmap) | 5 |

**Redis quick start:**

```bash
# Build and start the Redis container
podman build -t hllset-redis -f redis/Dockerfile .
podman run -d --name hllset-redis -p 6379:6379 hllset-redis

# Use it in Rust
let store = RedisStorage::connect("redis://127.0.0.1:6379").unwrap();
store.put("h:abc123", &hllset_bytes).unwrap();
```

## Key Features (July 2026)

### Core Algebra

- **HLLSet bitmap** (1024×32): union, intersection, difference, XOR — all O(1) bitwise
- **BSS morphisms**: inclusion (τ), exclusion (ρ), morphism check — float-based similarity
- **R-links**: topological intersection HLLSets — composable, content-addressed, FPGA-native
- **Cardinality**: Horvitz-Thompson estimator, monotonic guaranteed
- **Content addressing**: SHA-1 keys with full namespace taxonomy (o/h/r/d/n/t/v/l/c/u + system:)

### Storage Protocol (HLPP)

- **CA operations**: `put`/`get`/`has`/`list`/`pin`/`unpin`/`gc` — idempotent, IICA-compliant
- **Temporal operations**: `put_tmp`/`get_tmp`/`cas_tmp` — atomic compare-and-swap
- **3 backends**: Memory (19 tests), Sled/IPFS-native (13 tests), Redis (5 tests)
- **Legacy compatibility**: `store`/`load`/`exists` aliases delegate to canonical names

### Rank Algebra (hllset-ranks)

- **Five-level**: token → bit → register → HLLSet → compound — integer-only, FPGA-native
- **Derivatives**: ΔR (first-order), Δ²R (acceleration), rank flux, Noether steering
- **Fisher matrix**: sparse cross-layer bit co-occurrence, systemic vs noise detection
- **Observable mask**: rank-threshold attention filter — controls visibility, not existence
- **TfRegisterRanker**: TF vector → 1,024 register-level ranks — no TokenLUT needed

### Temporal Pyramid (hllset-temporal)

- **Configurable N-layer**: 7-layer default (second→year), tunable to any scale
- **Automatic carry cascade**: time-boundary detection, layer merge, reset
- **System state**: H_system = ∪L_i — bit-lossless union of all layers
- **TF snapshots**: per-layer and system-wide TF vectors for time-lens queries
- **Noether invariant**: structural guarantee of convergence without coordination
- **Presets**: standard, high-frequency, realtime-control, document-analysis, minimal

### Universal Bridge (hllset-bridge)

- **Two-pass ingestion**: representation (domain→HLLSet) + re-representation (bit→bridge)
- **3-gram fingerprinting**: structural invariant for cross-domain matching
- **Spearman rank correlation**: ranks vectors, computes ρ ∈ [-1,1]
- **Bridge pipeline**: re-represent → fingerprint → rank-correlate → top-K matches
- **Statistics constraint**: transfers structure, not TF — each domain learns independently

### DSL & Tooling

- **Lua runtime**: `hllset -e '<script>'` with full algebra + storage + temporal bindings
- **Forth DSL**: `hllset --forth '<code>'` with colon-definitions → Lua compilation
- **REPL**: interactive mode with shared runtime, persistent Lua variables
- **Mesh**: in-process tokio broadcast bus, Noether flux controller (integer)
- **14 notebooks**: from core algebra to advanced bridge, all Rust-kernel executable

## Key Changes from Original

| Aspect | Original (hllset_dsl) | hllset-next (current) |
| -------- | ---------------------- | ------------------- |
| Storage | HTTP to Go IPFS daemon | `ipfrs-core` + `sled` (local) + `Redis` (enterprise) |
| Messaging | ROS 2 Python nodes | `hllset-mesh` in-process tokio bus |
| External deps | Go, Python, ROS 2 | None beyond Rust |
| Language mix | Rust + Python + Go | Rust only |
| Rank system | None | Five-level integer algebra + Fisher + mask |
| Temporal | None | Configurable N-layer pyramid + carry cascade |
| Cross-domain | None | Universal bridge + Spearman ranking |
| CID taxonomy | h:/c: only | Full o/r/d/n/t/v/l/c/u + system: |
| Storage trait | 6 methods (CA only) | 11 methods (CA + temporal + CAS) |
| Forth | Parser + basic Lua | Colon-definitions → Lua functions |
| Test count | 212 | 291 |

## What's Next

The `MeshBus` trait in `hllset-mesh` is designed for a distributed transport
swap-in. The obvious candidate is `mielin-mesh` (Kademlia DHT + QUIC from the
MielinOS project), which would enable multi-node mesh networking without ROS 2.

Similarly, `IpfrsNativeStorage` is local-only (single sled database). A
mielin-mesh-replicated storage backend would provide distributed content-addressing
across nodes — reaching feature parity with the original's IPFS-based HLPP
protocol, but entirely in Rust.

**Remaining from STANDARD.md:**

- Self-ingestion pipeline (git commit → HLLSet ingest, llms.txt, folder views)
- caal-llm reference application hardening

See [`_DOCS/dev/STANDARD.md`](_DOCS/dev/STANDARD.md) for the complete
architecture specification and implementation status matrix.
