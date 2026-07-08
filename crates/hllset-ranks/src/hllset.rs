//! Level 4: HLLSet Rank — K(degree).
//!
//! An HLLSet's structural rank is derived from its position in the lattice DAG.
//! The default is simple degree: count of incident edges (operations that produced
//! or consumed this HLLSet).
//!
//! FPGA-native: POPCOUNT of the adjacency bitmask row.

use crate::Rank;
use hllset_dsl::LatticeElement;
use std::collections::HashMap;

/// Pluggable function: lattice position → hllset-R.
///
/// Implementations:
/// - `DegreeRankFn` — K = degree = count of incident edges (POPCOUNT, FPGA-native)
pub trait HLLSetRankFn: Send + Sync {
    /// Compute HLLSet rank from its lattice context.
    fn rank(&self, key: &str, degree: usize, _popcount: u64) -> Rank;

    fn name(&self) -> &'static str;
}

/// Degree-based rank — the simplest structural measure.
///
/// FPGA-native: degree is just a POPCOUNT of the adjacency row.
#[derive(Clone, Copy, Default)]
pub struct DegreeRankFn;

impl HLLSetRankFn for DegreeRankFn {
    fn rank(&self, _key: &str, degree: usize, _popcount: u64) -> Rank {
        degree as u64
    }
    fn name(&self) -> &'static str {
        "degree"
    }
}

/// Weighted degree — degree weighted by the popcount of each incident R-link.
///
/// For each neighbor H', the edge contributes popcount(H ∩ H') to the weighted degree.
/// FPGA-native: AND + POPCOUNT per edge, then ADD across edges.
#[derive(Clone, Copy, Default)]
pub struct WeightedDegreeRankFn;

impl HLLSetRankFn for WeightedDegreeRankFn {
    fn rank(&self, _key: &str, _degree: usize, popcount: u64) -> Rank {
        popcount
    }
    fn name(&self) -> &'static str {
        "weighted-degree"
    }
}

/// HLLSet rank — the result of K.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HLLSetRank {
    pub key: String,
    pub value: Rank,
    /// Raw degree in the lattice graph.
    pub degree: usize,
    /// Popcount of the HLLSet itself (for weighted-degree computations).
    pub popcount: u64,
}

impl HLLSetRank {
    /// Compute HLLSet rank from its lattice context.
    pub fn new(elem: &LatticeElement, degree: usize, k: &dyn HLLSetRankFn) -> Self {
        let popcount = elem.popcount();
        Self {
            key: elem.key().to_string(),
            value: k.rank(elem.key(), degree, popcount),
            degree,
            popcount,
        }
    }

    /// Create from raw data (when LatticeElement is not available).
    pub fn from_raw(key: &str, degree: usize, popcount: u64, k: &dyn HLLSetRankFn) -> Self {
        Self {
            key: key.to_string(),
            value: k.rank(key, degree, popcount),
            degree,
            popcount,
        }
    }
}

/// A collection of HLLSet ranks, keyed by content key.
#[derive(Debug, Clone, Default)]
pub struct HLLSetRankIndex {
    ranks: HashMap<String, HLLSetRank>,
}

impl HLLSetRankIndex {
    pub fn new() -> Self {
        Self {
            ranks: HashMap::new(),
        }
    }

    pub fn insert(&mut self, rank: HLLSetRank) {
        self.ranks.insert(rank.key.clone(), rank);
    }

    pub fn get(&self, key: &str) -> Option<&HLLSetRank> {
        self.ranks.get(key)
    }

    pub fn len(&self) -> usize {
        self.ranks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ranks.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &HLLSetRank> {
        self.ranks.values()
    }

    /// Update a degree — used when the lattice graph changes.
    pub fn update_degree(&mut self, key: &str, new_degree: usize, k: &dyn HLLSetRankFn) -> bool {
        if let Some(rank) = self.ranks.get_mut(key) {
            rank.degree = new_degree;
            rank.value = k.rank(key, new_degree, rank.popcount);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hllset_core::HLLSet;

    #[test]
    fn test_degree_rank() {
        let k = DegreeRankFn;
        assert_eq!(k.rank("h:test", 5, 100), 5);
        assert_eq!(k.rank("h:empty", 0, 0), 0);
    }

    #[test]
    fn test_weighted_degree_rank() {
        let k = WeightedDegreeRankFn;
        assert_eq!(k.rank("h:test", 5, 100), 100);
    }

    #[test]
    fn test_hllset_rank_from_elem() {
        let tokens = &["hello", "world"];
        let elem = LatticeElement::from_tokens(tokens);
        let k = DegreeRankFn;
        let r = HLLSetRank::new(&elem, 3, &k);
        assert_eq!(r.key, elem.key().to_string());
        assert_eq!(r.value, 3); // degree = 3
        assert!(r.popcount > 0);
    }

    #[test]
    fn test_rank_index() {
        let mut idx = HLLSetRankIndex::new();
        let r1 = HLLSetRank::from_raw("h:a", 2, 50, &DegreeRankFn);
        let r2 = HLLSetRank::from_raw("h:b", 5, 200, &DegreeRankFn);
        idx.insert(r1);
        idx.insert(r2);
        assert_eq!(idx.len(), 2);
        assert_eq!(idx.get("h:a").unwrap().value, 2);
        assert_eq!(idx.get("h:b").unwrap().value, 5);

        // Update degree
        idx.update_degree("h:a", 10, &DegreeRankFn);
        assert_eq!(idx.get("h:a").unwrap().value, 10);
    }
}
