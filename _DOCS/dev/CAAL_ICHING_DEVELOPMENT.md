# CAAL + I Ching: Development Plan

> **Session:** July 19, 2026
> **Status:** Ready for implementation
> **Refs:** `CAAL_ICHING_ARCHITECTURE.md`, `UNIVERSAL_BRIDGE.md`, `TF_VS_RANK.md`
> **Supersedes:** Previous versions of this document

---

## 0. Architecture Summary

```text
Source domain (any) → UNIVERSAL BRIDGE → CAAL lattice → I Ching sub-lattice → actuator
                         ↑                    ↑                 ↑
                    hllset-bridge        hllset-caal      hllset-iching
                    (one algorithm)    (tokenizer+LUT)   (corpus+R-links)
```

Three crates. One algorithm. Two applications. The bridge is domain-universal;
CAAL and I Ching are configuration + data.

---

## 1. Key Design Decisions

### 1.1 Shared o-HLLSet Foundation

CAAL and I Ching share the same o-HLLSet pool (I Ching vocabulary ingested
as flat o-level HLLSets). CAAL builds its lattice dynamically as it ingests.
I Ching overlays a fixed, pre-computed R-link structure on the same pool. The
I Ching lattice is immutable — pre-computed once, never rebuilt.

### 1.2 The Bridge Is Universal

The bridge algorithm is not CAAL-specific. It maps any HLLSet into any
HLLSet lattice via two-pass ingestion + 18-bit bucketing. It applies within
a single domain (new text → existing lattice) and across domains (English →
CAAL). One algorithm, one crate, one call pattern:

```text
hllset_bridge::map(source_hllset, target_lattice) → H_bridge
```

### 1.3 Two-Pass Ingestion with 18-Bit Bucketing

```text
Pass 1: domain_input → murmurhash3 → H_src
Pass 2: H_src active bits → h_18(pos) → CAAL vocabulary index → H_bridge
```

$h_{18}$ maps each source bit position to an 18-bit bucket (0..262143),
covering the extended CAAL vocabulary with 1/2/3-grams (~240K entries).
The CAAL vocabulary list is the only pre-computed structure (~2MB).

### 1.4 Cold-Start Rank Convergence

Blended rank with temperature annealing (same mechanism as Section 18 of
the architecture doc):

```text
blended_rank = α(t) · source_rank + (1-α(t)) · target_rank
α(t) = 0.5 · e^(-λt)
```

Frequent co-occurrences converge naturally via monotonic TF accumulation.

### 1.5 Domain LUTs for Resolution

Each domain owns its Token-LUT. Materialization goes through the LUT of
the HLLSet's native domain. $H_{\text{src}}$ materializes through source LUT
(≈ original tokens). $H_{\text{bridge}}$ materializes through CAAL LUT
(≈ structural interpretation in Chinese). No cross-materialization.

---

## 2. Development Phases

### Phase 1: Chinese Tokenizer (Foundation)

**Goal:** Ingest Chinese text into o-HLLSets. This is the only domain-specific
code — everything else is universal.

**Crate:** `crates/hllset-caal/`

| Step | Deliverable | Verification |
| --- | --- | --- |
| 1.1 | Chinese vocabulary: 80K characters + extensions for 240K 1/2/3-gram coverage | `vocabulary.json` loads |
| 1.2 | Tokenizer: text → unigrams, bigrams, trigrams → HLLSet | Given "车辆在十字路口", produces correct token list |
| 1.3 | CAAL LUT: token → murmurhash3 → (reg, tz) → monotonic TF | TF tracks correctly across ingestions |
| 1.4 | Materializer: HLLSet → CAAL LUT → Chinese tokens | Round-trip: text → HLLSet → text approximates original |
| 1.5 | Ingest I Ching corpus → o-HLLSets (flat, no lattice yet) | Each paragraph has content-addressed key |
| 1.6 | Notebook: `11_caal_core.ipynb` | |

### Phase 2: Universal Bridge (Algorithm)

**Goal:** One algorithm mapping any HLLSet into any HLLSet lattice.

**Crate:** `crates/hllset-bridge/`

| Step | Deliverable | Verification |
| --- | --- | --- |
| 2.1 | Two-pass ingestion: Pass 1 (murmurhash3) + Pass 2 (18-bit bucket → target) | $H_{\text{bridge}}$ is a citizen of target bit space |
| 2.2 | 18-bit bucketing: `h_18(pos)` → CAAL vocabulary index → target HLLSet | Deterministic, idempotent, BSS with source ≈ 0 |
| 2.3 | Blended rank: `α(t) · source_rank + (1-α(t)) · target_rank` with annealing | Cold-start converges over time |
| 2.4 | Bridge API: `bridge(source_HLLSet, target_lattice) → H_bridge` | Works for same-domain and cross-domain |
| 2.5 | Notebook: `12_universal_bridge.ipynb` | |

### Phase 3: I Ching Sub-Lattice (Confirmation)

**Goal:** Confirm I Ching lattice is immutable, pre-computed once, correct.

**Crate:** `crates/hllset-iching/`

| Step | Deliverable | Verification |
| --- | --- | --- |
| 3.1 | Hexagram HLLSets: union of paragraph o-HLLSets per hexagram | 64 hexagram HLLSets, keys: `h:<sha1>` |
| 3.2 | R-link matrix: 64×64 hexagram intersections | All $R_{ij}$ keys: `r:<sha1>`, weights correct |
| 3.3 | Consultation: BSS($H_{\text{bridge}}$, $H_i$) → current hexagram | Correct hexagram for known test scenes |
| 3.4 | Re-ranking: I Ching internal rank overrides CAAL global rank | Hexagram ranks reflect I Ching structure |
| 3.5 | Navigation: R-link + TF_forecast → next hexagram | Coherent transitions |
| 3.6 | I Ching temporal pyramid + flashlight forecasting | Forecast pre-ranks hexagrams |
| 3.7 | Notebook: `13_iching_core.ipynb` | |

### Phase 4: Integration (End-to-End)

**Goal:** Source domain → bridge → CAAL → I Ching → actuator.

**No new crate.** Integration notebook only.

| Step | Deliverable | Verification |
| --- | --- | --- |
| 4.1 | English text → bridge → $H_{\text{bridge}}$ → I Ching consultation | Correct hexagram selected |
| 4.2 | I Ching re-ranking + R-HLLSet pattern identification | Valid transition path |
| 4.3 | CAAL materializer → Chinese guidance text | Valid Chinese output |
| 4.4 | Actuator (robot): Chinese → VLA prompt scenario | Plausible strategic instruction |
| 4.5 | Actuator (conv): Chinese → translate → feed back | User receives guidance |
| 4.6 | Notebook: `14_caal_iching_e2e.ipynb` | |

---

## 3. Crate Structure

```text
hllset-next/
├── crates/
│   ├── hllset-core/            ← existing
│   ├── hllset-dsl/             ← existing
│   ├── hllset-storage/         ← existing
│   ├── hllset-materialize/     ← existing
│   ├── hllset-forth/           ← existing
│   ├── hllset-bridge/          ← NEW: Phase 2 (universal algorithm)
│   │   └── src/
│   │       ├── lib.rs           ← bridge(source, target_lattice) → H_bridge
│   │       ├── two_pass.rs      ← Pass 1 + Pass 2 ingestion
│   │       ├── bucket.rs        ← 18-bit bucket → vocabulary index
│   │       └── blend.rs         ← Blended rank with temperature annealing
│   ├── hllset-caal/            ← NEW: Phase 1 (tokenizer + LUT)
│   │   ├── src/
│   │   │   ├── lib.rs           ← CAAL tokenizer, LUT, materializer
│   │   │   ├── tokenizer.rs     ← Chinese text → 1/2/3-grams → HLLSet
│   │   │   ├── lut.rs           ← Token → hash position → monotonic TF
│   │   │   └── materialize.rs   ← HLLSet → Chinese tokens via LUT
│   │   └── data/
│   │       └── vocabulary.json  ← 80K characters + n-gram extensions
│   └── hllset-iching/          ← NEW: Phase 3 (corpus + R-links)
│       ├── src/
│       │   ├── lib.rs           ← I Ching lattice
│       │   ├── corpus.rs        ← Ingest → paragraph o-HLLSets
│       │   ├── hexagram.rs      ← Hexagram HLLSets + R-link matrix
│       │   ├── consult.rs       ← BSS-based hexagram selection
│       │   └── navigate.rs      ← R-link + TF_forecast → transition
│       └── data/
│           └── iching_corpus/   ← Original I Ching texts (Chinese)
├── _DOCS/
│   └── dev/
│       ├── UNIVERSAL_BRIDGE.md
│       ├── CAAL_ICHING_ARCHITECTURE.md
│       ├── CAAL_ICHING_DEVELOPMENT.md   ← this document
│       ├── TF_VS_RANK.md
│       ├── FORECASTING.md
│       ├── DIMENSIONAL_NESTING.md
│       └── forecasting_illustration.py
└── Cargo.toml
```

---

## 4. Dependency Graph

```text
hllset-iching
    ├── hllset-caal     (tokenizer, LUT, materializer)
    ├── hllset-storage  (IPFS content addressing)
    └── hllset-core     (∪, ∩, \, popcount, key, BSS)

hllset-caal
    ├── hllset-storage
    └── hllset-core

hllset-bridge
    └── hllset-core     (only — domain-agnostic, no CAAL dependency)

hllset-iching uses hllset-bridge at integration time
    (Phase 4), not as a compile dependency
```

---

## 5. Immediate Next Step

**Phase 1, Step 1.1:** Create `crates/hllset-caal/` with the Chinese
vocabulary file. Source: Unicode CJK Unified Ideographs (U+4E00–U+9FFF)
plus extensions. Public domain.

The CAAL vocabulary list is the only pre-computed structure the entire
architecture needs. Everything else flows from it.
