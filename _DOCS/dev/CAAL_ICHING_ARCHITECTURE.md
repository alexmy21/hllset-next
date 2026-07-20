# CAAL + I Ching: Architecture

> **Session:** July 17–19, 2026
> **Status:** Architecture complete — uncovering existing capabilities
> **Refs:** `UNIVERSAL_BRIDGE.md`, `TF_VS_RANK.md`, `FORECASTING.md`, `DIMENSIONAL_NESTING.md`
> **Supersedes:** Previous versions of this document

---

## 0. The Premise

Chinese is an assembly language for AI. Not metaphorically — structurally.

| Property | Assembly Language | Chinese | HLLSet Algebra |
| --- | --- | --- | --- |
| Fixed instruction set | Fixed opcodes | ~80K characters (stable) | 32,768 bit positions (fixed) |
| Compositional | Instructions combine | Characters combine to form meaning | Bits combine via OR |
| Non-inflectional | No morphology | Analytic language | Bitmasks are static |
| Idempotent | Same opcode = same operation | Same character = same meaning | Same tokens → same HLLSet |
| Deterministic | Given input, output is fixed | Given text, meaning is fixed | Given tokens, hash is fixed |

The CAAL (Chinese Assembly Language) model is a Token-LUT + vocabulary.
The I Ching model is a sub-lattice of CAAL with a fixed R-link structure.
Together they form a universal strategic core — any domain maps in, the
I Ching interprets, the CAAL LUT materializes guidance in Chinese.

**Key discovery (July 19):** CAAL and I Ching are not special architectures.
They are application labels on top of the universal bridge algorithm
(`UNIVERSAL_BRIDGE.md`). The same mechanism works for any domain pair.
Chinese was chosen because its linguistic properties match IICA perfectly,
but the algorithm doesn't care.

---

## 1. Architecture

```text
┌──────────────────────────────────────────────────────────────┐
│                    SOURCE DOMAIN (any)                       │
│  English text, sensor readings, latent vectors, robot state  │
└───────────────────────────┬──────────────────────────────────┘
                            │
                    ┌───────▼─────────┐
                    │  UNIVERSAL      │  Two-pass ingestion:
                    │  BRIDGE         │  Pass 1: murmurhash3 → H_src
                    │                 │  Pass 2: 18-bit bucket → H_bridge
                    │  (hllset-bridge)│  H_bridge lives in CAAL bit space
                    └───────┬─────────┘
                            │
┌───────────────────────────▼──────────────────────────────────┐
│                    CAAL LATTICE                              │
│                                                              │
│  Shared o-HLLSet pool: 80K characters × 1/2/3-grams          │
│  LUT: character → murmurhash3 → (reg, tz) → monotonic TF     │
│  Materializer: HLLSet → Chinese tokens (CAAL LUT)            │
│  Temporal pyramid: ingestion history + ΔTF, Δ²TF             │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐    │
│  │              I CHING SUB-LATTICE                     │    │
│  │                                                      │    │
│  │  Same o-HLLSet pool. Same hash function.             │    │
│  │  64 hexagram HLLSets (union of ~1,178 paragraphs)    │    │
│  │  R-link matrix: hex_i ∩ hex_j (pre-computed, fixed)  │    │
│  │  Consultation: BSS(H_bridge, hex_i) — native BSS     │    │
│  │  Navigation: R-link + TF_forecast → next hexagram    │    │
│  │  Re-ranking: I Ching-internal rank overrides CAAL    │    │
│  └──────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
                            │
                    ┌───────▼────────┐
                    │   ACTUATOR     │
                    │  CAAL LUT →    │
                    │  Chinese text  │
                    └───────┬────────┘
                            │
┌───────────────────────────▼──────────────────────────────────┐
│                    HOST SYSTEM                               │
│  Robot:   Chinese instruction → VLA strategic prompt         │
│  Conv AI: translate → host language → feed back              │
│                                                              │
│  PURPOSE: Make the host system READY for the future.         │
│           Not prediction. Not translation. Readiness.        │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. The Universal Bridge (Two-Pass Ingestion)

Every external input enters the HLLSet Algebra twice. The algorithm is
domain-agnostic — it's the same for English, sensors, or any other source.

### Pass 1: Representation

```text
domain_input → murmurhash3 → H_src (lives in source domain's bit space)
```

### Pass 2: Re-Representation (18-bit Bucketing)

```text
H_src's active bit positions → h_18(pos) → CAAL vocabulary index → H_bridge

h_18: source bit position → 18-bit integer (0..262143)
      Each 18-bit bucket maps to one CAAL token hash (1/2/3-gram)
      Vocabulary: ~240K entries covering all n-gram levels

H_bridge lives in CAAL bit space.
BSS(H_bridge, any_CAAL_HLLSet) works directly.
```

This is compression by bucketing: $2^{64}$ possible source positions → $2^{18}$
buckets → 240K CAAL token hashes. Analogy: the same aggregation that maps
bit-ranks to register-ranks in the five-level rank algebra (Section 5.1 of
the architecture doc). The 18-bit hash is the $G$ function — deterministic
reduction while preserving IICA.

The CAAL vocabulary list (~240K entries × 8 bytes per hash ≈ 2MB) is the
only pre-computed structure. The bridge is not a table lookup of $2^{64}$
entries — it's $h_{18}$ applied to active bits only. $O(\text{popcount}(H_{\text{src}}))$.

### Rank Convergence (Cold Start)

At $t=0$, source-domain and CAAL rank distributions are uncorrelated.
The bridge uses blended rank with temperature annealing:

```text
blended_rank = α(t) · source_rank + (1-α(t)) · CAAL_rank
α(t) = 0.5 · e^(-λt)
```

Same mechanism as Section 18 of the architecture doc (exploration →
exploitation). Over time, frequent source tokens consistently hit the same
18-bit buckets → same CAAL tokens → their CAAL TF accumulates → ranks
converge → α → 0.

---

## 3. CAAL: Chinese Assembly Language

### 3.1 Why Chinese

- **Fixed character set.** ~80K characters (Unicode CJK). The vocabulary
  does not grow. New concepts = new combinations of existing characters.
- **Analytic language.** No inflection. Characters are invariant under
  grammatical context. Every occurrence of the same character hashes to
  the same bit position, every time.
- **1.4 billion speakers.** Everything humanly expressible has a Chinese
  representation. The vocabulary IS the concept space.

### 3.2 Tokenization

```text
Input: "车辆在十字路口" (vehicle at intersection)

Tokens produced:
  Character unigrams:  车, 辆, 在, 十, 字, 路, 口
  Character bigrams:   车辆, 辆在, 在十, 十字, 字路, 路口
  Character trigrams:  车辆在, 辆在十, 在十字, 十字路, 字路口
  Word segmentation:   车辆, 在, 十字路口
  Boundary markers:    _START_, _END_

Each token → murmurhash3 → (register, trailing_zeros) → sets bit in HLLSet
```

All granularities feed into the same HLLSet. The TF vector determines
which positions carry weight at query time. Token-sequence padding on
3-grams ensures every 1-gram in the vocabulary is covered.

### 3.3 The CAAL LUT

```text
character "山" → {(reg=42, tz=17): {tf: 0.03, first_seen: t=0}, ...}
character "水" → {(reg=891, tz=3): {tf: 0.07, first_seen: t=0}, ...}
...
~80K characters × 1-3 hash positions ≈ 200K entries × ~64 bytes ≈ 12.8 MB
```

Monotonic CRDT. TF accumulates with ingestion. The LUT is CAAL's
"understanding" of Chinese — which characters appear, how often, when.

### 3.4 Materialization

HLLSet → CAAL LUT → Chinese tokens. The materializer resolves ambiguous
bit positions via LUT: highest-TF character at this position right now.
The TF lens selects which characters materialize.

Domain-LUT resolution: HLLSets materialized through their native domain's
LUT produce meaningful output. Through the wrong LUT — meaningless. The
architecture never cross-materializes because each HLLSet's bit space
identifies its domain.

---

## 4. I Ching: Strategic Sub-Lattice

### 4.1 Same Foundation as CAAL

The I Ching shares the CAAL o-HLLSet pool and hash function. Its HLLSets
are CAAL citizens — BSS and R-links work directly, no bridge needed for
internal operations. The I Ching is distinguished by its **fixed lattice
structure** — the hexagram R-link matrix is pre-computed once and immutable.

### 4.2 Corpus Structure

```text
For each of 64 hexagrams:
  Ingest scripture, xiang, line commentaries → ~1,178 paragraph HLLSets
  Each paragraph → key: i:<sha1>

  Hexagram HLLSet = union of all its paragraph HLLSets → key: h:<sha1>

Total: 64 hexagram HLLSets + ~1,178 paragraph HLLSets ≈ 1,242 HLLSets ≈ 5 MB
```

### 4.3 Hexagram R-Link Matrix

$$R_{ij} = H_i \cap H_j \quad \text{for all } i,j \in \{0, \ldots, 63\}$$

Pre-computed once. Each $R_{ij}$ is a storable HLLSet (`r:<sha1>`).
Weight = popcount. High popcount → structurally related → smooth transition.
Low popcount → phase change. The intersections ARE the edges. No separate
graph database.

### 4.4 Consultation Engine

```text
1. Current: h_curr = argmax BSS(H_bridge, H_i) for i in 0..63
   H_bridge is a CAAL citizen → native BSS, no cross-domain needed

2. Candidates: {j : popcount(R_{curr,j}) > threshold}

3. Rank: score(j) = f(BSS(H_bridge, H_j), R_weight(curr, j), TF_forecast(j))

4. Select: h_next = argmax score(j)
```

### 4.5 Re-Ranking Within the I Ching Sub-Lattice

The I Ching overrides CAAL's global rank ordering. Within the sub-lattice,
HLLSets are re-ranked using I Ching-internal structure (R-link in/out-degree,
hexagram-specific TF). This is the mechanism that applies I Ching wisdom:
it reorders priorities based on the hexagram transition structure, not the
global CAAL frequency distribution.

### 4.6 Temporal Pyramid + Flashlight Forecasting

Each consultation is a scan ingested into the I Ching's own temporal pyramid.
The TF flashlight pre-positions the system: before the next consultation,
hexagrams likely to become relevant are already ranked. When $H_{\text{bridge}}$
arrives, the system is oriented.

---

## 5. CAAL and I Ching as Application Labels

The architecture doesn't require CAAL or I Ching specifically. It requires:

1. A tokenizer (any language, any domain)
2. A LUT mapping tokens to bit positions
3. A pre-computed sub-lattice for strategic interpretation (optional)

Chinese was chosen because its properties match IICA perfectly. The I Ching
was chosen because its 3,000-year-old hexagram structure provides a rich,
documented, content-addressed strategic vocabulary. But the same architecture
with a different tokenizer and corpus would serve any domain.

```text
CAAL + I Ching = application instance of {universal bridge + token-LUT + sub-lattice}
```

---

## 6. Nothing Was Added

Every mechanism in this architecture existed before the session:

| Mechanism | Where it was already |
| --- | --- |
| Two-pass ingestion (re-representation) | murmurhash3 called twice on different strings |
| 18-bit bucketing | $G$ function in five-level rank algebra |
| Rank convergence (blended + annealing) | Temperature parameter, Section 18 |
| I Ching sub-lattice re-ranking | Rank reshuffling, Section 5 |
| Flashlight forecasting | TF pyramid, Section 4 / FORECASTING.md |
| Domain LUTs for materialization | Existing materializer infrastructure |
| Swarm/dimensional nesting | DIMENSIONAL_NESTING.md |

The session didn't build. It uncovered.
