//! Level 1: Token Rank — F(TF).
//!
//! A token's rank is derived from its term frequency via a pluggable function F.
//! The default is identity: F(x) = x. FPGA-native.
//!
//! # Constraints
//! - F must be monotonic: higher TF → not-lower rank
//! - Output is always `Rank` (u64 integer)

use crate::Rank;

/// Pluggable function F: TF → token-R.
///
/// Implementations:
/// - `IdentityRankFn` — F(x) = x (default, FPGA-native)
/// - `Log2RankFn` — F(x) = floor(log2(x)) via leading-zero count (FPGA-native)
pub trait TokenRankFn: Send + Sync {
    /// Compute token rank from term frequency.
    fn rank(&self, tf: u64) -> Rank;

    /// Human-readable name for diagnostics.
    fn name(&self) -> &'static str;
}

/// F(x) = x — the simplest rank function. FPGA-native: integer load.
#[derive(Clone, Copy, Default)]
pub struct IdentityRankFn;

impl TokenRankFn for IdentityRankFn {
    fn rank(&self, tf: u64) -> Rank {
        tf
    }
    fn name(&self) -> &'static str {
        "identity"
    }
}

/// F(x) = floor(log2(x)) via leading-zero count. FPGA-native: LZCNT.
/// Avoids log(0) by returning 0 for tf=0.
#[derive(Clone, Copy, Default)]
pub struct Log2RankFn;

impl TokenRankFn for Log2RankFn {
    fn rank(&self, tf: u64) -> Rank {
        if tf == 0 {
            return 0;
        }
        // floor(log2(tf)) = 63 - leading_zeros(tf)
        (63 - tf.leading_zeros()) as u64
    }
    fn name(&self) -> &'static str {
        "log2"
    }
}

/// Token rank — holds the token's TF and its derived rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenRank {
    pub tf: u64,
    pub value: Rank,
}

impl TokenRank {
    /// Compute token rank from TF using the given function.
    pub fn new(tf: u64, f: &dyn TokenRankFn) -> Self {
        Self {
            tf,
            value: f.rank(tf),
        }
    }

    /// Convenience: compute with default (identity) function.
    pub fn with_identity(tf: u64) -> Self {
        Self::new(tf, &IdentityRankFn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_monotonic() {
        let f = IdentityRankFn;
        assert!(f.rank(0) <= f.rank(1));
        assert!(f.rank(10) <= f.rank(100));
        assert_eq!(f.rank(42), 42);
    }

    #[test]
    fn test_log2() {
        let f = Log2RankFn;
        assert_eq!(f.rank(0), 0);
        assert_eq!(f.rank(1), 0); // floor(log2(1)) = 0
        assert_eq!(f.rank(2), 1);
        assert_eq!(f.rank(7), 2); // floor(log2(7)) = 2
        assert_eq!(f.rank(8), 3);
        assert_eq!(f.rank(1024), 10);
    }

    #[test]
    fn test_token_rank_new() {
        let r = TokenRank::new(42, &IdentityRankFn);
        assert_eq!(r.tf, 42);
        assert_eq!(r.value, 42);
    }

    #[test]
    fn test_log2_monotonic() {
        let f = Log2RankFn;
        let values: Vec<Rank> = (0..20).map(|tf| f.rank(tf)).collect();
        for w in values.windows(2) {
            assert!(w[0] <= w[1], "not monotonic at {} -> {}", w[0], w[1]);
        }
    }
}
