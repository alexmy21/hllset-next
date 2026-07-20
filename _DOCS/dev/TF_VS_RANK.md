# TF vs Rank: Two Signals, Two Purposes

> **Session:** July 19, 2026
> **Status:** Architectural principle
> **Applies to:** All HLLSet Algebra systems
> **References:** `SELF_REPROGRAMMING_ARCHITECTURE.md` (Section 5, 12, 17.1), `FORECASTING.md`

---

## 0. The Problem

TF (Term Frequency) and Rank are often conflated in HLLSet discussions.
They are derived from the same underlying data — token ingestion counts —
but they serve **fundamentally different purposes** and operate under
**fundamentally different constraints**.

```text
TF   → continuous, monotonic, same-token-base
Rank → ordinal, structural, cross-domain
```

Mixing them produces reasoning errors. Keeping them separate enables
domain-universal operation.

---

## 1. TF: Term Frequency

### 1.1 Definition

TF is a **continuous, monotonic scalar** that measures how much activity
a given entity has accumulated. It is a CRDT — two nodes ingesting the
same tokens converge to identical TF values without coordination.

### 1.2 Levels of TF

| Level | Entity | Meaning | Range |
| --- | --- | --- | --- |
| TF₀ (token) | A specific token string | How many times did this token appear? | [0, ∞) |
| TF₁ (bit) | A bit position $(r, tz)$ | How much total activity at this hash bucket? | [0, ∞) |
| TF₂ (register) | A register (aggregation of 32 bits) | How much activity in this register? | [0, ∞) |
| TF₃ (HLLSet, internal) | An HLLSet within its own lattice | In-degree: how many R-links point TO this HLLSet? | [0, ∞) |
| TF₄ (HLLSet, external) | An HLLSet across lattices | Cross-lattice in/out-degree: how many cross-lattice R-links involve this HLLSet? | [0, ∞) |

TF₁ (bit-level) is the most commonly referenced. It's computed by reducing
token-level TFs through a pluggable function $f$ (sum, max, entropy proxy —
see Section 4 of the proposal):

```math
\text{TF}[b] = f\big(\text{tf}(t_1), \ldots, \text{tf}(t_k)\big)
```

where $t_1, \ldots, t_k$ are all tokens that hash to bit position $b$.

### 1.3 What TF Is Used For

| Operation | Formula | Purpose |
| --- | --- | --- |
| Distance | $\text{KL}(\text{TF}_A \| \text{TF}_B)$ | How different are two TF distributions? |
| Covariance | $\text{Cov}(\text{TF}[b], \text{TF}[b'])$ | Do bits $b$ and $b'$ move together? |
| First derivative | $\Delta\text{TF} = \text{TF}(t) - \text{TF}(t-1)$ | What changed since last scan? |
| Second derivative | $\Delta^2\text{TF} = \Delta\text{TF}(t) - \Delta\text{TF}(t-1)$ | Is change accelerating? |
| Fisher matrix | $F_{bb'} = \sum_i B^{(i)}_b \cdot B^{(i)}_{b'}$ | Which bits co-occur across temporal layers? |
| Holographic projection | $H_{\text{world}} \odot \text{TF}_{\text{stack}}[t]$ | Recover past state from TF lens |
| Materialization | Token selection via TF-weighted LUT | Which token at this bit position is most active? |
| Forecasting | $\text{TF}_{\text{forecast}} = \text{propagate}(\text{TF}_{\text{history}})$ | Pre-position the interpreter |

### 1.4 The Critical Constraint

**TF requires the same token base.** The bit position $b = (r, tz)$ is
determined by the hash function. If two HLLSets were produced by different
hash functions (or the same hash function applied to different token
vocabularies), their bit positions encode different things. TF[314] in a
Chinese-derived HLLSet means "the hash bucket for 山"; TF[314] in an
English-derived HLLSet means "the hash bucket for purchase." Comparing
them is meaningless — the numbers are in different spaces.

---

## 2. Rank: Ordinal Position

### 2.1 Definition

Rank is an **ordinal position** derived from TF by sorting. For bit-level:

$$\text{rank}(b) = \text{position of } b \text{ in the sorted list of all active bits by TF}$$

Bit with highest TF → rank 0. Bit with second-highest TF → rank 1. And so
on up to 32,767.

For HLLSet-level (within a lattice or sub-lattice):

$$\text{rank}(H) = \text{position of } H \text{ in the ordered list of HLLSets by their aggregate TF}$$

### 2.2 What Rank Measures

Rank measures **structural position**, not absolute intensity. It answers:
"how important is this entity *relative to others* in the same system?"

- A bit with TF = 0.001 and rank = 3 is the 4th most active bit in its HLLSet.
- A bit with TF = 0.92 and rank = 3 is ALSO the 4th most active bit in its HLLSet.

Same rank, different TF. The rank captures the structural role — "this is
a top-tier position" — without committing to an absolute magnitude.

### 2.3 What Rank Is Used For

| Operation | Formula | Purpose |
| --- | --- | --- |
| Rank correlation | $\rho(\text{rank}_A, \text{rank}_B)$ (Spearman) | Do two HLLSets have similar structural profiles? |
| Rank entropy | $H(\text{rank})$ | Is the HLLSet concentrated (few hot bits) or dispersed? |
| Structural similarity | $\tau(\text{rank}_A, \text{rank}_B)$ (Kendall) | How similar are the importance orderings? |
| Cross-domain matching | $\text{rank\_corr}(H_{\text{CN}}, H_{\text{EN}})$ | Do the Chinese and English HLLSets have the same shape? |
| Rank reshuffling | Section 5 of architecture doc | Learning = changing rank ordering in the Forth dictionary |

### 2.4 Why Rank Works Across Domains

Rank is **hash-function-agnostic**. It doesn't care what bit position 314
means in any given domain. It only cares that it's the 3rd most active
position.

Two HLLSets built from completely different token vocabularies, different
hash functions, different domains:

```text
H_CAAL (Chinese):  TF[山_pos] = 0.92 (rank 0), TF[水_pos] = 0.87 (rank 1), ...
H_EN  (English):   TF[purchase_pos] = 0.03 (rank 0), TF[mountain_pos] = 0.028 (rank 1), ...
```

TF comparison: meaningless. TF[山_pos] vs TF[purchase_pos] — different
semantic spaces.

Rank comparison: meaningful. Both HLLSets have a strongly dominant top bit
(rank 0 >> rank 1 gap), then a cluster of mid-ranked bits. The structural
profile — "one dominant peak, long tail" — is similar even though the
specific tokens are unrelated.

This is how the hash bridge achieves domain universality: different hash
functions produce different bit-position assignments, but the **rank
structure** of a well-formed HLLSet — the shape of its importance
distribution — is invariant under re-hashing.

---

## 3. The Separation Principle

```text
┌─────────────────────────────────────────────────────────────┐
│                     SAME TOKEN BASE                         │
│                                                             │
│  Use TF for:                                                │
│    • Distance metrics (KL divergence)                       │
│    • Derivatives (ΔTF, Δ²TF)                                │
│    • Fisher matrix (co-occurrence coupling)                 │
│    • Holographic projection (time lens)                     │
│    • Materialization (LUT disambiguation)                   │
│    • Forecasting (constraint propagation)                   │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                     DIFFERENT TOKEN BASES                   │
│                                                             │
│  Use Rank for:                                              │
│    • Structural similarity (rank correlation)               │
│    • Cross-domain matching (same shape, different content)  │
│    • Hash bridge comparison (CAAL ↔ English)                │
│    • Lattice isomorphism detection                          │
│    • Domain-universal HLLSet classification                 │
│                                                             │
│  TF is INAPPLICABLE across different token bases.           │
│  Rank is MEANINGFUL across different token bases.           │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Relationship Between TF and Rank

### 4.1 Derivation

Rank is always derived from TF, never independent:

```math
\text{rank} = \text{argsort}(\text{TF}, \text{descending})
```

But the reverse is not true: you cannot recover TF from rank. Rank discards
magnitude — it preserves only ordering. This is a feature, not a bug. It's
what makes rank portable across domains while TF stays anchored to a specific
token base.

### 4.2 When They Diverge

| Scenario | TF says | Rank says | Which to trust |
| --- | --- | --- | --- |
| Same domain, same vocabulary | Both agree | Both agree | TF (more information) |
| Same domain, vocabulary shift | Magnitudes changed | Ordering may be stable | Rank if comparing structure; TF if measuring intensity |
| Different hash functions | Incomparable | Comparable | Rank |
| Cross-domain (CAAL → English) | Incomparable | Comparable | Rank |

### 4.3 The Hash Bridge Revisited

The hash bridge $h_B: \text{Domain}_B \rightarrow \text{HLLSet}$ creates
a new token base. Within that base, TF is meaningful. Across bases, only
Rank is meaningful.

This means the CAAL + I Ching architecture has two comparison modes:

```text
Within CAAL:  BSS, TF distance, Fisher matrix — all work
              (same hash function, same token base)

Across bridges: Rank correlation, structural similarity
                (different hash functions, different token bases)
```

The I Ching consultation uses within-CAAL comparison (the scene HLLSet
and the hexagram HLLSets share the CAAL hash function). The hash bridge
from an English domain uses across-bridge comparison — the English
HLLSet and the CAAL HLLSet have different hash functions, so the bridge
must use rank-based matching to find the CAAL HLLSet with the most
similar structural profile.

---

## 5. Implementation Notes

### 5.1 Rank Computation

```text
Bit rank (single HLLSet):
  active_bits = {b : TF[b] > 0}
  sorted_bits = sort(active_bits, by=TF, descending)
  rank[b] = position of b in sorted_bits

HLLSet rank (within lattice):
  hllsets = {H_1, ..., H_n}
  aggregate_TF(H) = sum(TF[b] for b in H)
  sorted_hllsets = sort(hllsets, by=aggregate_TF, descending)
  rank(H) = position of H in sorted_hllsets
```

### 5.2 Rank Correlation (Spearman)

```math
\rho = 1 - \frac{6 \sum d_i^2}{n(n^2 - 1)}
```

where $d_i = \text{rank}_A(b_i) - \text{rank}_B(b_i)$ for each bit $b_i$
active in both HLLSets. Only bits active in BOTH HLLSets contribute to the
correlation — bits active in only one are structural divergence, not
comparable positions.

### 5.3 FPGA Cost

| Operation | TF approach | Rank approach |
| --- | --- | --- |
| Comparison | Float KL divergence (division) | Integer sort + correlation (cmp, add, mul) |
| Storage | f64 per active bit | u16 per active bit (0..32767) |
| Update | Monotonic add (CRDT) | Re-sort on TF change |
| FPGA-native | No (requires float) | Yes (integer arithmetic) |

Rank operations are FPGA-native. TF operations (KL divergence) require
floating-point. This reinforces the architectural preference established
in Section 12 of the architecture doc: BSS was replaced by R-links
(integer popcount) for the same reason.

---

## 6. Summary

| | TF | Rank |
| --- | --- | --- |
| Type | Continuous scalar | Ordinal position |
| Monotonic? | Yes (CRDT) | Re-sorted on every TF change |
| Domain portability | No (bound to hash function) | Yes (invariant under re-hashing) |
| Information preserved | Magnitude + ordering | Ordering only |
| Used for | Distance, derivatives, Fisher, materialization, forecasting | Structural similarity, cross-domain matching, classification |
| FPGA cost | Float operations | Integer operations |

**The rule:** TF when you're working within a single token base. Rank when
you're comparing across token bases. The hash bridge translates domains;
Rank translates structure.
