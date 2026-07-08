//! Level 2: Bit Rank — G({token-R}).
//!
//! Multiple tokens may hash to the same (register, tz) bit position.
//! G aggregates their token-ranks into a single bit-rank.
//!
//! FPGA-native choices: Max (CMP) or Sum (ADD).

use crate::token::TokenRank;
use crate::Rank;

/// Pluggable aggregator: {token-R} → bit-R.
///
/// Implementations:
/// - `MaxAggregator` — strongest token controls the bit (CMP, FPGA-native)
/// - `SumAggregator` — all tokens contribute (ADD, FPGA-native)
pub trait BitRankAggregator: Send + Sync {
    /// Aggregate multiple token ranks at a single bit position.
    fn aggregate(&self, tokens: &[TokenRank]) -> Rank;

    fn name(&self) -> &'static str;
}

/// Max aggregator — the dominant token's rank IS the bit's rank.
///
/// FPGA-native: comparator tree (31 CMPs for 32 entries).
#[derive(Clone, Copy, Default)]
pub struct MaxAggregator;

impl BitRankAggregator for MaxAggregator {
    fn aggregate(&self, tokens: &[TokenRank]) -> Rank {
        tokens.iter().map(|t| t.value).max().unwrap_or(0)
    }
    fn name(&self) -> &'static str {
        "max"
    }
}

/// Sum aggregator — all token ranks add together.
///
/// FPGA-native: integer adders (31 ADDs for 32 entries).
#[derive(Clone, Copy, Default)]
pub struct SumAggregator;

impl BitRankAggregator for SumAggregator {
    fn aggregate(&self, tokens: &[TokenRank]) -> Rank {
        tokens.iter().map(|t| t.value).sum()
    }
    fn name(&self) -> &'static str {
        "sum"
    }
}

/// Bit rank at a specific (register, tz) position — the result of G.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BitRank {
    pub register: u32,
    pub tz: u32,
    pub value: Rank,
    /// How many tokens hash to this position.
    pub token_count: usize,
}

impl BitRank {
    /// Compute bit rank from the tokens that hash to (reg, tz).
    pub fn new(register: u32, tz: u32, tokens: &[TokenRank], g: &dyn BitRankAggregator) -> Self {
        Self {
            register,
            tz,
            value: g.aggregate(tokens),
            token_count: tokens.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::IdentityRankFn;

    fn make_tokens(tfs: &[u64]) -> Vec<TokenRank> {
        tfs.iter()
            .map(|&tf| TokenRank::new(tf, &IdentityRankFn))
            .collect()
    }

    #[test]
    fn test_max_aggregator() {
        let g = MaxAggregator;
        let tokens = make_tokens(&[10, 42, 7, 100]);
        assert_eq!(g.aggregate(&tokens), 100);
        assert_eq!(g.aggregate(&[]), 0);
    }

    #[test]
    fn test_sum_aggregator() {
        let g = SumAggregator;
        let tokens = make_tokens(&[10, 20, 30]);
        assert_eq!(g.aggregate(&tokens), 60);
        assert_eq!(g.aggregate(&[]), 0);
    }

    #[test]
    fn test_bit_rank_new() {
        let g = MaxAggregator;
        let tokens = make_tokens(&[5, 15, 3]);
        let br = BitRank::new(42, 17, &tokens, &g);
        assert_eq!(br.register, 42);
        assert_eq!(br.tz, 17);
        assert_eq!(br.value, 15);
        assert_eq!(br.token_count, 3);
    }
}
