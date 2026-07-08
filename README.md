# HLLSet DSL — Next Iteration (hllset-next)

**Unified Rust platform for HLLSet Algebra.** Experimental POC replacing
Go IPFS daemon and ROS 2 Python with ipfrs-core and a Rust-native mesh bus.

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
# 189 tests, 0 failures
```

## Notebooks

All 8 notebooks from the original project have been copied, path-adjusted,
and verified against `hllset-next`. Every notebook executes with **0 errors**.

| # | Notebook | Language | Description |
| --- | ---------- | ---------- | ------------- |
| 01 | `hllset_core` | Rust (evcxr) | HLLSet basics, BSS morphisms, lattice operations |
| 02 | `tokenizer_materialization` | Rust (evcxr) | Tokenizer pipeline, LUT construction, materialization |
| 03 | `client_demo` | Python | External client: ingestion, comparison, IPFS store/load |
| 04 | `algebraic_chunk_space` | Rust (evcxr) | IICA: chunked LUT, closure, BSS vector, Merkle tree |
| 05 | `iica_forth` | Python | Forth DSL: immutable, idempotent, content-addressed workflow |
| 06 | `fpga_self_reprogram` | Python | FPGA self-reprogramming, DRN evolution, temporal layers |
| 07 | `secure_exchange` | Python | Secure HLLSet exchange protocol with LUT-based understanding |
| 08 | `holographic_memory` | Python | Holographic lattice memory, TF time lens, reconstruction |

Python notebooks shell out to the `hllset` CLI binary via subprocess — same
interface, same JSON output. Rust notebooks use the evcxr Jupyter kernel with
`:dep` directives pointing at the `hllset-next` crates. The zero-error result
confirms that infrastructure changes (storage, messaging) are isolated behind
trait boundaries and do not affect the core algebra or CLI interface.

```bash
# Build the binary (required for Python notebooks)
cargo build --release

# Run a single notebook
jupyter nbconvert --execute --to notebook _DOCS/notebooks/03_client_demo.ipynb

# Run all notebooks
for nb in _DOCS/notebooks/*.ipynb; do
    jupyter nbconvert --execute --to notebook "$nb"
done
```

## Key Changes from Original

| Aspect | Original (hllset_dsl) | New (hllset-next) |
| -------- | ---------------------- | ------------------- |
| Storage | HTTP to Go IPFS daemon (`ureq`) | `ipfrs-core` CID/Block types + `sled` |
| Messaging | ROS 2 Python nodes (subprocess) | `hllset-mesh` in-process tokio bus |
| External deps | Go, Python, ROS 2, rclpy | None beyond Rust |
| Language mix | Rust + Python + Go | Rust only |

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
