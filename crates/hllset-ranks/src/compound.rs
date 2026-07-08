//! Level 5: Compound Rank — L(max), M(min).
//!
//! When HLLSets combine via lattice operations, compound ranks propagate:
//!   rank(A ∪ B) = L(max{rank(A), rank(B)})
//!   rank(A ∩ B) = M(min{rank(A), rank(B)})
//!
//! FPGA-native: CMP (comparator).

use crate::Rank;

/// Compound rank for a lattice operation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompoundRank {
    pub value: Rank,
    /// Rank of the left operand.
    pub left_value: Rank,
    /// Rank of the right operand.
    pub right_value: Rank,
}

impl CompoundRank {
    /// Union rank: L = max(left, right).
    /// FPGA-native: single CMP.
    pub fn union(left: Rank, right: Rank) -> Self {
        Self {
            value: left.max(right),
            left_value: left,
            right_value: right,
        }
    }

    /// Intersection rank: M = min(left, right).
    /// FPGA-native: single CMP.
    pub fn intersection(left: Rank, right: Rank) -> Self {
        Self {
            value: left.min(right),
            left_value: left,
            right_value: right,
        }
    }

    /// Difference rank: inherits the left operand's rank.
    /// A - B = the part of A not in B — structurally closer to A.
    pub fn difference(left: Rank, _right: Rank) -> Self {
        Self {
            value: left,
            left_value: left,
            right_value: _right,
        }
    }

    /// Symmetric difference: max of the two ranks.
    /// A ⊕ B = (A - B) ∪ (B - A) — structurally the "conflict" between them.
    pub fn symmetric_difference(left: Rank, right: Rank) -> Self {
        Self {
            value: left.max(right),
            left_value: left,
            right_value: right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union_max() {
        let r = CompoundRank::union(10, 42);
        assert_eq!(r.value, 42);
        assert_eq!(r.left_value, 10);
        assert_eq!(r.right_value, 42);
    }

    #[test]
    fn test_intersection_min() {
        let r = CompoundRank::intersection(10, 42);
        assert_eq!(r.value, 10);
    }

    #[test]
    fn test_difference_inherits_left() {
        let r = CompoundRank::difference(100, 999);
        assert_eq!(r.value, 100);
    }

    #[test]
    fn test_symmetric_difference_max() {
        let r = CompoundRank::symmetric_difference(5, 95);
        assert_eq!(r.value, 95);
    }

    #[test]
    fn test_idempotent_union() {
        let r = CompoundRank::union(7, 7);
        assert_eq!(r.value, 7);
    }

    #[test]
    fn test_idempotent_intersection() {
        let r = CompoundRank::intersection(7, 7);
        assert_eq!(r.value, 7);
    }
}
