# HLLSet DSL — Next Iteration (hllset-next)

**Unified Rust platform for HLLSet Algebra.** Experimental POC replacing
Go IPFS daemon and ROS 2 Python with ipfrs-core and a Rust-native mesh bus.

**Developer tip.** HLLSet Algebra is pattern matching in a bitmask lattice.
The notation is precise, but the concepts are familiar:
`tokenize` = compile text into a bitmask; `union` = OR; `intersection` = AND;
`bss_inclusion` = confidence that pattern B matches within pattern A;
`materialize` = which known tokens match these bits?;
`Noether steering` = pattern drift detector;
`Fisher matrix` = temporal co-occurrence of sub-patterns;
`O(θ)` = which patterns exceed a relevance threshold. Same code, same FPGA,
just different names for the same operations.

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
├── Cargo.toml                 # Workspace manifest
├── crates/
│   ├── hllset-core/           # HLLSet algebra (unchanged)
│   ├── hllset-dsl/            # Lua VM, tokenizer, materializer (unchanged)
│   ├── hllset-forth/          # Forth parser + AST (unchanged)
│   ├── hllset-materialize/    # Pluggable materialization (unchanged)
│   ├── hllset-duckdb/         # Chunked LUT engine (unchanged)
│   ├── hllset-storage/        # MODIFIED: ipfrs-core + sled (was HTTP→Go IPFS)
│   ├── hllset-mesh/           # NEW: Rust-native pub/sub (replaces ROS 2)
│   └── hllset-cli/            # CLI + mesh commands
└── _DOCS/
    └── MIGRATION.md           # Full migration rationale and assessment
```

## Quick Start

```bash
# Build
cargo build

# Lua evaluation (unchanged from original)
cargo run -- -e 'return hllset.tokenize("hello world"):key()'

# Forth DSL (unchanged)
cargo run -- --forth '"neural" "network" 2 INSCRIBE KEY'

# Interactive REPL (unchanged)
cargo run -- --repl

# Mesh: start algebra node (new)
cargo run -- --mesh-algebra

# Mesh: start worker (new)
cargo run -- --mesh-worker worker-0

# Mesh: start Noether flux controller (new)
cargo run -- --mesh-noether 0.1
```

## Test Suite

```bash
cargo test
# 212 tests, 0 failures
```

## Notebooks

All 11 notebooks execute with **0 errors**.

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
| 11 | `caal_llm_demo` | **CAAL-LLM: Content-addressed Chinese LLM + I Ching** |

All Python notebooks shell out to the `hllset` CLI binary via subprocess.

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

## Key Changes from Original

| Aspect | Original (hllset_dsl) | New (hllset-next) |
| -------- | ---------------------- | ------------------- |
| Storage | HTTP to Go IPFS daemon (`ureq`) | `ipfrs-core` CID/Block types + `sled` (local) + `Redis` (enterprise) |
| Messaging | ROS 2 Python nodes (subprocess) | `hllset-mesh` in-process tokio bus |
| External deps | Go, Python, ROS 2, rclpy | None beyond Rust |
| Language mix | Rust + Python + Go | Rust only |

## Storage Backends — The Trait-Boundary Design

Every storage backend implements the same `Storage` trait (6 methods: `store`,
`load`, `exists`, `delete`, `list`, `gc`). Switching backends is a one-line
change — everything above the trait (Lua runtime, materializer, DuckDB LUT,
ingest pipeline, mesh nodes, rank algebra) works identically.

| Backend | Crate | Use case | Status |
| --------- | ------- | ---------- | -------- |
| `MemoryStorage` | `hllset-storage` | Development, testing | 13 tests |
| `IpfrsNativeStorage` | `hllset-storage` | Local (sled, no daemon) | 13 tests |
| `RedisStorage` | `hllset-storage-redis` | Enterprise (Redis 7.0.15 + Roaring Bitmap + RediSearch + RedisGraph) | 5 tests |

**Redis quick start:**

```bash
# Build and start the Redis container
podman build -t hllset-redis -f redis/Dockerfile .
podman run -d --name hllset-redis -p 6379:6379 hllset-redis

# Use it in Rust
let store = RedisStorage::connect("redis://127.0.0.1:6379").unwrap();
store.store("h:abc123", &hllset_bytes).unwrap();
```

The Redis container includes `redis-roaring` (compressed HLLSet bitmask
storage), `redisearch` (token LUT indexing), and `redisgraph` (graph engine
for Phase 2+ integration). All processing stays in Rust — Redis is storage only.

See [`_DOCS/notebooks/11_redis_bridge.ipynb`](_DOCS/notebooks/11_redis_bridge.ipynb)
for the validation notebook.

## What's Next

The `MeshBus` trait in `hllset-mesh` is designed for a distributed transport
swap-in. The obvious candidate is `mielin-mesh` (Kademlia DHT + QUIC from the
MielinOS project), which would enable multi-node mesh networking without ROS 2.

Similarly, `IpfrsNativeStorage` is local-only (single sled database). A
mielin-mesh-replicated storage backend would provide distributed content-addressing
across nodes — reaching feature parity with the original's IPFS-based HLPP
protocol, but entirely in Rust.

See [`_DOCS/MIGRATION.md`](_DOCS/MIGRATION.md) for the full migration
rationale, feasibility assessment, and architectural decisions.
