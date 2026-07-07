//! LatticeElement — the core DSL type wrapping an HLLSet with a content key.
//!
//! `LatticeElement` is the unit of computation in the HLLSet DSL. It pairs an
//! HLLSet fingerprint with its content-addressable key (`h:<sha1>` or `c:<sha1>`).
//! All operations produce new `LatticeElement`s (immutable lattice semantics).
//!
//! ## Lattice properties
//!
//! LatticeElements form a **bounded distributive lattice** where:
//! - Join (∪, `+`): union via bitwise OR
//! - Meet (∩, `*`): intersection via bitwise AND
//!
//! These operations are associative, commutative, and idempotent — the
//! mathematical foundation for eventually-consistent distributed systems.

use hllset_core::core::bss::BSSResult;
use hllset_core::HLLSet;

/// A lattice element: an HLLSet fingerprint with its content-addressable key.
///
/// # Examples
///
/// ```rust
/// use hllset_dsl::LatticeElement;
/// use hllset_core::HLLSet;
///
/// let hll = HLLSet::from_tokens(&["hello", "world"]);
/// let elem = LatticeElement::new(hll);
/// println!("key: {}", elem.key());
/// ```
#[derive(Clone, Debug)]
pub struct LatticeElement {
    hllset: HLLSet,
    key: String,
}

impl LatticeElement {
    /// Create a new LatticeElement from an HLLSet.
    ///
    /// The key is derived from the HLLSet's content hash.
    pub fn new(hllset: HLLSet) -> Self {
        let key = hllset.content_key();
        Self { hllset, key }
    }

    /// Create a LatticeElement from tokens (heterogeneous data).
    ///
    /// Tokens are inscribed into an HLLSet, then a content key is generated.
    pub fn from_tokens<'a, I, S>(tokens: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<[u8]>,
    {
        Self::new(HLLSet::from_tokens(tokens))
    }

    /// Create an empty LatticeElement (bottom of the lattice, ⊥).
    pub fn empty() -> Self {
        Self::new(HLLSet::new())
    }

    /// The content-addressable key (`h:<sha1>` or `c:<sha1>`).
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Direct access to the underlying HLLSet (for Rust consumers).
    pub fn hllset(&self) -> &HLLSet {
        &self.hllset
    }

    /// Consume and return the underlying HLLSet.
    pub fn into_hllset(self) -> HLLSet {
        self.hllset
    }

    /// Content hash (SHA-1 hex) of the serialized HLLSet.
    pub fn content_hash(&self) -> String {
        self.hllset.content_hash()
    }

    // ── Cardinality ──────────────────────────────────────────────────────

    /// Estimate cardinality (Horvitz-Thompson estimator).
    pub fn cardinality(&self) -> f64 {
        self.hllset.cardinality()
    }

    /// Number of bits set in the bitmap (popcount).
    pub fn popcount(&self) -> u64 {
        self.hllset.popcount()
    }

    /// Is this the empty lattice element?
    pub fn is_empty(&self) -> bool {
        self.hllset.is_empty()
    }

    // ── Lattice operations ────────────────────────────────────────────────

    /// Union (join): A ∪ B — bitwise OR.
    pub fn union(&self, other: &LatticeElement) -> LatticeElement {
        Self::new(self.hllset.union(&other.hllset))
    }

    /// Intersection (meet): A ∩ B — bitwise AND.
    pub fn intersection(&self, other: &LatticeElement) -> LatticeElement {
        Self::new(self.hllset.intersection(&other.hllset))
    }

    /// Difference: A \ B — bits in A but not in B.
    pub fn difference(&self, other: &LatticeElement) -> LatticeElement {
        Self::new(self.hllset.difference(&other.hllset))
    }

    /// Symmetric difference (XOR): A ⊕ B — bits in exactly one set.
    pub fn symmetric_difference(&self, other: &LatticeElement) -> LatticeElement {
        Self::new(self.hllset.symmetric_difference(&other.hllset))
    }

    /// Jaccard similarity: |A ∩ B| / |A ∪ B|.
    pub fn jaccard_similarity(&self, other: &LatticeElement) -> f64 {
        self.hllset.jaccard_similarity(&other.hllset)
    }

    // ── Subset relations ─────────────────────────────────────────────────

    /// Is `self` a subset of `other`?
    pub fn is_subset_of(&self, other: &LatticeElement) -> bool {
        self.hllset.is_subset_of(&other.hllset)
    }

    /// Is `self` a superset of `other`?
    pub fn is_superset_of(&self, other: &LatticeElement) -> bool {
        self.hllset.is_superset_of(&other.hllset)
    }

    // ── BSS morphisms ─────────────────────────────────────────────────────

    /// BSSτ: Bell State Similarity inclusion — |A ∩ B| / |B|.
    ///
    /// How much of B's content is also in A.
    pub fn bss_inclusion(&self, other: &LatticeElement) -> f64 {
        self.hllset.bss_inclusion(&other.hllset)
    }

    /// BSSρ: Bell State Similarity exclusion — |A \ B| / |B|.
    ///
    /// How much of A's content is NOT in B.
    pub fn bss_exclusion(&self, other: &LatticeElement) -> f64 {
        self.hllset.bss_exclusion(&other.hllset)
    }

    /// BSS morphism check: does A → B hold under thresholds?
    ///
    /// A → B iff  BSSτ(A, B) ≥ τ_min  AND  BSSρ(A, B) ≤ ρ_max.
    pub fn morph_to(&self, other: &LatticeElement, tau_min: f64, rho_max: f64) -> BSSResult {
        self.hllset.morph_to(&other.hllset, tau_min, rho_max)
    }

    // ── Serialization ─────────────────────────────────────────────────────

    /// Serialize to bytes (Roaring bitmap format).
    pub fn to_bytes(&self) -> Vec<u8> {
        self.hllset.to_bytes()
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        HLLSet::from_bytes(bytes).map(Self::new)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_assigns_key() {
        let elem = LatticeElement::from_tokens(&["hello", "world"]);
        assert!(elem.key().starts_with("h:"));
        assert_eq!(elem.key().len(), 42); // "h:" + 40 hex chars
    }

    #[test]
    fn test_empty_key() {
        let elem = LatticeElement::empty();
        // Even empty HLLSet has a content hash
        assert!(elem.key().starts_with("h:"));
    }

    #[test]
    fn test_union_commutative() {
        let a = LatticeElement::from_tokens(&["a", "b"]);
        let b = LatticeElement::from_tokens(&["b", "c"]);
        let aub = a.union(&b);
        let bua = b.union(&a);
        assert_eq!(aub.popcount(), bua.popcount());
    }

    #[test]
    fn test_intersection_distinct_keys() {
        let a = LatticeElement::from_tokens(&["x", "y"]);
        let b = LatticeElement::from_tokens(&["y", "z"]);
        let inter = a.intersection(&b);
        assert_ne!(a.key(), b.key());
        assert_ne!(inter.key(), a.key());
    }

    #[test]
    fn test_bss_inclusion_self() {
        let a = LatticeElement::from_tokens(&["a", "b", "c"]);
        let tau = a.bss_inclusion(&a);
        assert!((tau - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_bss_exclusion_self_is_zero() {
        let a = LatticeElement::from_tokens(&["a", "b"]);
        let rho = a.bss_exclusion(&a);
        assert!(rho < 0.01);
    }

    #[test]
    fn test_roundtrip_bytes() {
        let orig = LatticeElement::from_tokens(&["serialize", "test"]);
        let bytes = orig.to_bytes();
        let restored = LatticeElement::from_bytes(&bytes).unwrap();
        assert_eq!(orig.key(), restored.key());
        assert_eq!(orig.popcount(), restored.popcount());
    }

    #[test]
    fn test_is_empty_true() {
        let e = LatticeElement::empty();
        assert!(e.is_empty());
        assert_eq!(e.popcount(), 0);
    }

    #[test]
    fn test_is_empty_false() {
        let e = LatticeElement::from_tokens(&["something"]);
        assert!(!e.is_empty());
    }

    #[test]
    fn test_morph_to_self_holds() {
        let a = LatticeElement::from_tokens(&["a", "b", "c"]);
        let result = a.morph_to(&a, 0.8, 0.2);
        assert!(result.morphism_holds);
    }
}
