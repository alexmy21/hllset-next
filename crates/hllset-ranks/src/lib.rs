//! Five-level rank algebra for HLLSet lattice.
//!
//! All rank values are `u64` integers — FPGA-native (AND, OR, POPCOUNT, ADD, SUB, CMP).
//! Every level has a pluggable trait so the application can choose the aggregation function.
//!
//! ```text
//! Level 5: compound rank   L(max{R}), M(min{R})
//! Level 4: hllset rank     K(degree in lattice)
//! Level 3: register rank   H({bit-R[tz] | tz ∈ 0..31})
//! Level 2: bit rank        G({token-R | hash → (reg, tz)})
//! Level 1: token rank      F(TF)
//! ```

pub mod token;
pub mod bit;
pub mod register;
pub mod hllset;
pub mod compound;
pub mod derivatives;
pub mod fisher;
pub mod mask;

pub use token::{TokenRank, TokenRankFn};
pub use bit::{BitRank, BitRankAggregator};
pub use register::{RegisterRank, RegisterRankAggregator};
pub use hllset::{HLLSetRank, HLLSetRankFn};
pub use compound::CompoundRank;
pub use derivatives::{RankDerivatives, NoetherSteering};
pub use fisher::{FisherMatrix, FisherProjection};
pub use mask::{ObservableMask, SubLatticeDegree};

/// The universal rank type — all levels produce `u64`.
///
/// FPGA-native: every operation on Rank is AND, OR, POPCOUNT, ADD, SUB, or CMP.
pub type Rank = u64;

/// Threshold for the observable mask O(θ).
pub type Threshold = Rank;
