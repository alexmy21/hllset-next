# The Universal Bridge: Domain Universality via Re-Representation

> **Session:** July 19, 2026
> **Status:** Definitive architecture
> **Supersedes:** `CAAL_ICHING_ARCHITECTURE.md`, `CAAL_ICHING_DEVELOPMENT.md` (specifics merged here)
> **Depends on:** `TF_VS_RANK.md`, `FORECASTING.md`, `DIMENSIONAL_NESTING.md`

---

## 0. The Discovery

A single algorithm maps any HLLSet into any HLLSet lattice, regardless of
domain, vocabulary, or hash function. The algorithm is not a translation.
It is a structural projection through 3-gram space.

```text
One algorithm. Two passes. Domain-universal.
```

---

## 1. Two-Pass Ingestion

Every external input enters the HLLSet Algebra twice.

### Pass 1: Representation

```text
domain_input → murmurhash3 → H_src (lives in source domain's bit space)
```

$H_{\text{src}}$ is the source-domain HLLSet. Its materialization through the
source domain's Token-LUT approximates the original input.

### Pass 2: Re-Representation

```text
H_src's active bit positions → formatted as tokens → murmurhash3 → H_bridge
                                 ("reg:314:tz:17", "reg:8912:tz:3", ...)
```

$H_{\text{bridge}}$ lives in the **target domain's bit space**. It is a
citizen of the target lattice. BSS, R-links, union, intersection with all
target-domain HLLSets work directly — no cross-domain translation layer.

### Why Two Passes

| | Pass 1 ($H_{\text{src}}$) | Pass 2 ($H_{\text{bridge}}$) |
| --- | --- | --- |
| Bit space | Source domain | Target domain |
| Materialize via source LUT | ✓ (recovers original) | ✗ (different token base) |
| Materialize via target LUT | ✗ (different token base) | ✓ (structural interpretation) |
| BSS with target HLLSets | ~0 (entanglement: different seeds/spaces) | ✓ (same bit space) |
| BSS with source HLLSets | ✓ | ~0 |

$H_{\text{bridge}}$ carries the source's **structure** into the target
lattice while severing the link to source tokens. It is a structural
fingerprint, not a translation.

### Same Seed, Different Input

No seed management is required. The second pass hashes bit-position labels
(`"reg:314:tz:17"`) instead of original tokens (`"mountain"`). The hash
function is identical; the input strings are different. The composition
`murmurhash3("reg:" + murmurhash3(token))` is a deterministic hash function
with the same IICA properties as the original. BSS($H_{\text{src}}$,
$H_{\text{bridge}}$) ≈ 0 automatically.

---

## 2. 3-Gram Structural Fingerprinting

A 3-gram HLLSet is built from all consecutive token triples in the source
text (with boundary padding). It encodes both adjacency patterns AND
vocabulary — each 3-gram with token-sequence padding covers every 1-gram
in the vocabulary.

The 3-gram HLLSet is the structural invariant. Two texts in different
languages with similar discourse structure produce 3-gram HLLSets with
correlated rank distributions, even though their vocabularies are disjoint.

```text
English:  "the cat sat" → {("the","cat","sat"), ("_START_","the","cat"), ("cat","sat","_END_")}
Chinese:  "猫坐在"      → {("猫","坐","在"), ("_START_","猫","坐"), ("坐","在","_END_")}

Different tokens. Same structural role (subject-verb-preposition).
3-gram rank distribution captures this.
```

---

## 3. The Algorithm

```text
algorithm bridge(source_HLLSet, target_lattice):
    # Pass 2: re-represent source into target bit space
    H_bridge = re_represent(source_HLLSet)
    # H_bridge is now a target-lattice citizen

    # Extract 3-gram structural fingerprints
    S_3gram = extract_3gram(source_HLLSet)
    B_3gram = extract_3gram(H_bridge)

    # Rank-correlate against all HLLSets in target lattice
    candidates = []
    for H_target in target_lattice:
        T_3gram = extract_3gram(H_target)
        ρ = spearman_rank_correlation(B_3gram, T_3gram)
        if ρ > threshold:
            candidates.append((H_target, ρ))

    # Select minimal cover with maximal rank correlation
    cover = select_cover(candidates, S_3gram)

    return {
        bridge: H_bridge,
        cover: cover,
        top_match: candidates[0]
    }
```

---

## 4. Resolution: Domain LUTs

Each domain owns its own Token-LUT. Materialization always goes through
the LUT of the HLLSet's native domain.

```text
Source HLLSet → source_LUT → source tokens (approximates original)
Bridge HLLSet → target_LUT → target tokens (structural interpretation)
```

Domain LUTs are independent. The source LUT maps English vocabulary. The
target LUT maps Chinese characters. They don't interfere. They don't need
to agree. An HLLSet materialized through the wrong LUT produces meaningless
output — but the architecture never does that, because each HLLSet carries
its domain identity through the bit-space it inhabits.

---

## 5. Implementation

### 5.1 Universal Algorithm Crate

```text
crates/hllset-bridge/
├── src/
│   ├── lib.rs              ← public API: bridge(source, target_lattice) → result
│   ├── re_represent.rs     ← Pass 2: bit positions → tokens → HLLSet
│   ├── ngram.rs            ← 3-gram extraction from any HLLSet
│   ├── rank.rs             ← Spearman ρ, Kendall τ, rank vectors
│   ├── cover.rs            ← Minimal cover selection with rank scoring
│   └── lut.rs              ← Domain LUT: token → hash position → TF
└── tests/
    ├── same_domain.rs      ← Chinese → CAAL (same hash, same vocab)
    └── cross_domain.rs     ← English → CAAL (different hash, different vocab)
```

### 5.2 Application Crates (Config + Data)

```text
crates/hllset-caal/
├── data/
│   └── vocabulary.json     ← 80K Chinese characters
└── src/
    └── lib.rs              ← ~50 lines: load vocab, init LUT, expose tokenizer

crates/hllset-iching/
├── data/
│   └── iching_corpus/      ← 64 hexagrams × commentary texts
└── src/
    └── lib.rs              ← ~100 lines: ingest corpus → HLLSets,
                            ←   build hexagram R-link matrix,
                            ←   consultation = argmax BSS(H_bridge, hex_i)
```

### 5.3 Application Example: CAAL + I Ching

```rust
use hllset_bridge::bridge;
use hllset_caal::CaalLattice;
use hllset_iching::IChingLattice;

// Phase 1: Ingest I Ching corpus once
let caal = CaalLattice::new("data/vocabulary.json");
let iching = IChingLattice::from_corpus("data/iching_corpus/", &caal);

// Phase 2: Any source domain enters through the bridge
let source_hllset = tokenize_english("vehicle at intersection, pedestrian left");
let result = bridge(source_hllset, &caal);

// Phase 3: I Ching consultation on the bridge HLLSet
let hexagram = iching.consult(&result.bridge);
let next_hex = iching.navigate(hexagram, &result.bridge);

// Phase 4: Materialize guidance
let guidance = caal.materialize(&next_hex.commentary);
// → Chinese instruction for VLA, or translate to host language
```

---

## 6. Why This Architecture

1. **The algorithm is domain-agnostic.** It doesn't know what Chinese or
   English or sensor data is. It knows HLLSets, 3-grams, and rank correlation.

2. **The bridge HLLSet is a citizen.** Once re-represented, the source's
   structure lives natively in the target lattice. No ongoing translation.
   No cross-domain BSS.

3. **Resolution is LUT-mediated.** Each domain has its own LUT. The same
   HLLSet materialized through different LUTs produces different tokens.
   This is correct — each domain interprets the structure in its own terms.

4. **Applications are configuration.** CAAL = Chinese tokenizer + LUT.
   I Ching = pre-computed corpus HLLSets + R-link matrix. Together they're
   ~200 lines of setup. The bridge algorithm is ~500 lines of Rust.

5. **No new algebra.** Re-representation is murmurhash3 applied twice.
   3-gram extraction is tokenization at a different granularity. Rank
   correlation is integer sort + Spearman formula. LUT materialization
   already exists. Everything is composition of existing operations.

---

## 7. What This Enables

| Scenario | Mechanism |
| --- | --- |
| Chinese text ingested into CAAL | Pass 1 only (already in target space) |
| English text mapped to CAAL | Pass 1 (English hash) + Pass 2 (re-representation into CAAL) |
| Sensor data mapped to CAAL | Pass 1 (sensor hash) + Pass 2 (re-representation into CAAL) |
| I Ching consultation | BSS(H_bridge, hex_i) — native, same bit space |
| Materialize as Chinese | H_bridge → CAAL LUT → Chinese tokens |
| Materialize as English | H_src → English LUT → English tokens (recovery) |
| Cross-domain comparison | 3-gram rank correlation between H_src and H_bridge |
