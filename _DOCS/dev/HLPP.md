# HLLSet Lattice Persistence Protocol (HLPP)

> **Status:** Proposal
> **Date:** June 30, 2026 (revised July 2, 2026)
>
> The lattice is the computation. IPFS is the persistence. This document
> defines the interface-agnostic protocol that connects them — first as
> an algebraic specification, then as concrete bindings.

---

## 1. Algebraic Specification

The algebraic spec is the source of truth. Every interface binding (Rust,
HTTP, Lua, Forth) must satisfy these laws. The spec filters out ambiguity
and inconsistency before a single line of implementation code is written.

### 1.1 Sorts (Types)

```text
Sort definitions:

  Bytes     = byte sequence (opaque)
  SHA1      = 40-char hex string
  UUID      = 32-char hex string (canonical, no dashes)
  Name      = UTF-8 string matching [a-zA-Z_][a-zA-Z0-9_]*
  Prefix    = { o, h, r, d, n, t, v }
  CID       = Prefix ":" SHA1                       -- content-addressed ID
  TmpID     = "u:" UUID                             -- user-assigned temporal
             | "system:" Name                       -- named global
  Key       = CID | TmpID
  HLLSet    = ⟨magic:8, version:4, M:4, B:4, regs:M×B⟩
  TFVec     = ⟨N:4, vals:N×f64⟩                     -- N = M × B = 32768
  Commit    = ⟨ts:u64, s:CID, h:CID, d:CID, r:CID, n:CID⟩
  Bool      = { true, false }
  Option[T] = Some(T) | None
```

### 1.2 Operations (Signatures)

```text
── CA Operations ──────────────────────────────────────────────

  PUT   : CID × Bytes → Unit
  GET   : CID → Option[Bytes]
  HAS   : CID → Bool
  LIST  : Prefix → List[CID]
  PIN   : CID → Unit
  UNPIN : CID → Unit
  GC    : Unit → List[CID]

── Temporal Operations ────────────────────────────────────────

  GET_TMP : TmpID → Option[Bytes]
  PUT_TMP : TmpID × Bytes → Unit
  CAS_TMP : TmpID × Bytes × Bytes → Bool
```

### 1.3 Laws (Invariants)

```text
── IICA (Immutable, Idempotent, Content-Addressed) ────────────

  LAW put-get:     ∀ cid, bytes ∶
                     PUT(cid, bytes); GET(cid) = Some(bytes)

  LAW idempotent:  ∀ cid, bytes ∶
                     PUT(cid, bytes); PUT(cid, bytes) = PUT(cid, bytes)

  LAW sha1-match:  ∀ cid = prefix:sha1, bytes ∶
                     PUT(cid, bytes) ⇒ sha1 = SHA1(bytes)

── Monotonicity ───────────────────────────────────────────────

  LAW pin-idempotent:   ∀ cid ∶ PIN(cid); PIN(cid) = PIN(cid)
  LAW unpin-idempotent: ∀ cid ∶ UNPIN(cid); UNPIN(cid) = UNPIN(cid)
  LAW gc-pin-safe:      ∀ cid ∶ PIN(cid) ⇒ cid ∉ GC()

── Temporal ───────────────────────────────────────────────────

  LAW tmp-put-get:  ∀ k, bytes ∶ PUT_TMP(k, bytes); GET_TMP(k) = Some(bytes)
  LAW cas-correct:  ∀ k, old, new ∶
                      GET_TMP(k) = Some(old) ⇒ CAS_TMP(k, old, new) = true
  LAW cas-reject:   ∀ k, old, cur, new ∶ cur ≠ old ⇒
                      CAS_TMP(k, old, new) = false

── Commit Chain ────────────────────────────────────────────────

  LAW commit-link:  ∀ commit c = ⟨ts, s, h, d, r, n⟩ stored at CID cid ∶
                      GET_HEAD() = Some(prev) ⇒ h = prev
```

### 1.4 Derived Operations

```text
Derived from the primitives above:

  PUT_HLL  : HLLSet → Unit
           = let bytes = serialize(HLLSet) in
             PUT("h:" + SHA1(bytes), bytes)

  GET_HLL  : CID → Option[HLLSet]
           = match GET(cid) { Some(b) ⇒ deserialize(b), None ⇒ None }

  PUT_TF   : TFVec → Unit
           = PUT_TMP("system:tf", serialize(TFVec))

  GET_TF   : Unit → Option[TFVec]
           = match GET_TMP("system:tf") { Some(b) ⇒ deserialize(b), None ⇒ None }

  PUT_HEAD : CID → Unit
           = PUT_TMP("system:head", ascii_bytes(cid))

  GET_HEAD : Unit → Option[CID]
           = match GET_TMP("system:head") { Some(b) ⇒ parse_cid(b), None ⇒ None }

  COMMIT   : HLLSet^5 → CID
           = let c = Commit(now(), s, h, d, r, n) in
             let cid = "t:" + SHA1(json(c)) in
             PUT(cid, json(c));
             PUT_HEAD(cid)
```

### 1.5 State Machine

```text
States:

  S = ⟨ store:    Map[CID → Bytes],
        temporal: Map[TmpID → Bytes],
        pinned:   Set[CID] ⟩

Initial state: S₀ = ⟨∅, ∅, ∅⟩

Transitions:

  PUT(cid, bytes):
    precondition: cid = prefix:sha1 ⇒ sha1 = SHA1(bytes)
    store' = store ⊕ {cid ↦ bytes}
    pinned' = pinned  -- unchanged

  GET(cid):
    return store(cid)

  HAS(cid):
    return cid ∈ dom(store)

  LIST(prefix):
    return [cid | cid ∈ dom(store), cid starts_with prefix]

  PIN(cid):
    pinned' = pinned ∪ {cid}
    store' = store

  UNPIN(cid):
    pinned' = pinned ∖ {cid}
    store' = store

  GC():
    removed = {cid | cid ∈ dom(store), cid ∉ pinned}
    store' = store ∖ removed
    return removed

  PUT_TMP(k, bytes):
    temporal' = temporal ⊕ {k ↦ bytes}

  GET_TMP(k):
    return temporal(k)

  CAS_TMP(k, old, new):
    if temporal(k) = Some(old):
      temporal' = temporal ⊕ {k ↦ new}
      return true
    return false
```

---

## 2. Object Namespaces

| Namespace | Identity | Replication | Meaning |
| --- | --- | --- | --- |
| `o:` | SHA1 (40 hex) | K≥3 (source) | Original HLLSet — from tokenizer, immutable |
| `h:` | SHA1 (40 hex) | K=1 (cache) | Standard HLLSet — any operation result |
| `r:` | SHA1 (40 hex) | K=1 | Retained HLLSet (R-link / intersection) |
| `d:` | SHA1 (40 hex) | K=1 | Departed HLLSet (difference) |
| `n:` | SHA1 (40 hex) | K=1 | New HLLSet (difference) |
| `t:` | SHA1 (40 hex) | K=2 | Commit object — CA by content |
| `v:` | SHA1 (40 hex) | none | View HLLSet — ephemeral, not persisted |
| `u:` | UUID (32 hex) | K=1 | User-assigned temporal identifier |
| `system:` | Fixed name | K=1 | Named global (tf, head, global_N) |

### System Keys

| Key | Type | Updated by | Description |
| --- | --- | --- | --- |
| `system:tf` | `TFVec` | Ingestion only | Global TF vector (32,768 × f64). Monotonic CRDT. |
| `system:tf_0` | `TFVec` | Second boundary | L0 TF snapshot — current second's frequencies |
| `system:tf_1` | `TFVec` | Minute boundary | L1 TF snapshot — minute-scale frequencies |
| `system:tf_2` | `TFVec` | Hour boundary | L2 TF snapshot — hour-scale frequencies |
| `system:tf_3` | `TFVec` | Day boundary | L3 TF snapshot — day-scale frequencies |
| `system:tf_4` | `TFVec` | Week boundary | L4 TF snapshot — week-scale frequencies |
| `system:tf_5` | `TFVec` | Month boundary | L5 TF snapshot — month-scale frequencies |
| `system:tf_6` | `TFVec` | Year boundary | L6 TF snapshot — year-scale frequencies |
| `system:head` | `CID` (string) | Commit | Latest commit CID — chain tip |
| `system:global_1` | `HLLSet` | Operation | System-wide aggregation #1 |
| `system:global_2` | `HLLSet` | Operation | System-wide aggregation #2 |
| `system:global_3` | `HLLSet` | Operation | System-wide aggregation #3 |
| `system:layer_0` | `HLLSet` | Ingestion (second) | L0 — current second, active S(t) |
| `system:layer_1` | `HLLSet` | Ingestion (minute) | L1 — completed seconds in current minute (∪ L0 over 59s) |
| `system:layer_2` | `HLLSet` | Ingestion (hour) | L2 — completed minutes in current hour (∪ L1 over 59min) |
| `system:layer_3` | `HLLSet` | Ingestion (day) | L3 — completed hours in current day (∪ L2 over 23h) |
| `system:layer_4` | `HLLSet` | Ingestion (week) | L4 — completed days in current week (∪ L3 over 6d) |
| `system:layer_5` | `HLLSet` | Ingestion (month) | L5 — completed weeks in current month (∪ L4 over ~3.5w) |
| `system:layer_6` | `HLLSet` | Ingestion (year) | L6 — completed months in current year (∪ L5 over 11mo) |

Layers are **mutually exclusive** time windows. Their union IS the complete
system state:

```text
H_system(t) = L0 ∪ L1 ∪ L2 ∪ L3 ∪ L4 ∪ L5 ∪ L6

No overlap. Each S(t) enters at L0, then percolates upward through
compression at time boundaries. At any moment, exactly one layer contains
any given time slice of the system's history.

---

## 3. Wire Formats (Canonical)

### 3.1 HLLSet (CA)

```text
Offset  Size    Field
0       8       Magic: "HLLSET\0\0"
8       4       Version: uint32 LE
12      4       M (register count): uint32 LE (= 1024)
16      4       B (bits per register): uint32 LE (= 32)
20      4096    Register array: 1024 × uint32 LE
4116    *       Optional metadata (currently empty, reserved)

Total: 4116 bytes fixed
CID: h:SHA1(bytes[0..4116])
```

### 3.2 TF Vector (Temporal)

```text
Offset  Size    Field
0       4       N (entry count): uint32 LE (= 32768)
4       262144  TF values: 32768 × float64 LE

Total: 262148 bytes fixed
Key: system:tf
```

### 3.3 Commit (CA)

```text
Compact JSON with canonical key ordering:
{"d":"<cid>","h":"<cid>","n":"<cid>","r":"<cid>","s":"<cid>","ts":<u64>}

CID: t:SHA1(json_bytes)
```

---

## 4. The Rank Separation Principle

**TF is stored. Rank is derived. They are not the same thing.**

Three distinct concepts, often confused:

| Concept | Level | What it measures | Example |
| --- | --- | --- | --- |
| Token TF | per-token | How often did this word appear? | `tf("hello") = 42` |
| Bit TF | per-position (32,768) | How much activity at this hash bucket? | `TF[1023][17] = sum of tf(t) for all t hashing here` |
| Rank | per-HLLSet (derived) | How important is this HLLSet right now? | `rank(H) = Σ TF[b] for b ∈ H` |

**Token TF → Bit TF.**  Each bit position aggregates multiple tokens.
A bit position $b$ receives tokens $\{t_1, \ldots, t_k\}$ with token-level
frequencies $\text{tf}(t_j)$.  The bit-level TF is:

$$\text{TF}[b] = f\big(\text{tf}(t_1), \ldots, \text{tf}(t_k)\big)$$

where $f$ is a pluggable reduction (sum, max, entropy --- see Section R in
the proposal).  Any monotonic $f$ preserves CRDT convergence.  The protocol
stores bit-level TF, not token-level TF.

```text
                 ┌──────────────────────────────┐
                 │      Shared TF Vector        │
                 │      Key: system:tf          │
                 │      32,768 × f64            │
                 │                              │
                 │  Updated ONLY by ingestion   │
                 │  Monotonic (increment only)  │
                 │  CRDT-convergent by IICA     │
                 │  Bit-level, not token-level  │
                 └──────────────┬───────────────┘
                                │
          ┌─────────────────────┼─────────────────────┐
          ▼                     ▼                     ▼
    aggregated rank       rank vector          normalized rank
    Σ TF[b] ∀ b∈H        {TF[b] ∀ b∈H}        ΣTF / |H|
    (scalar)              (vector)             (density)

    All computed locally from TF — never stored in protocol
```

| Action | Reads TF? | Writes TF? |
| -------- | :---------: | :----------: |
| `INSCRIBE` (tokenize) | No | **Yes** — increments bit-level TF via chosen $f$ |
| `UNION` / `INTERSECT` / `DIFF` | No | No — bitmask-only |
| Rank query (any form) | **Yes** — projects onto bit-level TF | No |
| Commit | No — stores DRN CIDs | No |

Why this separation:

1. **TF is monotonic CRDT.** Same tokens → same hashes → same increments. No consensus.
2. **Rank is context-dependent.** Different measures from the same TF. Protocol doesn't prescribe.
3. **TF writes are bounded.** Only ingestion touches TF. Queries are read-only.
4. **Rank is always fresh.** Monotonic TF → all HLLSets automatically reflect new data.

---

### 4.1 Rank Algebra — Five-Level Hierarchy

The Rank Separation Principle (Section 4) established that TF is stored and
rank is derived. This section formalizes **how** rank is derived through five
compositional levels. Each level is a function from its inputs to a rank value;
the levels compose deterministically.

```text
Level 5: compound HLLSet rank    L(max{R}) for union, M(min{R}) for intersection
         ↑
Level 4: HLLSet rank             K(degree in lattice graph)
         ↑
Level 3: register rank           H({bit-R[tz] | tz ∈ 0..31})
         ↑
Level 2: bit rank                G({token-R | all tokens hashing to (reg, tz)})
         ↑
Level 1: token rank              F(TF) — rank derived from token frequency
```

#### Level 1: Token Rank — F(TF)

A token $t$ has a Term Frequency $\text{TF}(t)$ — a raw count of occurrences.
Token rank is a transformation of TF:

```math
\text{token-R}(t) = F(\text{TF}(t))
```

$F$ is a design parameter. Candidates: identity ($F(x) = x$), logarithmic
($F(x) = \\log(1 + x)$), sigmoidal, or any monotonic function. The only
constraint: $F$ must be **monotonic** to preserve the CRDT convergence
guarantee — higher TF must not produce lower rank.

#### Level 2: Bit Rank — G({token-R})

A token $t$ hashes deterministically to a single bit position $(\\text{reg},
\\text{tz})$ via MurmurHash3. Multiple tokens may collide at the same position.
A bit's rank aggregates the ranks of all tokens that hash to it:

```math
\text{bit-R}(\text{reg}, \text{tz}) = G\big(\{\text{token-R}(t) \mid
\text{hash}(t) = (\text{reg}, \text{tz})\}\big)
```

$G$ is a pluggable aggregation. Candidates:

| G | Formula | Character |
| --- | --------- | ----------- |
| Max | $\max\{\text{token-R}\}$ | Dominant token controls the bit. Lossy. |
| Sum | $\sum\text{token-R}$ | All tokens contribute. Unbounded. |
| Weighted mean | $\frac{\sum w_i \cdot \text{token-R}_i}{\sum w_i}$ | Balanced, bounded. |

**Idempotency note.** HLLSet bit insertion is idempotent — setting the same
bit twice changes nothing. But rank aggregation at this level is NOT idempotent
unless $G$ is designed to be. If $G = \text{max}$, re-hashing the same token
adds nothing (the max is unchanged). If $G = \text{sum}$, re-hashing would
double-count — the storage layer must guard against this.

**tz independence.** The trailing-zero count $\text{tz} \in 0..31$ is an
address, not a weight. A token at tz=31 sets one bit, just like a token at
tz=0. The tz value carries no inherent importance signal — it is solely the
result of the hash function's output format. All 32 bit positions within a
register are equal citizens at this level.

#### Level 3: Register Rank — H({bit-R})

A register spans 32 bit positions (tz = 0..31). Each position has its own
bit-R from Level 2. The register rank aggregates across these 32 values:

```math
\text{reg-R}(r) = H\big(\{\text{bit-R}(r, \text{tz}) \mid \text{tz} \in
0..31\}\big)
```

$H$ must account for the fact that different tz positions within the same
register carry different structural meanings — they represent different
hash outputs, not different samples of the same quantity. Candidates:

| H | Formula | Character |
| --- | --------- | ----------- |
| Mean | $\frac{1}{32}\sum\text{bit-R}$ | Uniform weighting of all tz slots |
| Max-pool | $\max\{\text{bit-R}\}$ | Strongest bit dominates the register |
| Active-only mean | Mean over only set bits | Ignores empty tz slots |
| Population-weighted | Mean × (active bits / 32) | Registers with more activity get higher rank |

#### Level 4: HLLSet Rank — K(degree)

An HLLSet $H$ occupies a position in the lattice graph. Its structural rank
is a function of its graph-theoretic properties:

```math
\text{hllset-R}(H) = K\big(\text{degree}(H), \text{centrality}(H),
\ldots\big)
```

Where $\text{degree}(H)$ counts edges incident to $H$ in the lattice DAG
(operations that produced $H$ or used $H$ as input). Candidates:

| K | Formula | Character |
| --- | --------- | ----------- |
| Degree | $\text{degree}(H)$ | Simple count of lattice connections |
| Weighted degree | $\sum \text{popcount}(R_i)$ for each R-link $R_i$ incident to $H$ | Connections weighted by intersection strength |
| PageRank-like | Iterative importance propagation through the lattice DAG | Global structural importance |

This level captures **structural** importance — how central an HLLSet is to
the lattice topology — independent of its token-level statistics.

#### Level 5: Compound Rank — L(max) / M(min)

When HLLSets combine via lattice operations, ranks propagate:

```math
\text{rank}(A \cup B) = L\big(\max\{\text{rank}(A), \text{rank}(B)\},
\text{reg-R of union}\big)
```

```math
\text{rank}(A \cap B) = M\big(\min\{\text{rank}(A), \text{rank}(B)\},
\text{reg-R of intersection}\big)
```

The simplest forms: $L = \max$, $M = \min$ — the union inherits the
strongest component's rank; the intersection inherits the weakest. More
sophisticated forms could blend reg-R contributions from the compound's
own register-level aggregation.

#### Summary: The Five Functions

| Level | Function | Design space | FPGA-native choice | FPGA operations |
| ------- | ---------- | ------------- | ------------------- | ----------------- |
| 1 | $F$ | Identity, log, sigmoid | **Identity** ($F(x) = x$) or integer $\lfloor\log_2(x)\rfloor$ | Load, LZCNT |
| 2 | $G$ | Max, sum, weighted mean | **Max** or **Sum** | CMP, ADD |
| 3 | $H$ | Mean, max-pool, active-only | **Sum** or **Max-pool** | ADD (32 terms) or CMP tree |
| 4 | $K$ | Degree, weighted degree | **Degree** (popcount of adjacency row) | POPCOUNT |
| 5 | $L, M$ | Max/min, blended | **Max** / **Min** | CMP |

All five functions are **pluggable design parameters**. The protocol
specifies the framework; the application chooses the functions. The only
invariant across all choices: monotonicity with respect to TF — higher
token frequency must not decrease any derived rank.

**FPGA-native constraint.** For hardware implementation (Section 12 of the
Self-Reprogramming Architecture), all rank values must be fixed-width integers
derivable from bitmask AND/OR/XOR and popcount. Division is permitted only by
powers of two (right shift). The "FPGA-native choice" column identifies which
design-space option satisfies this constraint — notably rejecting mean (requires
division) in favor of sum or max-pool. Under these choices, the entire
five-level rank hierarchy compiles to AND, OR, XOR, POPCOUNT, ADD, SUB, and CMP
— no floating-point unit, no divider, no transcendental functions.

#### Dynamic Extension: Rank Derivatives and Fisher Matrix

The five-level hierarchy defines **static** rank — a snapshot of importance.
The dynamic behavior of ranks (velocity, acceleration, cross-layer coupling)
is analyzed in the Self-Reprogramming Architecture document, Section 17.1
("Rank Derivatives and Noether Steering"). Key results inherited by HLPP:

- **Rank-weighted Noether steering:** The conservation law $|\text{card}(N) -
\text{card}(D)| \to 0$ generalizes to rank flux — the net flow of rank into
and out of the system across D/R/N boundaries.
- **Fisher-like cross-layer matrix:** Co-occurrence of bit positions across
temporal layers $L_0..L_6$ forms a structural coupling matrix, enabling the
controller to distinguish isolated bit fluctuations from systemic phase
transitions.
- **Storage implication:** The Fisher matrix diagonal $F_{bb}$ (how many
layers contain each bit) is a natural priority signal for GC policies —
bits that appear across many temporal scales are structurally load-bearing
and should be pinned.

These dynamics operate *on top of* the static rank hierarchy defined here.
They do not change the protocol operations — they guide how the Noether
controller interprets the state that the protocol stores and retrieves.

### 4.2 Rank Depletion and the Observable Mask

The five-level hierarchy (Section 4.1) describes rank **construction** — how
token frequency propagates upward into bit, register, HLLSet, and compound
ranks. But rank also **depletes**. A token that fades from use creates a
symmetrical cascade downward through the same levels. This section formalizes
the depletion pathway and its structural consequence: re-masking the observable
HLLSet collection.

#### The Full Cycle: Rise and Fall

A token $t$ enters the system, its TF rises, $F(\\text{TF})$ increases, and the
rank chain propagates this increase upward. Later, $t$ fades — TF stagnates or
the token stops appearing in new scans. Token-R drops. The chain propagates the
drop upward with the same algebra but reversed sign:

```math
\text{token-R}(t)\!\downarrow \;\Rightarrow\;
\text{bit-R}(r, tz)\!\downarrow \;\Rightarrow\;
\text{reg-R}(r)\!\downarrow \;\Rightarrow\;
\text{hllset-R}(H)\!\downarrow
```

The five functions $(F, G, H, K, L, M)$ are the same for depletion as for
construction — they aggregate decreases the same way they aggregate increases.
The asymmetry is not in the algebra but in our attention: the TF vector is
monotonic (only increments), so token-R never decreases through TF alone.
**Depletion is driven by the baseline shifting around a static TF.** A
token's TF may remain constant while other tokens' TFs rise — the token's
*relative* rank drops not because it became less frequent, but because the
world around it became more active.

#### Observable Sample as a Mask

The complete collection $\mathcal{H}$ contains every HLLSet ever created.
The **observable sample** $\mathcal{O}(\theta) \subseteq \mathcal{H}$ is
the subset whose hllset-R exceeds a threshold $\theta$:

```math
\mathcal{O}(\theta) = \{H \in \mathcal{H} \mid \text{hllset-R}(H) > \theta\}
```

This is a **mask** — a bitmask over $\mathcal{H}$ where each bit indicates
whether an HLLSet is currently observable. Changing $\theta$ changes the mask;
changing ranks changes which HLLSets pass the mask.

**Re-masking.** When ranks reshuffle (new tokens rise, old tokens relatively
fade), the mask $\mathcal{O}(\theta)$ changes. HLLSets that were above
threshold may fall below it; HLLSets that were below may rise above. This is
not deletion — every HLLSet remains in $\mathcal{H}$, content-addressed and
retrievable. The mask controls **attention**, not existence.

#### Structural Consequence: Sub-Lattice Degree Change

The observable sample $\mathcal{O}(\theta)$ forms a **sub-lattice** of the
complete lattice. In the complete lattice, an HLLSet $H$ has degree
$\text{deg}(H)$ — the number of incident edges. In the sub-lattice, its
degree $\text{deg}_{\mathcal{O}}(H)$ counts only edges to other *observable*
HLLSets. When the mask changes:

- An HLLSet $H$ remains observable but its neighbor $H'$ drops out:
  $\text{deg}_{\mathcal{O}}(H)$ decreases by 1.
- An HLLSet $H$ was below threshold but a neighbor $H'$ rises above:
  $\text{deg}_{\mathcal{O}}(H)$ does not change (both must be observable
  for the edge to count).
- Both $H$ and $H'$ cross the threshold together: degree changes propagate
  through the sub-lattice, potentially triggering further re-masking as
  hllset-R depends on $K ( \text{degree} )$.

**This is the feedback loop that rearranges HLLSets.** Token-R changes at
Level 1 propagate upward to hllset-R at Level 4. The mask $\mathcal{O}(\theta)$
shifts. Sub-lattice degrees change. $K(\text{degree})$ produces new hllset-R
values. The mask shifts again. The loop continues until ranks stabilize — which
they do when the environment stabilizes, per the Noether convergence guarantee.

#### Symmetry Restored (With a Caveat)

| Direction | Trigger | Propagation | Structural effect |
| ----------- | --------- | ------------- | ------------------- |
| **Rise** | New tokens appear, TF increases | token-R↑ → bit-R↑ → reg-R↑ → hllset-R↑ | HLLSets enter $\mathcal{O}(\theta)$, sub-lattice expands |
| **Fall** | Baseline shifts, relative rank drops | token-R↓ → bit-R↓ → reg-R↓ → hllset-R↓ | HLLSets exit $\mathcal{O}(\theta)$, freeing space for new entrants |
| **Equilibrium** | | Rise ≈ Fall | $\mathcal{O}(\theta)$ composition churns but size stabilizes |

The five-level hierarchy is **symmetric under rank reversal** at the algebraic
level — the same functions $(F, G, H, K, L, M)$ that construct rank also
deconstruct it. What Section 4 called the "Rank Separation Principle" — TF is
stored, rank is derived — applies equally to depletion: TF does not decrease,
but the derived rank does, because the baseline continuously shifts.

#### Content vs. Structure: The Phase Boundary at Level 3→4

However, the symmetry claim must be qualified. The five levels are not a single
homogeneous chain. There is a **phase boundary** between Level 3 (register rank)
and Level 4 (HLLSet degree):

| Levels | Metric family | What is measured | Raw material |
| -------- | -------------- | ----------------- | -------------- |
| 1→2→3 | **Content-based** | Internal composition of the HLLSet | Token frequencies, hash positions, bit patterns |
| 4 | **Structure-based** | Position in the lattice graph | Similarity to other HLLSets (union, intersection edges) |
| 5 | **Propagation** | How ranks combine under lattice operations | Component ranks |

Level 3 (reg-R) is derived from what is *inside* the HLLSet — its tokens, their
frequencies, their hash positions. Level 4 (hllset-R = $K(\\text{degree})$) is
derived from the HLLSet's *relationships* — how many other HLLSets it has been
combined with, how central it is in the lattice DAG.

These two metric families are **correlated but not reducible to each other.**
The analogy is mass and gravity: a massive object tends to exert strong
gravitational pull, but mass is an intrinsic property while gravity is a
relational one. An HLLSet built from high-frequency tokens (high reg-R)
tends to participate in many lattice operations (high degree), but:

- A content-rich HLLSet may be isolated (low degree) if no operations have
  connected it to others — high mass, low gravity.
- A content-poor HLLSet may be highly central (high degree) if it happens to
  be the intersection of many other HLLSets — low mass, high gravity.
- Degree depends on similarity at the HLLSet level (which bit positions
  overlap), not directly on token frequencies. Two HLLSets with different
  token compositions can still have high overlap if their hashes collide
  favorably.

**Current implementation status.** The chain $F \to G \to H \to K$ treats
the content→structure transition as direct propagation, without modeling the
correlation gap. This is a deliberate simplification for the initial
implementation. The feedback loop still functions — reg-R influences which
HLLSets enter $\mathcal{O}(\theta)$, which affects degree, which feeds back
via $K(\text{degree})$ — but the coupling between content metrics and
structural metrics is implicit, not explicit.

**Future resolution.** A proper treatment would model the content→structure
relationship as a bipartite coupling rather than a linear chain:

```math
\text{hllset-R} = K\big(\text{degree}(H), \; \text{reg-R}(H)\big)
```

where $K$ takes *both* the structural position *and* the internal content rank
as independent inputs, rather than deriving degree from reg-R. This would allow
the system to distinguish the high-mass-low-gravity case from the
low-mass-high-gravity case, and to weight them appropriately per application.

For now, the correlation is handled implicitly by the feedback loop itself:
HLLSets with high reg-R tend to be operated on more often, which increases
their degree, which increases their hllset-R. The loop naturally couples the
two metrics — just not in a formally separated way. Documenting this gap
acknowledges where future refinement should occur.

The mask $\mathcal{O}(\theta)$ remains the interface between static storage
and dynamic attention, regardless of how the content↔structure coupling is
implemented.

---

## 5. Interface Bindings

### 5.1 Rust (Native)

```rust
pub trait HlppStorage {
    // CA
    fn put(&self, cid: &str, bytes: &[u8]) -> Result<(), HlppError>;
    fn get(&self, cid: &str) -> Result<Option<Vec<u8>>, HlppError>;
    fn has(&self, cid: &str) -> Result<bool, HlppError>;
    fn list(&self, prefix: &str) -> Result<Vec<String>, HlppError>;
    fn pin(&self, cid: &str) -> Result<(), HlppError>;
    fn unpin(&self, cid: &str) -> Result<(), HlppError>;
    fn gc(&self) -> Result<Vec<String>, HlppError>;
    // Temporal
    fn get_tmp(&self, key: &str) -> Result<Option<Vec<u8>>, HlppError>;
    fn put_tmp(&self, key: &str, bytes: &[u8]) -> Result<(), HlppError>;
    fn cas_tmp(&self, key: &str, old: &[u8], new: &[u8]) -> Result<bool, HlppError>;
}
```

### 5.2 HTTP

```text
GET    /api/v1/hllset/<cid>         → 200 + binary | 404
PUT    /api/v1/hllset/<cid>         → 201 | 409 (mismatch)
HEAD   /api/v1/hllset/<cid>         → 200 | 404
GET    /api/v1/hllset?prefix=h:     → 200 + [cid, ...]
POST   /api/v1/hllset/<cid>/pin     → 200
DELETE /api/v1/hllset/<cid>/pin     → 200
POST   /api/v1/hllset/gc            → 200 + [removed, ...]

GET    /api/v1/temporal/<key>       → 200 + binary | 404
PUT    /api/v1/temporal/<key>       → 200
POST   /api/v1/temporal/<key>/cas   → 200 (true) | 409 (false)
```

### 5.3 Lua

```lua
-- CA (implemented)
hllset.store(elem)       -- PUT
hllset.load(key)         -- GET
hllset.exists(key)       -- HAS
hllset.list(prefix)      -- LIST
hllset.pin(key)          -- PIN
hllset.unpin(key)        -- UNPIN
hllset.gc()              -- GC

-- Temporal (to implement)
hllset.get_tmp(key)      -- GET_TMP
hllset.put_tmp(key, val) -- PUT_TMP
hllset.cas_tmp(k, o, n)  -- CAS_TMP
```

### 5.4 Forth

```forth
\ CA operations
: STORE  ( h -- )        \ HLLSET>BYTES SWAP PUT
: LOAD   ( cid -- h|nil) \ DUP GET ?DUP IF BYTES>HLLSET THEN
: LIST   ( prefix -- cids )
: PIN    ( cid -- )
: UNPIN  ( cid -- )
: GC     ( -- removed )

\ Temporal operations
: PUT-TF  ( tf -- )      \ TF>BYTES "system:tf" PUT_TMP
: GET-TF  ( -- tf )      \ "system:tf" GET_TMP BYTES>TF
: COMMIT  ( S H D R N -- cid )
: GET-HEAD ( -- cid )    \ "system:head" GET_TMP
```

---

## 6. The Forth AST as Unifier

```text
                    ┌────────────────────────┐
                    │    Forth Source (write)│
                    │  "a" "b" 2 INSCRIBE    │
                    │  a b INTERSECT STORE   │
                    └──────────┬─────────────┘
                               │
                               ▼
                    ┌────────────────────────┐
                    │   Forth AST (canonical)│
                    └──────────┬─────────────┘
                               │
          ┌────────────────────┼──────────────────┐
          ▼                    ▼                  ▼
   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
   │  Lower: Lua  │    │ Lower: Rust  │    │ Lower: HW    │
   │  (software)  │    │ (software)   │    │ (FPGA)       │
   └──────┬───────┘    └──────┬───────┘    └──────┬───────┘
          │                   │                   │
          └───────────────────┼───────────────────┘
                              │
                              ▼
                    ┌─────────────────────┐
                    │   HLPP (IPFS)       │
                    │   Same CIDs, same   │
                    │   bytes, same       │
                    │   lattice state     │
                    └─────────────────────┘
```

---

## 7. Implementation Plan

| Phase | Status | Tasks |
| --- | --- | --- |
| **1. Protocol Formalization** | Partial | ✅ CA operations in Storage trait. ✅ Lua CA bindings. ⬜ Temporal operations (`get_tmp`, `put_tmp`, `cas_tmp`). ⬜ `TFVec` wire format in `hllset-core`. ⬜ `Commit` struct in `hllset-core`. ⬜ Lua temporal bindings. |
| **2. Forth Frontend** | ✅ | Parser + AST + Lua lowerer + CLI `--forth` flag |
| **3. Unification** | Deferred | AST as canonical test format. Multi-backend execution. |

---

## 8. Relationship to Existing Code

| Component | Change |
| --- | --- |
| `hllset-core` | Add `TFVec` wire format, `Commit` struct |
| `hllset-storage` | Add `get_tmp`, `put_tmp`, `cas_tmp` to trait |
| `hllset-dsl` (Lua) | Add temporal Lua bindings |
| `hllset-cli` | No change needed |
| `hllset-forth` | Add temporal Forth words |

---

## 9. IPLD Integration

HLPP objects are IPLD nodes. The Commit is a dag-json document. Every CID
field is an IPLD Link — the lattice is a navigable DAG in IPFS.

### 9.1 dag-json Codec

Commit objects use the dag-json multicodec (`0x0129`). CID references are
marked as IPLD Links using the `/` prefix:

```json
{"/": "t:4b38ac2be97210956c..."}
```

A full Commit in dag-json:

```json
{
  "ts": 1719876543210,
  "s":  {"/": "o:a1e7647eb2c601256c..."},
  "h":  {"/": "t:4b38ac2be97210956c..."},
  "d":  {"/": "d:9d8ac7f6d54ba51164..."},
  "r":  {"/": "r:4b38ac2be97210956c..."},
  "n":  {"/": "n:c15d62bb4a11190381..."}
}
```

### 9.2 Lattice Traversal

The commit chain is a DAG. IPFS tooling traverses it natively:

```bash
# Walk backward through commit history
ipfs dag get t:commit_N/h    # → t:commit_N-1
ipfs dag get t:commit_N/h/h  # → t:commit_N-2

# Access HLLSet data from a commit
ipfs dag get t:commit_N/s    # → o:session_hllset
ipfs block get o:session_hllset | hllset-inspect

# Follow R-links (retained intersections)
ipfs dag get t:commit_N/r    # → r:retained_intersection

# Get the full D/R/N decomposition
ipfs dag get t:commit_N      # → {ts, s, h, d, r, n}
```

### 9.3 Why IPLD Matters

1. **No custom traversal code.** IPFS/IPLD libraries resolve `{"/": "<cid>"}`
   links automatically. Walking the lattice history is `dag get` chaining.

2. **Standard tooling.** `ipfs dag resolve`, `ipfs dag export`, graph
   visualizers, IPLD explorers — all work on our Commit DAG without
   modification.

3. **CBOR option.** dag-json is the human-readable codec. dag-cbor (`0x71`)
   is the compact binary codec. The same Commit structure works with both.
   HLPP recommends dag-json for commits (small, human-readable) and raw
   bytes for HLLSets (fixed-size binary).

4. **Schema validation.** IPLD Schemas can validate Commit structure:

```ipldsch
type Commit struct {
  ts Int
  s  Link
  h  Link
  d  Link
  r  Link
  n  Link
}
```

Any IPLD Schema validator can verify Commit objects before they enter the DAG.
