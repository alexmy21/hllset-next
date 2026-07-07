//! hllset-forth — Forth DSL frontend for HLLSet algebra.
//!
//! Parses Forth source into an AST, then lowers to backend-specific code.
//! The AST is the canonical representation — one source, many targets.
//!
//! # Architecture
//!
//! ```text
//! Forth source → [parse] → AST → [lower_lua] → Lua script (hllset CLI)
//!                            → [lower_rust] → Rust code (embedded)
//!                            → [lower_hw] → Verilog (FPGA)
//! ```
//!
//! # Forth Syntax (subset)
//!
//! Forth is postfix, stack-based, whitespace-delimited. Every token is a word.
//!
//! ```forth
//! \ comment to end of line
//! ( block comment )
//!
//! \ String literals
//! "hello world"
//!
//! \ Numbers
//! 42  3.14  -1
//!
//! \ Words — everything else
//! DUP SWAP INSCRIBE TOKENIZE STORE LOAD
//! ```
//!
//! # HLLSet Words
//!
//! | Word | Stack effect | Description |
//! |------|-------------|-------------|
//! | `INSCRIBE` | ( tokens... n -- hllset ) | Create HLLSet from n tokens |
//! | `TOKENIZE` | ( text -- hllset ) | Tokenize text string |
//! | `EMPTY` | ( -- hllset ) | Empty HLLSet (bottom) |
//! | `UNION` | ( a b -- c ) | a ∪ b |
//! | `INTERSECT` | ( a b -- c ) | a ∩ b |
//! | `DIFF` | ( a b -- c ) | a - b |
//! | `CARD` | ( h -- n ) | Estimated cardinality |
//! | `POPCOUNT` | ( h -- n ) | Bits set |
//! | `BSS` | ( a b -- τ ) | BSS inclusion |
//! | `STORE` | ( h -- ) | Persist HLLSet |
//! | `LOAD` | ( cid -- h|nil ) | Load by CID |
//! | `LIST` | ( prefix -- cids ) | List CIDs by prefix |
//! | `PIN` | ( cid -- ) | Pin for GC |
//! | `UNPIN` | ( cid -- ) | Unpin |
//! | `GC` | ( -- removed ) | Garbage collect |
//! | `KEY` | ( h -- cid ) | Get content key |
//! | `DUP` | ( x -- x x ) | Duplicate top of stack |
//! | `SWAP` | ( x y -- y x ) | Swap top two |
//! | `DROP` | ( x -- ) | Discard top |
//! | `OVER` | ( x y -- x y x ) | Copy second to top |
//! | `ROT` | ( x y z -- y z x ) | Rotate third to top |

pub mod ast;
pub mod lower_lua;
pub mod parse;

/// Re-export the main types.
pub use ast::{Ast, Word};
pub use lower_lua::compile_to_lua;
pub use parse::parse;
