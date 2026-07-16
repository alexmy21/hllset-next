# FPGA Self-Reprogramming via Forth hllang

> **Session:** June 27, 2026
> **Status:** Architectural exploration — complete
> **Notebook:** `_DOCS/notebooks/06_fpga_self_reprogram.ipynb`
>
> *Most solutions emerged through dialogue. Neither participant had them before the conversation.*

---

## 1. Core Idea

HLLSet lattice evolution **IS** system evolution. The FPGA doesn't run an HLLSet
pipeline — the pipeline's own lattice state *is* the program, and state
transitions *are* reprogramming events.

```text
H(t) = H( S(t), H(t-1), D(t-1), R(t-1), N(t) )

where:
  S(t)     = current scan — new observation of the system as an HLLSet
  H(t-1)   = previous lattice state
  D(t-1)   = Departed   = H_prev - H_curr
  R(t-1)   = Retained   = H_prev ∩ H_curr
  N(t)     = New        = H_curr - H_prev
```

D, R, N are themselves HLLSets. The evolution record IS an HLLSet.

---

## 2. Temporal Lattice Layers — Semantic Time Pyramid

**Building is automatic. Utilization is system-dependant. The layers are
named by WHAT they represent (temporal scale), not by what you do with them.**

### The Time Pyramid

```text
Layer 6  YEAR     L6 = ∪ S(t) over 365 days          ← coarsest
Layer 5  MONTH    L5 = ∪ S(t) over 30 days
Layer 4  WEEK     L4 = ∪ S(t) over 7 days
Layer 3  DAY      L3 = ∪ S(t) over 24 hours
Layer 2  HOUR     L2 = ∪ S(t) over 60 minutes
Layer 1  MINUTE   L1 = ∪ S(t) over 60 seconds
Layer 0  SECOND   L0 = ∪ S(t) over current second    ← finest

Total coverage: 7 layers → ~1 year of compressed history
```

### Automatic Building (Union Aggregation)

The pyramid builds itself. No configuration. No policy. It's mechanical:

```text
Every second boundary:
  L1 = L1 ∪ L0          // previous second absorbed into minute
  L0 = ∅               // reset for next second

Every minute boundary:
  L2 = L2 ∪ L1          // previous minute absorbed into hour
  L1 = ∅

Every hour boundary:
  L3 = L3 ∪ L2          // previous hour absorbed into day
  L2 = ∅

...and so on up to L6 (year)
```

After compression, layers are **mutually exclusive** — no time slice appears
in more than one layer. The complete system state is their union:

```text
H_system(t) = L0 ∪ L1 ∪ L2 ∪ L3 ∪ L4 ∪ L5 ∪ L6
```

Each S(t) enters at L0, percolates upward through compression. At any moment,
exactly one layer contains any given moment of the system's history.
The union is **bit-lossless** — every bit from every S(t) survives. What is
lost is **temporal differentiation**: you cannot recover which second within
a minute a bit came from. The layer is a compressed index, not a replacement.
Every original S(t) HLLSet remains stored in IPFS at its own CID and can be
retrieved for full temporal resolution. Fragile signals survive at fine
layers; persistent signals propagate upward.

### The Noether Invariant: Eventual Consistency by Construction

The union equation is more than a definition — it's a conservation law:

```math
\bigcup_{i=0}^{6} L_i = \text{constant over time}
```

Noether's theorem: every continuous symmetry has a corresponding conserved
quantity. Here the symmetry is **path-independence**. Information can travel
L0→L1→L2 or L0→L2 directly or L1→L4→L5. Multiple R-link gates may drop.
Individual carries may be delayed. But the union of all layers is invariant:

```text
  Symmetry:     multiple paths through the pyramid
  Conserved:    total information in the union of all layers
  Guarantee:    H_system converges regardless of path
```

This IS eventual consistency — not as a protocol we must implement, but as a
property that falls out of the structure. No consensus algorithm. No leader
election. No retry logic. The lattice converges because the union is
monotonic and the multiple paths guarantee that every bit eventually reaches
every layer that needs it.

**Implication for the feedback gate:** R-link gates can be aggressive. Drop
everything below BSS 0.6. Drop entire carries. The information isn't lost —
it arrives via another path. The gate is an optimization, not a filter.

### The Layer Vector as System Clock

The seven layers form a **temporal address space**. Like a clock displays
`14:30:15`, the layer vector pinpoints when something happened:

```text
  L6  :  L5  :  L4 :  L3  : L2 : L1 : L0
 YEAR  MONTH  WEEK  DAY   HOUR MIN  SEC

Example addresses:
  2026:JULY:W1:THU:14:30:15   — exact moment
  2026:JULY:*:*:*:*:*         — "sometime in July 2026"
  *:*:*:YESTERDAY:14:*:*      — "yesterday afternoon"
```

This is more than a clock — it's a **content-addressable temporal coordinate
system**. Each coordinate maps to an HLLSet. Queries become BSS operations
at specific layer granularities:

```text
"Compare today vs yesterday at hour granularity":
  BSS(L3:today:L2:14, L3:yesterday:L2:14)

"Has this pattern appeared in the last month?"
  BSS(S(t), L5)

"Show me July at week resolution":
  L6:2026:L5:JULY  → yields the week layer within that month
```

### Compression Ratios

```text
  L0 → L1:  60:1    (seconds → minute)
  L1 → L2:  60:1    (minutes → hour)
  L2 → L3:  24:1    (hours → day)
  L3 → L4:   7:1    (days → week)
  L4 → L5:  ~4:1    (weeks → month)
  L5 → L6:  12:1    (months → year)

  Total pyramid: 60×60×24×7×4×12 ≈ 14.5 million seconds compressed into 1 HLLSet
```

### The Pyramid Is a Parameter, Not a Constant

The 7-layer second→year pyramid is **one instance** of a general sliding window.
The pyramid shape is a tunable parameter: number of layers $N$ and their
durations $[d_0, d_1, \ldots, d_{N-1}]$.

The general form:

```text
N layers with durations [d₀, d₁, ..., dₙ₋₁]
Each layer Lᵢ covers timespan dᵢ
Compression ratio Lᵢ → Lᵢ₊₁ = dᵢ₊₁ / dᵢ
```

**Example configurations:**

| Application | N | [d₀..dₙ₋₁] | Total span | Character |
| ------------- | --- | ------------- | ------------ | ----------- |
| High-frequency trading | 5 | 100ms each | 500ms | Micro-burst detection |
| Real-time control | 4 | 250ms each | 1s | Fast reflex, no deep history |
| Conversational agent | 10 | 6s each | 1min | Sentence-to-sentence coherence |
| Document analysis | 6 | 10min each | 1hr | Section-level context |
| Original (default) | 7 | [1s, 1min, 1hr, 1d, 1w, 1mo, 1yr] | 1yr | Long-term memory |

Nothing else in the architecture changes. The Noether steering, Fisher matrix,
rank algebra, and observable mask operate identically across all configurations.
The Fisher matrix $F_{bb'}$ still counts co-occurrences across $N$ layers —
more layers give finer temporal resolution in the coupling signal; fewer layers
give faster rotation and lower memory footprint.

**Design principle.** The pyramid is not a calendar. It is a configurable
sliding window whose depth and granularity are chosen per application. The
architecture is invariant under changes to $(N, [d_i])$ — the same FPGA,
the same operations, the same rank framework.

This brings the full control surface to six tunable parameters:

```text
Controller = {
    pyramid:    { layers: N, durations: [d₀..dₙ₋₁] },
    ranks:      { F, G, H, K, L, M },
    attention:  { θ (threshold) },
    steering:   { bit_threshold, rank_threshold },
}
```

Six knobs. Same FPGA. Any scale.

### The Layer Clock: Mechanical Precision

The pyramid operates like a mechanical arithmometer. L0 is the fastest wheel.
When it completes a full rotation (60 seconds), it ticks L1 forward one
position. When L1 completes 60 minutes, it ticks L2. And so on:

```text
  L0 tick (every second):     S(t) ingested, DRN computed for current second
  L0 → L1 carry (at 60s):     L1 = L1 ∪ L0,  L0 = ∅,  DRN computed at minute boundary
  L1 → L2 carry (at 60min):   L2 = L2 ∪ L1,  L1 = ∅,  DRN computed at hour boundary
  ...up to L6
```

**DRN runs at every carry.** Not just at L0. Every time any layer advances,
we compute:

```math
H(t_{\text{layer}}) = H\big(S(t), H(t_{\text{prev}}), D, R, N\big)
```

where $R = H_{\text{prev}} \cap H_{\text{curr}}$ captures the overlap between
the previous state of this layer and the newly absorbed content. This R-link
is the thread connecting layers through time.

### Navigational Structure: R-Links as Temporal Edges

Each carry produces an R-link. These form a navigable **temporal graph**:

```text
  L0(t) ──R₀₁──→ L0(t-1) ──R₀₁──→ L0(t-2) ──→ ...
    │               │
    │ carry         │ carry
    ▼               ▼
  L1(h) ──R₁₂──→ L1(h-1) ──→ ...
    │
    │ carry
    ▼
  L2(d) ──→ ...
```

Each edge has three properties:

| Property | Source | Meaning |
| --- | --- | --- |
| **Weight** | `popcount(R)` | How many bit positions overlap? |
| **Rank** | `Σ TF[b] for b ∈ R` | How much cumulative activity at the overlap? |
| **Latency** | Layer clock period | How much real time between these nodes? |

Latency is deterministic:

- Following an R-link within L0: 1 second
- Following a carry from L0 upward: 1 minute
- Following an R-link within L3: 1 day
- Crossing from L0 to L6 via R-link chain: ~1 year

### Navigation by R-Link Strength

To navigate the lattice, follow strong R-links:

```text
navigate(position, threshold τ):
    for each adjacent layer L_j:
        R_ij = L_i ∩ L_j
        if BSS(L_i, L_j) > τ:     // the R-link is strong enough
            follow(R_ij)          // move to L_j
```

**Interpretation:**

- **Strong R-link** (high BSS): The layers are coherent — what happened in
  L_i is still present in L_j. Follow it to trace a signal's path through time.
- **Weak R-link** (low BSS): Phase boundary — the system changed significantly
  between these layers. A bifurcation point. Branch here to find a different
  causal path.
- **Multiple paths**: The same event may connect to multiple layers. Follow
  the strongest R-link for the primary causal chain; follow weaker ones for
  alternative explanations.

### Message Propagation with Known Latency

A signal entering at L0 propagates upward through the R-link graph:

```text
  Signal at L0(t) → R₀₁ → L0(t-1) → carry → L1(h)
                                          → R₁₂ → L1(h-1) → carry → L2(d)
                                                                     → ...

  Latency: L0 → L1 = 1 minute
           L1 → L2 = 1 hour
           L0 → L6 = ~1 year (following the R-link chain)
```

This enables **causal queries**: "Did the event at L0(t=0) cause the pattern
at L3(t=3)?" Answer: walk the R-link chain. If strong R-links connect them
and the latency matches the expected propagation time, causality is supported.
If R-links are weak or the chain is broken, the events are independent.

The R-link graph IS the system's causal memory. No external clock needed —
the carry mechanism and R-link strength provide both timing and connectivity.

### R-Links as Feedback Gates

R-links don't just trace causality — they **select what to feed back** into
the main loop. Without gating, the materializer feeds every candidate token
into the tokenizer, flooding the perceptron with noise. R-links filter the
flood:

```text
Raw loop (no gating):
  S(t) → materialize all layers → all tokens → tokenizer → noise

Gated loop (R-link selection):
  S(t) → for each layer L_i:
           R = S(t) ∩ L_i                          // compute R-link
           if popcount(R) > θ:                     // is it relevant?
               materialize(L_i)                    // yes, feed it back
               tokenizer(feedback) → mesh with S(t+1)

  Strong R-link    → feedback HLLSet IS relevant → include
  Weak R-link      → HLLSet unrelated to current scan → skip
```

This transforms the feedback loop from "replay everything" to "replay what
matters right now." The R-link weight IS the relevance score.  No separate
relevance model. No attention mechanism. Just bitwise AND + popcount.

**Why R-link weight is statistically meaningful.** A bit in $R_{01} =
L_0 \cap L_1$ does not represent a hash collision — it represents a token
that appeared in multiple seconds within that minute. The weight of $R_{01}$
counts how many seconds carried that token. $R_{12}$ tracks persistence
across 60 minutes. $R_{23}$ across 24 hours. What looks like "overlap" is
a statistical measure of temporal durability. The deeper the carry that
survives, the stronger the evidence.

**Gating with BSS.** Use BSS as the threshold, not raw popcount:

```text
  S(t) → for each layer L_i:
           τ = BSS(S(t), L_i)
           if τ > 0.6:  materialize and feed back
           else:         skip — weak R-link
```

If BSS falls below threshold, the R-link is dropped — but information
isn't lost. The same tokens flow through multiple R-link paths (L0→L1
carry, L0→L2 carry, etc.). Redundancy in the pyramid means a weak R-link
at one level is compensated by stronger ones at others.

### Utilization is System-Dependant

The same pyramid serves many purposes:

```text
  Real-time controller:  query L0 (what's happening now?)
  Anomaly detection:     BSS(L0, L2) — minute-scale baseline
  Trend analysis:        BSS(L3_today, L4_last_week)
  Long-term memory:      L6 → persistent "what happened this year"
  FPGA reprogramming:    if BSS(L0, L1) < threshold → reload from L2
  Causal tracing:        follow R-link chain from L0 to target layer
```

The building is mechanical (automatic carries). The navigation is
intentional (R-link traversal with BSS thresholds). The same pyramid
is a clock, a memory, and a causal graph — depending on how you walk it.

### Cross-Layer BSS as Cognitive State

```text
        L0       L1       L2       L3    ...    L6
  L0    —       τ(0,1)   τ(0,2)   τ(0,3)      τ(0,6)
  L1    τ(1,0)    —      τ(1,2)   τ(1,3)      τ(1,6)
  L2    τ(2,0)  τ(2,1)     —      τ(2,3)      τ(2,6)
  ...
```

The BSS matrix between all layer pairs tells the controller:

- High τ(0,1): the last second is consistent with the last minute — stable
- Low τ(0,1): sudden change — attention required
- High τ(0,6): the current moment resembles the year's aggregate — déjà vu
- Drifting τ over time: adaptation in progress
temporal depths, all running simultaneously, each at a different stage
of maturity.

### BSS Convergence

Measurable signal that tracks the phase transitions:

```text
t=1: signal(mixed_stream) ≈ 0.2   # crude, few tokens in memory
t=2: signal(mixed_stream) ≈ 0.5   # sharpening
t=3: signal(mixed_stream) ≈ 0.8   # converging toward understanding

t=1: Δ = 1.0     (first step, everything is "new")
t=2: Δ = 0.6     (less new, more retained)
t=3: Δ = 0.3     (stabilizing)
t=4: Δ → 0.15    (converging — "watching TV")
```

---

## 3. Fire-and-Forget State Communication

Each state in the window sends output to exactly two destinations — no
coordination, no acknowledgments, no retries:

```text
Window W(t) = { S0(now), S1, S2, S3, S4(deepest) }
               |   |    |   |   |
               v   v    v   v   v
          [ MATERIALIZER ] ← collects ALL outputs
          (final aggregator, guaranteed complete)
               |
          S0 → S1 → S2 → S3 → S4
          (aggregation chain, lossy OK)
```

**Key properties:**

- Each state Si sends to the **materializer** (fire-and-forget, always succeeds — content-addressed)
- Each state Si sends to **Si+1** for aggregation (fire-and-forget, lossy tolerated)
- Si+1 may miss Si's output — not a problem: the materializer still received it directly
- The materializer eventually has `H(S0) ∪ H(S1) ∪ ... ∪ H(S4)` — the complete picture
- The materializer is NOT in the feedback loop — it's the exit point, the "TV watcher"

**No coordination required.** Each state is an independent fire-and-forget node.
This is possible because HLLSets are idempotent (sending twice changes nothing),
content-addressed (no race conditions on identity), and CRDT-mergeable (union
is commutative — any order produces the same result).

---

## 4. Noether Controller 2.0 — Layer-Driven Reprogramming

The Noether controller no longer checks a simple threshold ("Δ > 0.3?").
It reads the cross-layer BSS matrix and decides **which layer** should drive
the next action:

```text
if L0 (second) ↔ L1 (minute) τ < 0.5:
    → DIVERGENCE detected
    → Reprogram FPGA with L1 (minute)'s HLLSet (override instinct with context)

elif L0 (second) ↔ L3 (day) τ < 0.3:
    → RE-ROUTE: current input diverging from long-term goal
    → Reprogram with L3 (day)'s HLLSet (restore direction)

else:
    → stable: instinct aligned with context and trajectory
    → Continue with L0 (second)
```

**The BSS matrix IS the controller's input signal.** Not a single scalar,
but a complete cross-layer relationship map.

---

## 5. Rank-Based Learning

**The critical architectural insight of the session:**

HLLSets are IICA — Immutable, Idempotent, Content-Addressed. They CANNOT
change. The tokenizer produces them. The materializer collects them. Neither
of them **learns**.

**Learning = the Forth dictionary reshuffling ranks.**

```text
HLLSets (fixed, content-addressed)
    │
    ▼
Forth Dictionary ──→ assigns RANKS ──→ THIS is learning
    │
    ▼
Behavior = highest-ranked HLLSet drives action
```

### Clean Separation of Concerns

| Component | What it does | Does it learn? |
| ----------- | ------------- | :---: |
| **Tokenizer** | bytes → HLLSet | No (deterministic function) |
| **Materializer** | collects all HLLSet outputs | No (passive observer) |
| **Forth Dictionary** | assigns ranks to HLLSets | **Yes** (only this) |

### The TF Signal: Token-Level vs Bit-Level

Rank adaptation requires a signal. That signal is the TF vector — but the
term "TF" operates at two distinct levels. Confusing them causes reasoning
errors.

| Concept | Level | Question it answers |
| --- | --- | --- |
| **Token TF** | per-token | How often did this word appear? |
| **Bit TF** | per-position (32,768) | How much activity at this hash bucket? |
| **Rank** | per-HLLSet (derived) | How important is this HLLSet right now? |

A bit position aggregates **multiple tokens** that hash to the same
(register, zero-count). Token TF is reduced to bit TF via a pluggable
function $f$:

$$\text{TF}[b] = f\big(\text{tf}(t_1), \ldots, \text{tf}(t_k)\big)$$

Candidates for $f$: sum (total activity), max (dominant token), entropy
(diversity measure). Any monotonic $f$ preserves CRDT convergence. The
rank of an HLLSet is then derived from bit-level TF by projection:

$$\text{rank}(H) = g\big(\{\text{TF}[b] \mid b \in H\}\big)$$

where $g$ is an aggregation function (sum, mean, weighted sum). Neither
$f$ nor $g$ is mandated — they are design parameters chosen per application.

**Key guardrail:** When thinking about "how important is this HLLSet?", you
are computing rank from bit-level TF, not from token-level TF. A bit
position with high TF may represent many distinct tokens that collided
there — not a single "important" token. The distinction prevents the
intuition error of treating bit activity as word frequency.

### How Ranking Works

- Scan arrives → load TF vectors for relevant layers
- Compute bit-level TF projection for every word in the dictionary
- High-signal words get a rank boost (they match the current situation)
- Low-signal words get rank decay (they're less relevant now)
- The highest-ranked word = L0 (second) (drives current action)
- The second-highest = L1 (minute) (was recently instinct)
- Accumulated high-rank words over time = L3 (day) (persistent relevance)

**Roles are NOT fixed by temporal depth.** The same HLLSet can be L0 (second) at
t=0 (its tokens match the scan), L1 (minute) at t=1 (a better match emerges), and
L3 (day) at t=5 (it represents the accumulated goal). The HLLSet didn't change —
its rank did.

### The Google Parallel

```text
Google:   query → PageRank → top result → you click → advertiser pays → rank adjusts
Our sys:  S(t)  → TF signal → rank → L0 (second) → behavior → perceptron feeds → rank adjusts
                                                       ▲
                                             the "advertiser" is the
                                             perceptron feedback loop itself
```

The perceptron doesn't lie — it genuinely believes feedback tokens are relevant
because they came from the system's own lattice. The system **advertises to
itself**. What you reacted to at t=0 becomes what you plan for at t=5. A ranking
bubble forms — exactly like Google's filter bubble, but self-generated.

### 5.1 Formal Rank Algebra — Five-Level Hierarchy

Section 5 established that HLLSets are immutable and only ranks change. This
section formalizes the five compositional levels through which rank propagates
from raw token frequency to compound lattice operations. Each level is a
function from its inputs to a rank value; the levels compose deterministically.
All five functions $(F, G, H, K, L, M)$ are pluggable design parameters —
the architecture specifies the framework; the application chooses the functions.

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

**Level 1: Token Rank.** $\text{token-R}(t) = F(\text{TF}(t))$. $F$ is
any monotonic function: identity, log, sigmoid. The only constraint:
higher TF must not produce lower rank, preserving CRDT convergence.

**Level 2: Bit Rank.** $\text{bit-R}(r, tz) = G(\{token-R(t) \mid
\text{hash}(t) = (r, tz)\})$. Multiple tokens may hash to the same
$(r, tz)$ bit. $G$ aggregates their ranks: max (dominant token), sum
(all contribute), weighted mean (balanced). The HLLSet bitmask is
idempotent — setting an already-set bit changes nothing — but $G$
may or may not be idempotent depending on choice.

**tz is an address, not a weight.** A token at tz=31 sets one bit; a token
at tz=0 also sets one bit. The trailing-zero count carries no inherent
importance signal — it is solely the hash output format. All 32 bit
positions within a register are equal citizens at this level.

**Level 3: Register Rank.** $\text{reg-R}(r) = H(\{\text{bit-R}(r, tz)
\mid tz \in 0..31\})$. A register spans 32 bit positions; each carries
its own bit-R from Level 2. $H$ aggregates: mean (uniform), max-pool
(strongest bit dominates), active-only mean (ignores empty tz slots),
population-weighted mean (activity-scaled).

**Level 4: HLLSet Rank.** $\text{hllset-R}(H) = K(\text{degree}(H))$.
Structural importance in the lattice DAG — how many operations produced
or consumed this HLLSet. Candidates: raw degree, popcount-weighted
degree (sum of incident R-link weights), PageRank-like iterative
propagation.

**Level 5: Compound Rank.** When HLLSets combine:

```math
\text{rank}(A \cup B) = L(\max\{\text{rank}(A), \text{rank}(B)\})
```

```math
\text{rank}(A \cap B) = M(\min\{\text{rank}(A), \text{rank}(B)\})
```

Simplest forms: $L = \max$, $M = \min$. More sophisticated forms can
blend register-level contributions from the compound's own structure.

**Summary table:**

| Level | Function | Domain | Design space |
| ------- | ---------- | -------- | ------------- |
| 1 | $F$ | TF → token-R | Identity, log, sigmoid |
| 2 | $G$ | {token-R} → bit-R | Max, sum, weighted mean |
| 3 | $H$ | {bit-R[tz]} → reg-R | Mean, max-pool, active-only |
| 4 | $K$ | Graph → hllset-R | Degree, weighted degree |
| 5 | $L, M$ | R² → compound-R | Max/min, blended |

The HLLSet bitmask is the **structural scaffold** — which bits are set never
changes. The five-level rank hierarchy is the **dynamic signal** layered on
top — it changes with every ingestion, every recombination, every lattice
restructuring. Section 12's per-register TF ranking is the Level 3 mechanism
specialized to the case where $F$ is identity and $G$ aggregates TF-derived
token ranks by their hash positions. The five-level framework generalizes this
to arbitrary monotonic $F$ and pluggable $G$, $H$, $K$, $L$, $M$.

---

## 6. System Lifecycle — Birth, Death, Reproduction

**Systems are mortal. They develop rank bubbles. Instead of tweaking a running
system, you let it live — and spawn a new one.**

```text
┌──────────────────────────────────────────────────────────┐
│                   SYSTEM LIFECYCLE                       │
│                                                          │
│  BIRTH:  Seed HLLSets + initial lattice (ranks)          │
│     │                                                    │
│     ▼                                                    │
│  LIFE:   Tokenizer → HLLSets                             │
│          Forth → reshuffles ranks (learns)               │
│          Materializer → collects outputs                 │
│          Ranks inevitably develop bubbles                │
│     │                                                    │
│     ▼                                                    │
│  DEATH:  Don't fix it. Don't tweak it.                   │
│          Don't hot-patch the ranking.                    │
│     │                                                    │
│     ▼                                                    │
│  REPRODUCE:  Copy HLLSets + lattice → new system         │
│              Fresh start, accumulated knowledge          │
│              No rank bubbles. No pathologies.            │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### Why Reproduction Works (IICA Properties)

| Property | Mechanism | Consequence |
| ---------- | ----------- | ------------- |
| **Immutable** | HLLSets never change | Safe to copy, identical in any system |
| **Idempotent** | Copy twice = same result | IPFS deduplicates automatically |
| **Content-Addressed** | Every HLLSet has a CID | Transfer = `ipfs get <cid>` |

**The lattice (ranks) IS the only mutable state.** It's the system's "mind."
When you spawn a child, you copy:

1. All HLLSets (the knowledge) — trivial, content-addressed, IPFS
2. Current ranks (the learned priorities) — the only mutable state
3. The child starts with accumulated wisdom but a clean rank-dynamic slate

### Knowledge Transfer is Natural

```text
Parent system:  lived, learned, developed consume-bubble (advertiser influence)
Child system:   same HLLSets + same final ranks → fresh start
                new scans reshuffle ranks naturally
                no bubble carried over, only wisdom
                
Transfer:  ipfs get <cid>          (HLLSets, trivial)
           copy the rank vector    (the only mutable state)
           spawn new FPGA/process  (hardware/software)
```

**No hotfixes. No runtime patches. No tweaking rank bubbles.**
Systems don't get fixed — they reproduce. This is the IICA lifecycle.

---

## 7. Content-Addressable Computation (Program Counter)

The FPGA doesn't need a separate program loader. Forth dictionary HLLSets
**are** the DenseLUT configuration. The "program counter" is BSS similarity:

```text
Program Counter = argmax( BSSτ(input_HLLSet, word_HLLSet) for word ∈ dictionary )
```

No program counter, no instruction fetch, no branch prediction — just lattice
similarity routing. The next instruction is whichever Forth word's HLLSet
most closely matches the current input.

---

## 8. Actuation — From Tokens to Ordered Deliverables

The materializer produces candidate tokens. But the real world needs ORDER.
Text needs word order. Images need spatial layout. Robotics needs action
sequences. Audio needs temporal waveforms.

**The tokenizer doesn't preserve global order.** It reduces bytes to an
unordered HLLSet — n-grams encode only local adjacency (bigram "the cat"
knows "the" is next to "cat", but not where in the text it appears).
The materializer recovers candidate tokens from the HLLSet. Between them,
**global order is lost**.

**Actuation = restoring the order for the target modality.**

### DeBruijn — The Text Actuator

The current `materialize.rs` already implements this. n-grams with boundary
markers (`_START_`, `_END_`) encode overlapping adjacency pairs:

The overlap — "cat" is suffix of "the\0cat" AND prefix of "cat\0sat" —
becomes an edge in a De Bruijn graph. The Eulerian path from `_START_`
to `_END_` reconstructs the original word order:

Gaps in the n-grams produce gaps in the reconstructed sequence.
The confidence metric in \`MaterializedResult\` tracks how many
HLLSet bits were resolved.

### Actuation per Modality

| Modality | Token structure | Order encoding | Actuation strategy |
| ---------- | ---------------- | ---------------- | ------------------- |
| **Text** | n-grams with boundary pads | Overlap adjacency | DeBruijn (Eulerian path) |
| **Images** | Spatial patches (HxW) | Patch coordinates | 2D layout reconstruction |
| **Audio** | Spectral bins (Mel) | Temporal frames | Overlap-add windowing |
| **DNA** | k-mers (fixed length) | Overlap by k-1 | DeBruijn (standard) |
| **Robotics** | Action primitives | Temporal sequence | Plan execution ordering |

### Relationship to the Loop

The actuator sits AFTER the materializer. It's the final stage before
the output leaves the system. The materializer is the fire-and-forget
collector; the actuator is the modality-specific renderer.

The actuator does NOT feed back into the loop. It's one-way — the exit point.

---

## 9. Architecture Diagram

```text
                    ┌── L0 (second) (depth 0) ← S(t)
                    │
  W(t) = { L(t),  L(t-1),  L(t-2),  L(t-3),  L(t-4)  }
           now    correct  context  traject  project
             │       │        │        │         │
             │       │        │        │         │
             └───────┴────────┴────────┴─────────┘
                     │        │        │
               Cross-Layer BSS Matrix (cognitive state)
                     │
               Noether Controller
            "Which layer drives action?"
                     │
          ┌──────────┼──────────┐
          ▼          ▼          ▼
       L0 (second)   L1 (minute)    RE-ROUTE
          │          │          │
          └──────────┼──────────┘
                     ▼
            FPGA RECONFIGURE
         (load chosen layer HLLSet)
                     │
                     ▼
            [ MATERIALIZER ]
         (collects all, never feeds back)
```

---

## 10. Summary of Discoveries

| Concept | Discovery |
| --------- | ----------- |
| **Evolution Equation** | D, R, N are themselves HLLSets. The evolution record IS an HLLSet. |
| **Temporal Layers** | All layers simultaneously active. Cross-layer BSS = cognitive state. |
| **Fire-and-Forget** | Each state → materializer + next state. No coordination. Lossy chain OK. |
| **Rank-Based Learning** | HLLSets are immutable. Only ranks change. That IS learning. Clean separation from tokenizer and materializer. |
| **System Lifecycle** | Birth, life, death, reproduction. No hotfixes — spawn a child. IICA makes transfer trivial. |
| **Actuation** | Materializer output is a bag. Actuator restores order: DeBruijn for text, spatial layout for images, action sequencing for robotics. |
| **Content-Addressable PC** | Next instruction = argmax(BSS(input, word)). No counter, no fetch. |
| **The Google Parallel** | PageRank filter bubble ≈ rank bubble from self-advertising perceptron. Same mechanism. |

---

## 11. Implementation Status

All concepts are implemented in `_DOCS/notebooks/06_fpga_self_reprogram.ipynb`
(24 cells, 11 executable code cells). The notebook runs against the `hllset`
CLI binary with inline Lua scripts. All operations verified via lattice
closure tests.

### Next Steps

1. **Multi-die coordination**: Each FPGA die runs a different temporal depth or dictionary chunk
2. **BSS program counter in Verilog**: Hardware implementation of lattice-routed execution
3. **Hardware perceptron**: FEEDBACK/ENVIRONMENT classifier in RTL
4. **Bitstream generation**: Compile Forth dictionary rank-ordered HLLSets → FPGA configuration
5. **Noether controller in silicon**: Cross-layer BSS matrix reader driving partial reconfiguration

---

---

## 12. Topological R-Links -- Replacing BSS with HLLSet Intersection

BSS (Bell State Similarity) is a scalar: tau = |A cap B| / |B|. It requires
floating-point division -- expensive in FPGA, not native to the bitwise fabric.

**Replacement: R = A cap B -- the Retained intersection from D/R/N decomposition.
R is itself an HLLSet, not a scalar. The link weight = popcount(R).**

```text
BEFORE (scalar BSS):
    tau = |A cap B| / |B|     floating-point division, multi-cycle
    Compare tau > threshold   scalar comparison

AFTER (topological R):
    R = A cap B               bitwise AND across 1024 registers, single-cycle
    weight = popcount(R)      count set bits, single-cycle
    Select: f(rank, weight)   integer arithmetic only
```

### Why This Matters for FPGA

| Operation | BSS approach | R-link approach | FPGA cost |
| ----------- | ------------- | ----------------- | ----------- |
| Intersection | \|$A \cap B$\| via float math | $A \cap B$ via bitwise AND | 1 cycle, 1024 parallel gates |
| Cardinality | HT estimator (division) | popcount(R) | 1 cycle, popcount unit |
| Weight | tau in [0,1] float | popcount in [0, 32768] int | No FPU needed |
| Storage | scalar (discard) | R is an HLLSet (keep) | Content-addressed, IPFS |

**The R IS the relationship.** It has a content key (r:\<sha1\>). It can be
stored, compared, and used as input to further operations. BSS was a dead-end
scalar -- compute it, use it, throw it away. R is a lattice element that
persists in the same algebraic space as A and B.

### Selection Function

```text
next = argmax( f(rank, weight) )  for all candidate words in dictionary

where:
    rank   = Forth dictionary rank (learning-based, from Step 7)
    weight = popcount( current_HLLSet cap candidate_HLLSet )
    f      = rank * weight        (simplest form; can be any integer function)
```

### Per-Register TF Ranking

The HLLSet has 1024 registers x 32 bits = 32,768 bit positions. Due to
MurmurHash3 randomization, each bit position is a pointer to a collection
of tokens -- mostly unrelated tokens that happened to hash to the same
(reg, zeros). For each position, we can compute:

```text
TF(pos) = |{tokens at this position that appear in the current scan}|
          / |{all tokens at this position}|
```

The normalized TF across positions gives a per-bit weight. Since HLLSet
intersection is bitwise AND, positions that survive into R are positions
where BOTH HLLSets had bits set -- and those positions carry their TF
weight into the selection.

**This TF-based ranking is monotonically related to external ranking**
(like Google's PageRank), but it is NOT semantic relevance. Tokens that
co-occur in the same hash bucket are related by MurmurHash3 collision --
a deterministic, content-blind mapping. "Bank" and "tank" may hash nearby;
"bank" (financial) and "bank" (river) hash to the SAME position regardless
of context.

**This hash-location relevance is MORE stable than semantic relevance:**

- Hash functions are deterministic: the same token always lands in the same bucket
- Semantic meaning shifts with context ("bank" = finance vs. river)
- Hash-location never shifts -- it's a fixed property of the token's bytes
- For a ranking system that shouldn't drift with semantic fashion, this is an advantage, not a limitation

### Statistical Uniformity and the 1024-Dimensional Rank Vector

MurmurHash3 distributes tokens uniformly across the 32,768 bit positions.
Statistically, every position receives approximately the same number of tokens.
The variance is noise — ignorable.

**Consequence: each bit position (reg, zeros) has a token collection and TF
weight that is fixed FOR THE DURATION OF A SCAN -- the same weight in every
HLLSet where it's active during that scan. But different HLLSets activate
DIFFERENT positions. The permutation of active positions across 32,768 bits
IS what makes two HLLSets different. A register's rank = sum of TF weights
of its active positions.**

**Token collections evolve across scans.** A new ingest can bring new tokens
(added to the bucket, expanding the collection) and redundant old tokens
(already present, increasing their TF). The TF distribution shifts as the
system ingests data. The rank vector is a moving average of experience,
not a static embedding. A stable environment produces stable ranks;
a changing environment produces evolving ones.

### The LUT Is the Dynamic Layer -- HLLSets Are Immutable Pointers

**A bit position in an HLLSet is a reference to a (reg, zeros) bucket in the
TokenLUT.** The HLLSet stores the bitmask (which positions are active). The
LUT stores the actual token collections and their current TF values. The
HLLSet's rank is NOT stored in the HLLSet -- it's computed at query time:

```text
rank(HLLSet) = sum( LUT[reg][zeros].TF for all (reg, zeros) where bit == 1 )
```

**This is the mechanism that pushes HLLSets up or down in ranking without
touching them.** An HLLSet created years ago can rise in relevance today
not because the HLLSet changed (it's immutable) but because the LUT changed
around it:

- New slang emerges -> tokens at certain positions increase in TF
- Any HLLSet with bits set at those positions gets a rank boost
- Old HLLSets that "predicted" the new vocabulary rise in ranking
- HLLSets whose positions become stale (TF drops) quietly fall

```text
HLLSet (static, content-addressed)        LUT (dynamic, evolves with ingestion)
---------------                           -----------------------------------
bit[42] = 1  ----------------->            LUT[7][10].TF = 0.03  (was 0.01)
                                          ^ new slang made this position hot
bit[99] = 1  ----------------->            LUT[3][3].TF  = 0.00  (was 0.05)
                                          ^ tokens at this position fell out
                                          of use -- HLLSet's rank drops
```

**This is NOT the Forth dictionary reshuffling ranks (Step 7).** That's
experience-based learning. This is environment-based ranking: the world
changes, the LUT reflects it, and HLLSet ranks shift accordingly -- even
for HLLSets that haven't been "thought about" in years. The system doesn't
need to revisit old HLLSets to re-evaluate them. The LUT does it automatically.

It's the same separation as before:

- Tokenizer -> produces HLLSets (immutable)
- LUT -> tracks TF evolution (the world's vocabulary)
- Forth dictionary -> assigns learned ranks (experience)
- Materializer -> collects everything (passive observer)

The LUT is the bridge between the static HLLSet space and the changing world.

```text
Register rank = sum(TF(pos) for pos in register where bit == 1)
              ~ (number of active bits in register) * (average TF)
```

This means:

- **Comparison by individual coordinates makes no sense.** HLLSet A's register 42
  vs HLLSet B's register 42 in isolation — meaningless. Different bit patterns,
  same register index.
- **The 1024-register rank vector IS the semantic encoding.** The pattern of
  which registers have high rank and which have low, across all 1024 dimensions,
  encodes what the HLLSet "means." Individual registers are noise; the vector
  is signal.
- **This is analogous to word embeddings.** Individual dimensions of a word2vec
  vector are uninterpretable, but the 300-dimensional vector together encodes
  semantic relationships. Similarly, individual HLLSet registers carry no
  meaning, but the 1024-register rank vector is a stable hash-location-based
  semantic signature.
- **The R-link captures this naturally.** Intersection (bitwise AND) preserves
  only positions active in both HLLSets. The popcount of R is the count of
  surviving active positions. The surviving positions carry their register ranks
  — so the R-link weight is weighted by the semantic signature.

**No further aggregation makes sense.** Don't decompose the rank vector.
Don't compare individual coordinates. The 1024-vector IS the atom of
comparison — any finer granularity is just MurmurHash3 noise.

### Topological vs Scalar

| Property | BSS (scalar) | R-link (topological) |
| ---------- | ------------- | --------------------- |
| Output type | float in [0,1] | HLLSet (1024x32 bits) |
| Storable | No (ephemeral) | Yes (content-addressed) |
| Composable | No | Yes -- R can be intersected with C |
| FPGA-native | No (division) | Yes (AND + popcount) |
| Information | 1 number | 32,768 bits of relationship structure |

### Impact on the Architecture

This changes the Noether controller's decision logic from:

```text
if BSS(instinct, correct) < threshold:  # scalar comparison
```

to:

```text
R = instinct_hllset cap correct_hllset  # HLLSet
weight = popcount(R)                     # integer
if weight < threshold:                   # integer comparison
```

And the BSS matrix (Step 5) becomes an **R-link matrix** -- each cell is an
HLLSet key (r:\<sha1\>), not a float. The matrix IS a lattice of relationships,
not a table of numbers.

### HLLSet Prefix Taxonomy

In the Redis version, metadata HLLSets (h:\<sha1\>:m) provided provenance and
lattice structure. In the Forth hllang brain, **the Forth dictionary IS the
lattice definition.** Words are content-addressed — their names ARE SHA1
hashes. The dictionary grows (never shrinks) as new HLLSets and new words
are explicitly wired together.

**Human-readable names are prosthetic.** The notebook uses "brake", "drive",
"navigate" for demonstration. The real system speaks SHA1:

| Prefix | Type | Origin | Example |
| -------- | ------ | -------- | --------- |
| `h:` | HLLSet | Any operation (inscribe, union, intersect) | `h:3baa4cce...` |
| `o:` | Original | Tokenization — sourcing tokens from environment | `o:a1e7647e...` |
| `r:` | Retained | Intersection — the R-link | `r:4b38ac2b...` |
| `d:` | Departed | Difference — what left the lattice | `d:9d8ac7f6...` |
| `n:` | New | Difference — what entered the lattice | `n:c15d62bb...` |
| `v:` | View | A query perspective — filtered subset of the lattice | `v:7890ed68...` |

**We do NOT need `z:` for ordering/sequence.** Ordering is the actuator's
responsibility (Section 8). The HLLSet brain is orderless by design — tokens
are sets, HLLSets are bitmasks, the lattice is a partial order. The Forth
dictionary wires HLLSets together by content addressing, not by sequence.

A Forth word definition in the real system:

```text
o:abc123... → inscribed as HLLSet h:def456...
o:ghi789... → inscribed as HLLSet h:jkl012...
h:def456... cap h:jkl012... → r:mno345...  (R-link between them)
```

No names. Just SHA1 hashes wired by lattice operations. The dictionary IS
the explicit wiring — a DAG of content-addressed references. Human-readable
labels are a UX layer, not part of the architecture.

## 14. Novelty, Not Frequency — What Actually Prevents Thinking

**Scan frequency doesn't matter. Content change does.**

The tokenizer never overwrites. When a scan produces tokens already present
in the LUT, their TF doesn't change — the entry already exists, the count
increases incrementally but the LUT structure is stable. Same scan content
→ same active positions → same rank vector → same R-links → stable layers.

```text
Key insight: S(t) = S(t-1) produces ZERO system change.
The LUT is unchanged. Ranks are unchanged. The window is stable.
L0 (second) holds longer — and other layers have time to engage.
```

### Four Regimes (Content Change × Scan Rate)

| Regime | Rate | Content | Result |
| -------- | ------ | --------- | -------- |
| **Deep** | any | stable | All layers form. Environment is consistent. System understands. |
| **Adaptive** | slow | changing | Layers form between changes. System adapts deliberately. |
| **Reactive** | fast | changing | L0 (second) + L1 (minute) only. Change outruns window. |
| **Reflexive** | flood | chaotic | Perpetual L0 (second). No layer survives long enough to form. |

### The Real Trap

The scan-rate problem was half the story. Rapid scans with IDENTICAL content
(a camera pointed at a wall, a sensor reading the same temperature) don't
prevent understanding — they reinforce it. Each repeated scan confirms:
*nothing changed, the LUT is right, the ranks are right.*

The trap is rapid scans with NOVEL content — a feed that's always different.
The iPhone problem isn't that you're looking at it frequently. It's that
it shows you something NEW every time. The LUT keeps shifting. Ranks keep
reshuffling. No layer stabilizes long enough to become L1 (minute), let alone
L3 (day) or L4+ (week+).

```text
Stable environment + high scan rate = deep understanding (reinforcement)
Novel environment + high scan rate = perpetual instinct (fragmentation)
Stable environment + low scan rate = calm understanding (contemplation)
Novel environment + low scan rate = deliberate adaptation (learning)
```

### Why the Tokenizer's "Never Overwrite" Matters

Because the tokenizer never overwrites, stable content produces a stable LUT.
A stable LUT produces stable rank vectors. Stable rank vectors produce stable
R-link matrices. Stable R-link matrices allow layers to accumulate. The
architecture self-stabilizes when the environment is consistent — regardless
of how fast the scans arrive.

This is the structural guarantee: **the system is only as unstable as its
environment.** In a static world, it converges. In a changing world, it
adapts. In a chaotic world, it fragments. The system doesn't have an
internal failure mode — it has an environmental response curve.

---

## 16. Ashby's Homeostat — The Ancestral Architecture

Ross Ashby's homeostat (1948) was a machine built from four interconnected
units, each containing a magnet moving in a coil. When perturbed, the
system automatically sought a new stable configuration. It had no central
controller, no explicit goal, no semantic understanding. It simply
**reconfigured its internal connections until stability returned.**

This is the same architecture, seventy-eight years later, running on
FPGAs instead of electromagnets.

### Structural Parallels

| Homeostat (1948) | HLLSet System (2026) |
| ------------------ | --------------------- |
| Four interconnected units | Window of temporal layers (L0 (second)..L4+ (week+)) |
| Magnet position = unit state | HLLSet bitmask = layer state |
| Coil current = interconnection strength | R-link popcount = connection weight |
| Perturbation -> seek new equilibrium | S(t) != H(t-1) -> D/R/N decomposition |
| Reconfigure connections until stable | Forth reshuffles ranks until Delta -> 0 |
| No central controller | Noether reads R-link matrix, doesn't command |
| Stability is structural, not semantic | Understanding = LUT resolution, not meaning |

### Ultrastability

Ashby's key concept was **ultrastability**: the ability to find a new
stable configuration after any perturbation within the system's operating
range. The homeostat didn't "learn" the perturbation -- it learned a new
internal wiring that accommodated it.

Our equivalent: after S(t) introduces change, the D/R/N decomposition
identifies what departed, what remained, and what arrived. The Forth
dictionary reshuffles ranks. The window absorbs the perturbation across
its layers. The system converges to a new stable ranking. **It doesn't
understand the change. It accommodates it structurally.**

```text
Ashby:    perturbation -> magnet moves -> current changes -> new wiring -> stable
Our sys:  S(t) arrives -> D/R/N splits -> ranks reshuffle -> Delta -> 0   -> stable
```

### Requisite Variety

Ashby's law of requisite variety: a controller must have at least as many
states as the system it controls. A thermostat with two states (on/off)
cannot regulate a room to 0.1-degree precision.

Our equivalent (Section 14): it's not about scan *rate* -- it's about
scan *content change*. If the environment changes faster than the
window can absorb (novelty rate > window capacity), the system fragments
into perpetual L0 (second). The requisite variety is:

```text
window_size x LUT_vocabulary_size >= environment_novelty_rate
```

A window of 5 layers with 10,000 tokens per bucket can absorb more
environmental variety than a window of 3 layers with 100 tokens. The
homeostat had four units -- a fixed variety budget. Our window size is
configurable -- the variety budget scales with hardware.

### The Essential Difference

Ashby's homeostat sought *any* stable state. It didn't care which
configuration it landed in, as long as the needle stopped moving.
It had no memory -- it was a single-layer system operating purely
at the level of L0 (second).

Our system seeks a *particular* stable state -- the one where ranks
align with the accumulated experience (Forth dictionary) and the
current environment (LUT TF values). But the deeper difference is
**temporal depth.** The homeostat has no layers. Our window has W of them.

```text
window_size = 1  ->  Ashby's homeostat  ->  pure L0 (second)
window_size = 2  ->  can L1 (minute)        ->  compare now with recent past
window_size = 3  ->  can see L2 (hour)    ->  patterns across time
window_size = 4  ->  can hold L3 (day)   ->  long-term direction
window_size = 5  ->  can L4+ (week+)        ->  anticipate future
```

**Intelligence is a function of window depth.** No other ingredient is
needed. A system with a window of 1 is a homeostat -- reactive, memoryless,
purely instinctive. A system with a window of 5 can correct its instincts,
recognize patterns, maintain goals, and anticipate. The architecture is
identical. The difference is how much operational memory it carries.

**Rank adjustment IS adaptation. Adaptation IS intelligence.** The rank
vector is the fossil record of past adaptations -- every reshuffling is a
trace of environmental pressure. The Forth dictionary doesn't "think" in
a semantic sense. It adjusts ranks. That *is* thinking. Ashby proved
adaptation doesn't require intelligence. We close the loop: adaptation,
at sufficient depth, is indistinguishable from intelligence. The homeostat
was intelligent -- just at window_depth = 1.

**The process itself is ephemeral.** Adaptation is not a state the system
arrives at. It is a wave that passes through the layers and never stops.
There is no "adapted" configuration -- there is only the continuous motion
of ranks adjusting, perturbations propagating, layers shifting. The rank
vector is a snapshot of where the wave was. The sliding window IS the wave.

```text
S(t) enters at depth 0 (L0 (second))
  → perturbation ripples through L1 (minute) → L2 (hour) → L3 (day) → L4+ (week+)
  → each layer absorbs part of the perturbation
  → by depth W-1, the perturbation has dissipated (Delta -> 0)
  → but S(t+1) is already arriving at depth 0
  → the wave never stops
```

Intelligence is not a property of the system. It is not a property of any
single layer's state. It is a property of the *wave* -- how perturbations
propagate through the operational memory, how deeply they penetrate before
dissipating, how they interact with perturbations already in flight. The
layers are the medium. The rank adjustments are the wave. Intelligence is
the shape of the wave, and the wave is always passing, never staying.

This also explains the scan-rate/novelty problem in structural terms: an
environment that floods the system with novelty prevents layers from
accumulating. The window stays at depth 0. The system is reduced to
homeostat-level behavior -- pure instinct, no memory. The phone notification
stream doesn't make you less intelligent. It prevents your window from
deepening. You're running the same architecture as Ashby's 1948 machine:
react, stabilize, react again -- never accumulate.

**Intelligence is not a special capability. It is operational memory depth.
Any system with enough layers will exhibit increasingly intelligent behavior.**
The architecture doesn't change. The window just needs to be wider.

### Why This Connection Matters

Ashby built his homeostat to demonstrate that adaptive behavior doesn't
require intelligence -- it requires the right structural coupling between
components. Our architecture demonstrates the same thing at a different
scale: understanding doesn't require semantics -- it requires the right
lattice structure, the right ranking mechanism, and a window wide enough
to absorb the environment's variety.

The connection also validates the design. When a modern architecture
converges on the same principles as a 1948 cybernetic device, it's
usually because both discovered something fundamental about adaptive
systems -- not because one copied the other.

---

## 17. The Lattice as a Neural Network: Jacobian Dynamics

The rank vector of any HLLSet $H$ at a given position in the evolution is the
element-wise (Hadamard) product of the TF vector and the HLLSet's bitmask:

$$\text{rank}(H) = \text{TF} \odot \text{bitmask}(H)$$

This is structurally identical to a neuron: bitmask = weights, TF =
activation signal, rank = output. But unlike a traditional neural network,
the "weights" are content-addressed and immutable — they change only when a
new HLLSet is created. The "activation" (TF) is a monotonic CRDT updated
only by ingestion. The "output" (rank) is derived locally without backprop.

### The Jacobian of the Lattice

For a set of HLLSets $\{H_1, \ldots, H_k\}$ in the dictionary, the rank
matrix $R$ has entries $R_{ij} = \text{TF}_j \cdot \text{bitmask}_i(j)$.
The Jacobian describes how ranks change between evolution steps:

$$J_{ij} = \frac{\partial \, \text{rank}(H_i)}{\partial \, \text{TF}_j} =
\text{bitmask}_i(j)$$

Since $\text{bitmask}_i(j) \in \{0, 1\}$, the Jacobian is a binary matrix —
each entry is either 0 (HLLSet $i$ is insensitive to bit position $j$) or 1
(sensitive). There is no floating-point gradient. The "learning" is not
weight adjustment but **bitmask selection**: which HLLSets are sensitive to
which positions in the shared TF vector.

### Higher-Order Dynamics

The evolution of the lattice can be analyzed through successive Jacobians:

$$\begin{aligned}
J^{(1)} &= \frac{\partial R}{\partial \text{TF}} \quad\text{(first-order:
which HLLSets respond to which TF changes?)} \\[4pt]
J^{(2)} &= \frac{\partial^2 R}{\partial \text{TF}^2} =
\frac{\partial J^{(1)}}{\partial t} \quad\text{(second-order: how does
sensitivity itself change over evolution steps?)}
\end{aligned}$$

$J^{(1)}$ is computed from the current bitmasks — it answers "what does the
system attend to right now?" $J^{(2)}$ is computed from the difference
between consecutive first-order Jacobians — it answers "is the system's
attention shifting?"

### Connection to Classical Neural Networks

| Concept | Classical NN | HLLSet Lattice |
|---|---|---|
| Weights | Floating-point matrix $W$ | Binary bitmask(H) — content-addressed |
| Activation | $\sigma(Wx + b)$ | TF ⊙ bitmask(H) — CRDT-convergent |
| Forward pass | Matrix multiply + nonlinearity | Bitwise AND + popcount |
| Learning rule | Backpropagation ($\partial L/\partial W$) | Bitmask selection in dictionary |
| Gradient | Continuous $\partial L/\partial w_{ij}$ | Binary Jacobian $J_{ij} \in \{0, 1\}$ |
| Convergence | Local minima of loss surface | CRDT convergence (monotonic union) |

The HLLSet lattice is a neural network operating on binary weights with
CRDT-convergent activations and content-addressed topology. The Jacobian
formalism provides the analytical bridge — it allows us to reason about
lattice dynamics using the same mathematical language as deep learning,
while preserving the FPGA-native bitwise operations that make it fast.

### 17.1 Rank Derivatives and Noether Steering

Ranks are static measurements — a snapshot of importance at one moment. But the
**differences** between ranks across successive states carry the dynamic signal
that the Noether controller uses to steer the system.

#### Discrete Derivatives of Rank

For HLLSet $H(t)$ at time $t$, the rank vector $\\mathbf{R}(t)$ has components
$R_b(t)$ for each bit position $b = (r, tz)$. The first discrete derivative
(element-wise difference) is the **rank velocity**:

```math
\Delta\mathbf{R}(t) = \mathbf{R}(t) - \mathbf{R}(t-1)
```

This decomposes naturally via the D/R/N split. Each bit position $b$ falls into
exactly one of three categories at time $t$:

| Category | Condition | Rank contribution to ΔR |
|----------|-----------|------------------------|
| Retained $R$ | $b \in H(t) \cap H(t-1)$ | $R_b(t) - R_b(t-1)$ — change in rank of a persistent bit |
| New $N(t)$ | $b \in H(t) \setminus H(t-1)$ | $+R_b(t)$ — newly added rank |
| Departed $D(t-1)$ | $b \in H(t-1) \setminus H(t)$ | $-R_b(t-1)$ — lost rank |

The total rank flux decomposes as:

```math
\sum_b \Delta R_b(t) = \underbrace{\sum_{b \in R}(R_b(t) - R_b(t-1))}_{\text{rank drift}}
+ \underbrace{\sum_{b \in N(t)} R_b(t)}_{\text{rank influx}}
- \underbrace{\sum_{b \in D(t-1)} R_b(t-1)}_{\text{rank outflux}}
```

The second discrete derivative is the **rank acceleration**:

```math
\Delta^2\mathbf{R}(t) = \Delta\mathbf{R}(t) - \Delta\mathbf{R}(t-1)
= \mathbf{R}(t) - 2\mathbf{R}(t-1) + \mathbf{R}(t-2)
```

Equivalently, this is the element-wise difference between successive first-order
Jacobians (Section 17, Higher-Order Dynamics):

```math
J^{(2)}_{ij} = \frac{\partial J^{(1)}_{ij}}{\partial t}
= \text{bitmask}^{(t)}_i(j) - \text{bitmask}^{(t-1)}_i(j)
```

#### Noether Steering Reinterpreted

The Noether steering equation is a conservation law on bit **count**:

```math
|\text{card}(N(t)) - \text{card}(D(t-1))| \to 0
```

When this quantity approaches zero, the number of bits entering equals the
number departing — the system is in structural equilibrium.

We can reinterpret this in rank terms. The **rank-weighted steering** equation
replaces bit counts with rank sums:

```math
\left|\sum_{b \in N(t)} R_b(t) - \sum_{b \in D(t-1)} R_b(t-1)\right| \to 0
```

This is a stronger condition. Structural equilibrium ($|N| \approx |D|$) does
not guarantee rank equilibrium. The system could be exchanging low-rank bits
for high-rank bits at equal count — structurally stable but semantically
drifting. The rank-weighted form catches this: if departing bits carry more
cumulative rank than arriving bits, the system is losing signal strength even
while maintaining bit count parity.

**System acceleration as Noether's control signal.** The second derivative
$\Delta^2\mathbf{R}(t)$ measures how fast the rank flux itself is changing:

- $\Delta^2\mathbf{R} \approx 0$: constant flux — steady inflow/outflow, no steering needed
- $\Delta^2\mathbf{R} > 0$: flux accelerating (more rank entering, or less leaving) — expansion
- $\Delta^2\mathbf{R} < 0$: flux decelerating (less rank entering, or more leaving) — contraction

The Noether controller monitors $\Delta^2\mathbf{R}$ to decide whether to
intervene. A system with positive acceleration is absorbing novelty; a system
with negative acceleration is shedding stale structure. Neither is inherently
wrong — but persistent acceleration in either direction signals that the
sliding window (Ashby-style temporal depth) is not absorbing the environment's
variety. The controller can respond by widening the window, adjusting the R-link
gate threshold, or triggering reproduction.

#### Fisher-Like Matrix Across Temporal Layers

With multiple HLLSet presentations across temporal layers $L_0, L_1, \ldots,
L_6$, each layer $L_i$ has its own rank vector $\mathbf{R}^{(i)}$ and bitmask
$\mathbf{B}^{(i)}$ where $B^{(i)}_b \in \{0, 1\}$.

The **cross-layer co-occurrence matrix** $\\mathbf{F}$ (Fisher-like) captures
how bit positions co-activate across temporal scales:

```math
F_{bb'} = \sum_{i=0}^{6} B^{(i)}_b \cdot B^{(i)}_{b'} +
\sum_{t \in \text{history}} B^{(t)}_b \cdot B^{(t)}_{b'}
```

Since bitmasks are binary, $F_{bb'}$ simply counts how many layers (and
historical snapshots) have both bits $b$ and $b'$ set simultaneously.

**Interpretation:**

| Element | Meaning |
|---------|---------|
| $F_{bb}$ (diagonal) | How many layers contain bit $b$ — structural persistence across time scales |
| $F_{bb'}$ (off-diagonal) | How often bits $b$ and $b'$ co-occur — functional coupling |
| High $F_{bb}$, low off-diagonal | A "loner" bit — appears often but independently |
| Low $F_{bb}$, high $F_{bb'}$ | A "dependent" bit — appears rarely but always with $b'$ |
| Cluster of high $F_{bb'}$ | A functional module — bits that track each other across layers |

**Relation to classical Fisher information.** In statistics, the Fisher matrix

$\mathcal{I}(\theta)_{ij} = \mathbb{E}[(\partial_\theta \log p)(\partial_\theta \log p)]$

measures the sensitivity of observations to parameter changes. Here, the
"parameter" is the TF vector (which bits are active in the environment), the
"observation" is the layer bitmask (which bits are active in each temporal
layer), and the sensitivity is binary (0 or 1). The cross-layer co-occurrence
$F_{bb'}$ is the empirical Fisher information — it measures how much the
temporal structure reveals about which bits are functionally coupled.

**Noether steering with Fisher guidance.** When the controller detects
divergence ($|\text{card}(N) - \text{card}(D)| > \text{threshold}$), it
consults $\mathbf{F}$ to identify **which bits are driving the divergence**:

1. Compute the divergence vector $\mathbf{d} = \text{bitmask}(N) - \text{bitmask}(D)$
   — element-wise: +1 for newly set bits, -1 for departed bits, 0 for stable
2. Project through $\mathbf{F}$: $\mathbf{s} = \mathbf{F} \cdot \mathbf{d}$
3. $s_b$ measures the **systemic impact** of bit $b$'s change: a high $s_b$
   means bit $b$ is strongly coupled (via co-occurrence) to many other bits
   that also changed — it's not an isolated fluctuation, it's a systemic shift
4. The controller focuses steering on bits with $|s_b| > \text{threshold}$ —
   these are the structurally significant changes, not the noise

Without the Fisher matrix, the controller treats every bit's entry/exit as
independent. With it, the controller sees **coupled structural change** — a
group of bits moving together across multiple layers is a phase transition,
not random drift. The Fisher matrix is the lens that transforms raw bit-level
divergence into system-level awareness.

#### FPGA-Native Realization: Every Operation Bitwise

The entire dynamic analysis above — rank derivatives, Noether steering, Fisher
matrix, projection — must execute without floating-point arithmetic. This is
not an optimization constraint; it is the architectural premise established in
Section 12: BSS (float division) was rejected in favor of R-links (bitwise AND
+ popcount). The same principle applies to every operation in the rank algebra.

**Constraint.** Every value must be representable as a fixed-width integer
derivable from bitmask AND/OR/XOR and popcount. Division is permitted only by
powers of two (right shift).

**What survives, what must adapt:**

| Concept | Naive formulation | FPGA-native reformulation | Operations |
|---------|------------------|--------------------------|------------|
| Token rank $F$ | $\log(1 + \text{TF})$ (float) | $\text{TF}$ (identity) or $\lfloor\log_2(\text{TF})\rfloor$ (integer, via leading-zero count) | Integer load |
| Bit rank $G$ | Weighted mean (float div) | **Max** (bitwise OR of candidate ranks, then priority encoder) or **Sum** (integer addition) | CMP, ADD |
| Register rank $H$ | Mean over 32 bits (float div by 32) | **Sum** (integer addition, 32 terms) or **Max-pool** (tree of 31 CMPs) | ADD or CMP |
| HLLSet rank $K$ | PageRank (float iterative) | **Degree** (count of incident R-links = popcount of adjacency row) | POPCOUNT |
| Compound $L, M$ | Blended (float weights) | **Max** for union, **Min** for intersection (integer CMP) | CMP |
| Rank velocity $\Delta\mathbf{R}$ | Vector subtraction (float) | Element-wise integer subtraction of popcount-derived ranks | SUB |
| Rank acceleration $\Delta^2\mathbf{R}$ | Second difference (float) | Integer subtraction of integer differences | SUB, SUB |
| Noether steering | $\sum R_b$ (potentially float) | Integer sum of popcounts across $N$ and $D$ bitmasks | POPCOUNT, SUB, CMP |
| Fisher matrix $F_{bb'}$ | Floating co-occurrence | Popcount of (layer_mask[b] AND layer_mask[b']) across layers — integer count | AND, POPCOUNT, ADD |
| Projection $\mathbf{s} = \mathbf{F}\mathbf{d}$ | Float matrix-vector | Integer multiply-accumulate: $s_b = \sum_{b'} F_{bb'} \cdot d_{b'}$ where $d_{b'} \in \{-1, 0, +1\}$ and $F_{bb'}$ is popcount | MUL (integer), ADD |

**Key simplification.** Every entry in the Fisher matrix $F_{bb'}$ is a
popcount — an integer between 0 and the number of layers (7 for the base
pyramid, plus however many historical snapshots are retained). Every rank
value feeding into derivatives is an integer (popcount or sum of popcounts).
No division, no logarithm, no exponential. The entire dynamic analysis
reduces to AND, OR, XOR, POPCOUNT, integer ADD/SUB/CMP, and integer MUL
(only for the Fisher projection — and even that can be replaced by repeated
addition since $d_{b'} \in \{-1, 0, +1\}$).

**The R-link precedent.** Section 12 replaced BSS (scalar float, requiring
division) with R-links (HLLSet, requiring only AND + popcount). This was not
an optimization — it was a category change from *measurement* to *object*. An
R-link is storable, composable, and content-addressed. The same category change
applies here: the Fisher matrix is not a table of floating similarity scores —
it is a matrix of integer co-occurrence counts, each row storable as a sparse
bitmask, the whole matrix navigable by the same lattice operations as any other
HLLSet structure. **The dynamic analysis is not a separate computation layered
on top of the lattice. It IS the lattice, viewed through a temporal lens.**

---

## 18. The Lattice as an Optimization Surface

A HLLSet is a **measurement**, not a context container. A single HLLSet
carries no inherent significance — only the collection matters. The lattice
is the population; each HLLSet is one sample from it. The task is not to
find the best HLLSet. The task is to find the best **cover**.

### Minimal Cover with Maximal Rank

Given the lattice state $H_{\text{system}} = \bigcup L_i$, and the full set of
HLLSets $L = \{H \mid H \text{ is any HLLSet in the system}\}$ — including
compounds (unions, intersections, R-links, layer aggregates):

$$\text{Find } C \subseteq L \text{ such that:}$$
$$\bigcup_{H \in C} H \supseteq H_{\text{system}} \quad\text{(cover)}$$
$$\sum_{H \in C} \text{rank}(H) \text{ is maximized} \quad\text{(quality)}$$
$$|C| \text{ is minimized} \quad\text{(parsimony)}$$

Compounds are included in the search because they capture relationships
(unions, intersections, R-links) that may explain the state better than
individual originals. But every selected compound traces back to its
original $o:$ HLLSets via the provenance tree. The cover is expressed in
compounds; the explanation is expressed in originals.

This is a PCA-like factorization of the lattice: find the low-rank
approximation that explains the observed state with the fewest, highest-rank
components. But unlike PCA, we don't need the optimal solution. We need the
**direction** toward it.

### Direction, Not Destination

At each evolution step, the system asks:

```text
  Given current lattice L and scan S(t):
    1. Which HLLSet, if added to the cover, would most improve coverage/rank?
    2. Which HLLSet, if removed, would least degrade it?
    3. Move one step in that direction.
```

The system never arrives at the optimal cover — the lattice keeps evolving.
But the gradient of the objective function (coverage × rank / |C|) points
toward better covers, and each step moves along that gradient.

### Temperature: Exploration vs Exploitation

A temperature parameter $T$ controls how far from the current optimum the
system is willing to search:

```text
  T → 0  (cold):   only high-rank HLLSets. Stick to what's proven.
                    Fast convergence, risk of rank bubbles.
                    Equivalent to greedy exploitation.

  T → ∞  (hot):    any HLLSet can be selected. Explore the dictionary.
                    Slow convergence, avoids local optima.
                    Equivalent to random exploration.

  T(t):             annealing schedule. Start hot (explore), cool down
                    (exploit) as the system stabilizes.
```

The Noether controller IS the temperature scheduler. When cross-layer BSS
is stable (high coherence), $T$ decreases — the system exploits. When BSS
diverges (novelty detected), $T$ increases — the system explores.

### Unified View

| Component | Optimization Role |
|---|---|
| Full lattice L | Search space — compounds that capture relationships |
| Rank | Objective function — quality of each component |
| R-link gate | Candidate filter — which components are eligible? |
| TF vector | Gradient signal — which bit positions are active? |
| Temperature $T$ | Exploration/exploitation trade-off |
| Noether controller | Temperature scheduler — driven by cross-layer BSS |
| Evolution step | One gradient step toward better cover |
| H_system = ∪ L_i | Covering target — the state to be explained |

What looks like "a Forth dictionary that learns by rank adjustment" is
actually a **set cover optimization problem solved by gradient-directed
search with temperature-controlled exploration**. The same mechanisms.
The same primitives. A different lens — and a more powerful one.

---

## The Lattice as Holographic Memory

The lattice top $H_{\text{system}} = \bigcup L_i$ implicitly contains every
HLLSet ever observed — any HLLSet in the lattice is a subset of the top.
They differ only in scope, not in content. The lattice top IS the complete
record.

The ordered sequence of TF vectors $[\text{TF}_0, \text{TF}_1, \ldots,
\text{TF}_6]$, plus historical snapshots, forms a temporal stack. By
applying a specific TF from this stack to the current lattice top, we
recover an approximation of the lattice state at that past moment:

```math
\text{past\_state}(t) \approx H_{\text{system}}(\text{now}) \odot
\text{TF}_{\text{stack}}[t]
```

Where $\odot$ means: project each HLLSet in the current lattice through the
TF vector at time $t$ to derive its approximate rank at time $t$. This is
not a perfect reconstruction — the lattice top may contain HLLSets created
after time $t$ — but the TF vector acts as a **time lens**: it selects which
bit positions were active at $t$, suppressing everything that appeared later.

### Practical Implication: Temporal Compression

You don't need to store every historical lattice snapshot. Store:

1. The current lattice top (one HLLSet — 4KB)
2. The TF stack (one vector per time step — 262KB each, compressible)

From these, approximate any past state. Combined with the original $o:$
HLLSets in IPFS (never lost, always retrievable by CID), the system can
reconstruct its own history from minimal storage.

### The Holographic Principle

Every part of the lattice contains information about the whole:

```text
  Lattice top     = complete record (union of all observations)
  TF stack        = time lens (which bits mattered when?)
  Past state      = top ⊙ TF[t] (view the top through a past lens)
  Original o:     = ground truth (retrievable from IPFS on demand)
```

This is holographic memory: the whole is encoded in the part. The TF
vector selects which slice of the whole you see. The lattice top never
shrinks — it's monotonic — so every past view is still accessible through
the right lens.

---

## 19. Self-Ingestion — The Codebase as Lattice Input

The architecture monitors external data streams via the tokenizer pipeline.
But the system's own source code is also a data stream — it evolves through
edits, refactors, and feature additions. **The codebase itself can be ingested
into the HLLSet lattice**, making the project's development history observable
through the same rank, derivative, and Fisher machinery that monitors external
data.

### Design

**Trigger: git commit.** Rather than ingesting on every keystroke (too noisy)
or every file save (too frequent), ingestion fires on `git commit`. Each
committed file is treated as a token stream: the file's content is tokenized
into an HLLSet, content-addressed, and stored. The commit becomes a lattice
event with its own D/R/N decomposition — which files were added (N), modified
(retained with drift), or deleted (D).

**Idempotency.** Ingestion is idempotent by design — re-running the ingest
script on the same files produces the same HLLSet fingerprints. This means
the init ingestion (backfilling the existing codebase) and incremental
ingestion (new commits) use the same pipeline with no special cases.

**Per-file granularity.** Each source file becomes one HLLSet. This enables:

| Query | Mechanism |
|-------|-----------|
| Which files changed most this sprint? | Rank velocity Δ²R over commit history |
| Which files tend to be edited together? | Fisher matrix: F(file_a, file_b) across commits |
| Is the codebase in a refactoring or stable phase? | Noether steering: |N| vs |D| across commits |
| Which modules are "hot" right now? | Observable mask O(θ) over file ranks |
| How similar is this commit to the last release? | BSS between current and tagged commit HLLSets |

**The commit chain becomes a navigable DAG.** Each commit HLLSet links to its
parent via the R-link (intersection of file sets between commits). The D/R/N
decomposition at commit granularity tracks which files entered, which were
modified, and which were removed. The temporal pyramid maps naturally: commits
within an hour aggregate into L2, within a day into L3, across a sprint into L4.

### Implementation Sketch

```text
Git hook: post-commit → ingest script → hllset CLI → lattice storage

For each file in the commit:
  1. Tokenize file content → HLLSet (content-addressed by SHA1)
  2. Store HLLSet in ipfrs-native storage (key: h:<sha1>)
  3. Record commit metadata: {ts, files: [cid, ...], parent: cid}
  4. Update TF vector (bit-level TF for the file's token positions)

Init ingestion (once, for existing codebase):
  for each source file in the repository:
    tokenize → HLLSet → store
  build initial commit HLLSet = union of all file HLLSets
```

### Terminal Interface: Textual

[Textual](https://textual.textualize.io/) is a Python framework for building
rich terminal user interfaces with reactive widgets, CSS-based styling, and
async event handling. It is the natural choice for the commit-ingest workflow
and lattice monitoring dashboard.

**Why Textual fits this project:**

| Requirement | Textual capability |
|-------------|-------------------|
| Diff-like view of what's being ingested | `RichLog` + `Tree` widgets for file lists with syntax-highlighted diffs |
| Real-time rank/Fisher display | Reactive `DataTable` bound to lattice state queries |
| Interactive threshold adjustment | `Slider` widgets for θ, bit/rank thresholds |
| Multi-panel dashboard | `Grid` layout: files panel, rank panel, Noether panel, commit history |
| Keyboard-driven workflow | Built-in keybindings, custom `Binding` for commit/abort |
| Async I/O | `textual run` is async-first — lattice queries don't block the UI |

**Proposed workflow for `hllset commit`:**

```text
┌─ Commit Ingest ───────────────────────────────────────────┐
│  ┌─ Staged Files ────────┐  ┌─ HLLSet Preview ──────────┐ │
│  │ src/main.rs    [+12]  │  │ h:a3f82c... (main.rs)     │ │
│  │ src/lib.rs     [ -3]  │  │ h:b7e91d... (lib.rs)      │ │
│  │ _DOCS/foo.md   [+45]  │  │ h:c15d62... (foo.md)      │ │
│  └───────────────────────┘  └───────────────────────────┘ │
│  ┌─ Noether Check ──────────────────────────────────────┐ │
│  │ |N|=2, |D|=0 → divergence=2  Δ²R=+3 (expansion)      │ │
│  │ Rank flux: +15 (net new content)                     │ │
│  └──────────────────────────────────────────────────────┘ │
│  ┌─ Actions ────────────────────────────────────────────┐ │
│  │ [C]ommit & Ingest  [A]bort  [V]iew diff  [R]efresh   │ │
│  └──────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────┘
```

The Textual interface sits between the developer and `git commit`, adding
HLLSet ingestion as a transparent step. The developer sees what will be
ingested, how it affects the lattice state, and whether the Noether
controller flags anything unusual — all before the commit reaches GitHub.

### Self-Referential Closure

This closes the final loop: the system that monitors external data also
monitors its own evolution. The same rank algebra that tracks token
frequency in sensor streams tracks code churn in the repository. The
same Fisher matrix that detects coupled bit positions detects coupled
source files. The same Noether controller that flags environmental
divergence flags refactoring storms.

The architecture is **self-describing** — the lattice that records the
project's history is built from the project's own code, ingested through
the project's own pipeline.

### 19.1 LLM Context Views — Bridging Prompts to Code

The self-ingestion pipeline stores every source file as an HLLSet. But raw code
tokens are a poor match for natural language queries — a user prompt "connect to
the database" needs to find `fn open_connection()`, not token-level overlap with
"connect" and "database" scattered across unrelated files.

**Solution: per-directory `llms.txt` + folder views.**

Each code directory gets an `llms.txt` file — a human-written annotation that
describes what the code in that directory does, in the same semantic space as
user prompts. The `llms.txt` is ingested into the lattice with a dedicated
prefix `l:` (LLM context), distinct from `h:` (code HLLSets) and `v:` (views).

A **folder view** is a union HLLSet that aggregates all code files in a
directory plus its `llms.txt`. Views carry the `v:` prefix.

```text
crates/hllset-storage/
├── llms.txt               → l:<sha1>   LLM context (human-written annotation)
├── src/
│   ├── lib.rs             → h:<sha1>   code
│   ├── storage.rs         → h:<sha1>   code
│   ├── ipfs.rs            → h:<sha1>   code
│   └── ...
└── [view]                 → v:<sha1>   union(lib, storage, ipfs, ..., llms.txt)
```

**Query flow for prompt → code matching:**

```text
1. User prompt: "connect to the database"
2. tokenize(prompt) → HLLSet P
3. Phase 1 — LLM context scan:
     for each l:<sha1> in lattice:
         τ = BSS(P, l:llms)
     if τ > 0.5: that directory is semantically relevant
4. Phase 2 — Folder view refinement:
     for top-K directories from Phase 1:
         τ = BSS(P, v:folder_view)
     return top matches with their code files
5. AI coder receives: matched llms.txt content + matched code files + prompt
```

**Why this works:**

| Property | Mechanism |
|----------|-----------|
| Semantic gap bridged | `llms.txt` is written in natural language, same space as prompts |
| Maintainable | One `llms.txt` per directory, not per file |
| Discoverable | `l:` prefix enables targeted BSS scans without deserializing code HLLSets |
| Self-reinforcing | New code + its `llms.txt` become instantly searchable via the commit hook |
| Privacy-preserving | `llms.txt` describes what the code does, not the proprietary logic itself |

**The `l:` prefix taxonomy.** Extended from the existing namespace:

| Prefix | Type | Origin | Use case |
|--------|------|--------|----------|
| `h:` | HLLSet | Any operation | General content-addressed data |
| `o:` | Original | Tokenization | Source-of-truth tokens from environment |
| `r:` | Retained | Intersection | R-link — structural relationship |
| `d:` | Departed | Difference | What left the lattice |
| `n:` | New | Difference | What entered the lattice |
| `v:` | View | Union aggregation | Folder-level aggregate of code + context |
| `l:` | LLM context | Human annotation | Semantic bridge between prompts and code |

**Self-referential closure.** The architecture that ingests code for
self-monitoring also ingests `llms.txt` for self-description. When a developer
writes an `llms.txt` and commits, the post-commit hook ingests it as `l:<sha1>`
and updates the folder view `v:<sha1>`. The lattice that records the project's
history also becomes the index that makes the project's code discoverable by
natural language — through the same pipeline, with the same operations, stored
in the same content-addressed space.

**Design principle: unions make joins free.**

The two-phase query flow (l: → v: → h:) works without graph traversal because
every aggregation is pre-computed at ingestion time and stored as a cheap `v:`
HLLSet. A folder view is one OR over its constituent files. A commit view is one
OR over changed files. Both are idempotent — re-computing produces the same
`v:<sha1>`. The result: queries that would require DAG traversal, index joins,
and set intersection in a traditional system reduce to a single BSS call.

```text
Traditional:
  "files touched in storage AND mesh this sprint?"
    → traverse commit DAG → diff each commit → join file lists → intersect

HLLSet:
  "files touched in storage AND mesh this sprint?"
    → BSS(v:commit_window, v:storage_folder ∪ v:mesh_folder)
    → one AND + one popcount
```

No embeddings. No vector database. No SPARQL endpoint. No graph traversal.
Every aggregation is pre-computed once (OR), every query is one AND+popcount.
The same two operations the FPGA already runs for everything else.

### 19.2 Implementation Notes

1. **Existing documentation is unchanged.** The `_DOCS/` tree, `README.md`, and
   inline docstrings remain the primary human documentation. `llms.txt` files
   are a separate semantic index layer -- they annotate, they do not replace.

2. **Auto-generated `llms.txt` from doc comments.** `llms.txt` files are not
   hand-maintained. The post-commit hook extracts doc comments from changed files
   and regenerates the containing folder's `llms.txt` automatically:

   ```text
   For each changed .rs file:
       extract //! module doc → "[description](relative/path)"
       extract /// pub fn docs → "[description](relative/path)"
   For each changed .md file:
       extract first paragraph → "[description](relative/path)"
   ```

   Format: `[one-sentence description](path/to/file)` — descriptions come from
   the code itself, paths are relative. The same commit that changes code docs
   also regenerates the semantic index. Zero human maintenance.

   ```text
   crates/hllset-storage/llms.txt (auto-generated):
       [Content-addressed storage for HLLSets](src/)
       [ipfrs-native storage backend backed by sled](src/ipfs.rs)
       [Sync storage trait for HLLSet data](src/storage.rs)
       ...
   ```

3. **Auto-update on self-reflection commit.** When a commit includes code or doc
   changes, the post-commit hook: (a) regenerates affected `llms.txt` files,
   (b) ingests them as `l:<sha1>`, (c) recomputes any affected folder views
   `v:<sha1>` = union(all files in dir + llms.txt). This keeps the semantic
   index in lockstep with the codebase — no manual step, no drift.

4. **Designated persistent storage.** The lattice metadata lives in
   `.hllset_lattice/metadata.json` (not committed to git). The ipfrs-native
   block storage (sled database) lives in `.hllset_lattice/storage/`. Both
   paths are in `.gitignore`. The storage directory persists across CLI
   sessions, making HLLSets from previous ingestions available to the running
   application without re-ingestion.

   ```text
   .hllset_lattice/
   +-- metadata.json         # file -> HLLSet key index, commit history
   +-- storage/              # sled database (ipfrs-native block store)
   ```

---

### 19.3 Unified Development Interface — DeepCode + HLLSet + Git

The current self-reflection architecture has two inconsistencies:
(1) it requires two terminals — one for DeepCode interaction, one for the
commit TUI; (2) self-reflection is implemented as a special case with a
dedicated `metadata.json`, when the standard lattice evolution equation
already handles it.

The redesigned interface unifies everything in a single Textual terminal
with three frames, and treats self-reflection as a standard ingest pipeline
with one exception: collaboration summaries are stored as files in IPFS
because the system owns the data it generates.

#### Design

```text
+-- hllset-dev TUI ----------------------------------------------+
| +- Prompt ───────────────────────────────────────────────────+ |
| | > Implement the auth module                                | |
| +------------------------------------------------------------+ |
| +- Response ─────────────────────────────────────────────────+ |
| | DeepCode: I'll create crates/auth/ with the following...   | |
| | [code blocks appear here]                                  | |
| +------------------------------------------------------------+ |
| +- Files ───────┬────────────────────────────────────────────+ |
| | St  File      │  Actions                                   | |
| | M   auth.rs   │  [Commit All] [Generate Summary] [Refresh] | |
| | A   auth.toml │                                            | |
| +───────────────┴────────────────────────────────────────────+ |
+----------------------------------------------------------------+
```

**Frame 1: Prompt.** User types prompts in plain English. The TUI sends them
to DeepCode (via CLI subprocess or API) and captures the response.

**Frame 2: Response.** DeepCode's output appears here — code blocks, explanations,
file listings. The TUI parses the response to identify which files DeepCode
created or modified. These populate Frame 3 automatically.

**Frame 3: Files.** Lists all changed files (staged + unstaged + untracked)
with status indicators. When DeepCode generates code, files appear here
immediately (detected via `inotify` or polling). Command buttons:

| Button | Action |
|--------|--------|
| **Generate Summary** | Prompts DeepCode: "Summarize this development session: what was requested, what was built, what files changed, what decisions were made." The summary becomes: (a) an HLLSet `l:<sha1>` for lattice search, (b) a file stored in IPFS with CID recorded in the commit. |
| **Commit All** | Runs `git add -A`, commits with the summary as the message body, triggers post-commit ingest. |
| **Refresh** | Re-scans for changed files. |

#### Self-Reflection as Standard Ingest

The evolution equation is unchanged:

```text
H(t) = H( S(t), H(t-1), D(t-1), R(t-1), N(t) )
```

For self-reflection, S(t) includes:

| Component | Source | Storage |
|-----------|--------|---------|
| Source code files (changed/added) | DeepCode output → git working tree | `h:<sha1>` in lattice, git push to GitHub |
| Collaboration summary | Generated by DeepCode on "Generate Summary" | `l:<sha1>` in lattice, file in IPFS |
| Prompt history | User prompts from Frame 1 | `h:<sha1>` in lattice (optional) |

The summary is the exception to normal IICA policy. Normally the system
fingerprints external data but does not store originals. Summaries are
system-generated — the system OWNS them — so they are persisted both as
HLLSet fingerprints and as full-text files in IPFS.

**No special metadata file.** The `.hllset_lattice/metadata.json` is
eliminated. The lattice state H(t) IS the metadata. The commit chain,
folder views, and LLM context HLLSets collectively encode everything that
`metadata.json` previously tracked — which files were ingested, when,
with which CIDs. Querying the lattice is querying the metadata.

#### Workflow

```text
1. User types prompt in Frame 1
2. DeepCode processes, response appears in Frame 2
3. Generated/modified files appear in Frame 3 (auto-detected)
4. User reviews, iterates (steps 1-3 repeat)
5. User clicks "Generate Summary"
   → DeepCode produces collaboration summary
   → Summary HLLSet: tokenize(summary) → l:<sha1>
   → Summary file: stored to IPFS, CID recorded
6. User clicks "Commit All"
   → git add -A
   → git commit -m "<summary>"
   → post-commit hook:
       * ingests all changed files → h:<sha1>
       * ingests summary → l:<sha1>
       * recomputes affected folder views → v:<sha1>
       * updates D/R/N for this commit in lattice
   → git push to GitHub
```

#### Implementation Notes

- DeepCode communication: the TUI spawns `deepcode` as a subprocess with
  the prompt as input, captures stdout as the response. Same model as the
  current `hllset` CLI subprocess calls in `ingest.py`.
- File detection: `git diff --name-status` + `git ls-files --others` on
  each refresh. DeepCode's file writes are picked up within the refresh
  interval (polling) or via `watchfiles`/`inotify` (push).
- Summary format: generated by DeepCode on request, includes:
  - What was requested
  - What was built (files, crates, functions)
  - Decisions made during development
  - Open questions or future work
- IPFS storage: `ipfrs add <summary_file>` → CID. The CID is recorded in
  the commit metadata and stored alongside the summary HLLSet.

### 19.4 The Trait-Boundary Principle — Why Redis Was 150 Lines

The `RedisStorage` backend took 150 lines of Rust to implement the `Storage`
trait — and then the entire framework (Lua runtime, materializer, DuckDB LUT,
ingest pipeline, mesh nodes, five-level rank algebra) worked against Redis
without a single change outside the crate.

```text
                ┌────────────────────────────────────┐
                │   hllset-dsl (Lua runtime)         │
                │   hllset-materialize (LUT engine)  │
                │   hllset-ranks (five-level algebra)│
                │   hllset-mesh (pub/sub bus)        │
                │   scripts/ingest.py (pipeline)     │
                └────────────┬───────────────────────┘
                             │ &dyn Storage
                ┌────────────┴───────────────────────┐
                │  trait Storage {                   │
                │    store, load, exists, delete,    │
                │    list, pin, unpin, gc            │
                │  }                                 │
                └─┬──────────┬──────────────┬────────┘
                  │          │              │
              MemoryStorage  IpfrsNative   RedisStorage
              (dev/test)     (sled/local)  (enterprise)
```

**Principle:** Isolate infrastructure behind a minimal trait boundary.
Every backend implements the same 6 methods. Everything above the trait
is pure domain logic in Rust, pointer to `Forge`, and Python scripts —
none of it knows or cares where bytes live.

**Consequence:** When `RedisStorage::connect("redis://...")` passed its
first PING, the entire HLLSet lattice stack was already running against
it. 212 tests. 0 failures. No integration work. Because the trait boundary
had already been paid for.

This is the same principle that will enable the LUT ↔ RediSearch bridge
and the graph ops ↔ RedisGraph bridge. Each is a new crate implementing
a trait. Each lights up the entire framework on connection.

---

## 20. Graph Engine Transition Path — RedisGraph + HLLSet

> **Status:** Architectural exploration — not yet implemented
> **Purpose:** Reference architecture for future Enterprise-grade transition

This section maps the path from the current demo-level HLLSet lattice to a
production graph engine built on RedisGraph internals. The goal is not to
replace RedisGraph but to embed HLLSet primitives as its native storage and
index layer, producing a graph database where every node and edge is
content-addressed, temporally indexed, and FPGA-native.

### 20.1 The Core Insight

RedisGraph represents graphs as **sparse adjacency matrices** manipulated
through GraphBLAS operations (SpMV, SpGEMM, element-wise multiply). The
HLLSet lattice represents relationships as **R-link HLLSets** manipulated
through AND+popcount. These are the same operation at different levels:

```text
GraphBLAS SpMV:    y = A * x      (float matrix-vector multiply)
HLLSet neighbour:  score = |query AND node|  (integer AND + popcount)
```

Every GraphBLAS operation has an HLLSet equivalent. The transition maps
the former onto the latter, replacing floating-point linear algebra with
integer bitwise operations -- trading precision for speed, determinism,
and FPGA compatibility.

### 20.2 Phase 1: Embedded Index (Immediate — weeks)

**Goal:** Use HLLSet as an external index alongside RedisGraph, without
modifying RedisGraph internals.

```text
+-------------------+     +---------------------------+
|   RedisGraph      |     |   HLLSet Lattice          |
|   (unchanged)     |     |   (companion index)       |
|                   |     |                           |
|  Nodes: UUID      |<--->|  Nodes: h:<sha1> (4KB)    |
|  Edges: (u,v,w)   |<--->|  Edges: r:<sha1> (4KB)    |
|  Queries: Cypher  |     |  Queries: BSS + popcount  |
+-------------------+     +---------------------------+
         |                          |
         +---------- API -----------+
         |  Node CID lookup         |
         |  BSS similarity search   |
         |  Temporal layer queries  |
         +--------------------------+
```

**What this enables (enterprise use cases):**

- **Fast pre-filtering.** Before a Cypher traversal, BSS against HLLSet index
  eliminates 90%+ of irrelevant nodes. "Find all customers similar to this support
  ticket" becomes one BSS call instead of a k-NN over embeddings.

- **Temporal queries.** "Show me the supply chain graph as it was at 14:30
  yesterday" -- query the L2 (hour) layer of the temporal pyramid, get a
  sub-graph of nodes active in that window.

- **Audit trail.** Every node mutation produces a new HLLSet with a CID. The
  commit chain is an immutable history. Compliance queries are BSS against
  historical snapshots.

**Implementation:**

1. `redis-hllset` Redis module: `HLLSET.ADD key content → CID`, `HLLSET.BSS cid1 cid2 → score`
2. Client library maps RedisGraph node UUIDs → HLLSet CIDs
3. Pre-filter pipeline: BSS query → filter candidate nodes → Cypher traversal

**Enterprise readiness at this phase:**
- Read-only integration, no RedisGraph changes
- HLLSet index can be rebuilt from scratch (idempotent)
- Temporal pyramid provides audit trail for compliance
- BSS pre-filtering reduces Cypher query latency by ~90% for similarity searches

### 20.3 Phase 2: Native Node Storage (Short-term — months)

**Goal:** Replace RedisGraph's internal node property storage with HLLSet
bitmasks. Nodes become content-addressed; properties become bit positions.

```text
RedisGraph node (current):
  { id: uuid, labels: ["Customer"], properties: {name: "Acme", tier: 3} }

RedisGraph node (HLLSet-native):
  { cid: "h:a3f82c...", labels: bits 0..7 of register 0, properties: bits 8.. }
```

**What this changes:**

- **Node identity is content.** Two nodes with identical properties produce
  the same CID. Deduplication is automatic -- no application-level logic needed.
- **Property lookup is bit test.** `has_bit(reg, prop_offset)` replaces hash
  table lookup. Single-cycle on FPGA.
- **Schema evolution is idempotent.** Adding a property = setting a new bit
  in the register array. Old nodes don't change; new nodes include the bit.
  No migration scripts.

**RedisGraph fork modifications:**

| Component | Change |
|-----------|--------|
| `GraphEntity` (node) | `node_id` field becomes `cid: [u8; 20]` (SHA1) |
| `GraphEntity_AddProperty` | Writes to HLLSet bitmask via `HLLSet::add_property(offset, value)` |
| `GraphEntity_GetProperty` | Reads from HLLSet bitmask via `HLLSet::has_bit(reg, tz)` |
| `Graph_AddNode` | Calls `HLLSet::from_properties()` → generates CID |
| `Graph_DeleteNode` | No-op for HLLSet (immutable); mask from observable set |

**Enterprise readiness at this phase:**
- Property operations are O(1) bit tests, not hash lookups
- Node identity is cryptographically verifiable (SHA1 of content)
- Zero-cost deduplication across the entire graph
- Schema changes are backward-compatible by construction

### 20.4 Phase 3: Sparse Matrix as Fisher Matrix (Medium-term — quarters)

**Goal:** Replace RedisGraph's internal adjacency matrix with the Fisher
coupling matrix. Edge weights become popcount(R-link). GraphBLAS operations
become AND+popcount.

```text
RedisGraph adjacency (current):
  A[u][v] = f64 weight  (e.g., 0.73)

HLLSet adjacency (native):
  A[u][v] = popcount(R-link(u, v))  (integer, e.g., 1247)
  R-link(u, v) = u.hllset AND v.hllset  (stored as r:<sha1>)
```

**GraphBLAS → HLLSet operation mapping:**

| GraphBLAS | Current (float) | HLLSet-native (integer) |
|-----------|----------------|-------------------------|
| SpMV: y = A * x | Float mul+add | `for each neighbour: y += popcount(R AND x)` |
| SpGEMM: C = A * B | Float mul+add on indices | `R(A∩B entries) = R-links of intersecting neighbours` |
| eWiseMult | Float multiply | `A AND B` (bitwise, single-cycle) |
| eWiseAdd | Float add | `A OR B` (bitwise, single-cycle) |
| Reduce | Float sum over row | `row_popcount = sum of R-link popcounts` |

**Why this matters for enterprise:**

- **Deterministic.** `A AND B` always produces the same result. Float accumulation
  varies by order (non-associative rounding). Audit passes every time.
- **Storable.** An R-link is an HLLSet with a CID. You can persist it, query it,
  and traverse it later. Raw float weights are ephemeral.
- **Temporal.** Every R-link carries a timestamp (when was this edge created?).
  The time pyramid gives each edge a temporal address. "Show me the graph before
  the merger" = apply a TF time lens to the adjacency matrix.
- **Bounded memory.** Each edge is exactly 4KB regardless of complexity. No
  unbounded property lists per edge.

**RedisGraph fork modifications:**

| Component | Change |
|-----------|--------|
| `Graph_ConnectNodes` | Computes R-link = `src AND dst`, stores as `r:<sha1>` |
| `Graph_GetEdgeWeight` | Returns `popcount(R-link)` |
| `RG_Matrix_extractRow` | Returns bitmask of connected CIDs + their R-link popcounts |
| `TupleIter_next` (tuple iterator) | Iterates R-link list instead of float matrix row |
| Delta matrix ΔA | Computed as D/R/N between successive adjacency snapshots |

### 20.5 Phase 4: Graph as Lattice (Long-term)

**Goal:** The graph IS the lattice. Node operations (union, intersection,
difference) produce new nodes. The graph evolves through D/R/N decomposition
just like the temporal pyramid. Every Cypher query IS a BSS operation.

```text
Cypher: MATCH (a:Customer)-[:BOUGHT]->(p:Product) RETURN p

HLLSet: BSS(h:customer_pattern, v:all_products)
        WHERE popcount(R-link(customer, product)) > threshold
```

The Cypher query planner becomes a BSS optimizer. Pattern matching becomes
bitwise AND. Graph traversal becomes R-link chain navigation with latency
guarantees from the time pyramid.

### 20.6 What Gets Us from Demo to Enterprise

| Axis | Demo (current) | Enterprise (target) |
|------|---------------|---------------------|
| **Scale** | Tens of HLLSets, single process | Billions of nodes, distributed cluster |
| **Persistence** | In-memory or local sled | Distributed ipfrs storage with replication |
| **Query** | Python scripts, Forth REPL | Cypher/SPARQL with BSS optimizer |
| **Temporal** | Manual layer inspection | Automatic pyramid maintenance, time-lens queries |
| **Mutation rate** | Batch ingestion, occasional commits | Streaming, millions of graph ops/second |
| **Consistency** | Single-node, Noether-based | CRDT-convergent, D/R/N delta propagation |
| **Monitoring** | Print statements | Prometheus metrics, Fisher-based anomaly detection |
| **Security** | None | Content-addressed audit trail, CID-based access control |
| **Deployment** | cargo run | Kubernetes operator, Redis cluster with HLLSet module |

**The critical path to enterprise:**

1. **Scale the ingestion pipeline.** The self-ingestion commit hook is the
   model, but for streaming graph data. Every insert/update/delete becomes a
   D/R/N event. The time pyramid absorbs the flow.

2. **Distribute the lattice.** Nodes on different shards hold different subsets
   of the adjacency matrix. BSS is local to each shard; R-links cross shards
   when popcount > threshold. Eventual consistency via Noether convergence.

3. **Productionize the rank algebra.** The five-level hierarchy (F,G,H,K,L,M)
   must run at wire speed. Every graph mutation triggers rank recomputation for
   affected nodes. The observable mask O(θ) becomes the graph's working set --
   nodes below threshold are cold storage.

4. **Fork RedisGraph at Phase 2.** Phase 1 proves the concept without forking.
   Phase 2 requires internal changes to node storage -- that's the fork point.
   The fork should be minimal: replace `GraphEntity` property storage and
   `Graph_ConnectNodes` edge creation. Everything else (query planner, indexing,
   cluster management) stays as close to upstream as possible.

### 20.7 Fork Strategy

```
upstream/RedisGraph  (track master, rebase regularly)
    │
    └── hllset/redisgraph  (our fork)
        ├── src/hllset/           # HLLSet integration layer
        │   ├── node.rs           # Content-addressed node storage
        │   ├── edge.rs           # R-link edge storage
        │   ├── temporal.rs       # Time pyramid layer management
        │   ├── rank.rs           # Five-level rank propagation
        │   └── matrix.rs         # Sparse adjacency via Fisher coupling
        ├── module/               # Redis module commands
        │   └── hllset_cmds.rs    # HLLSET.BSS, HLLSET.TEMPORAL, etc.
        └── tests/
```

**Rebase policy:** Pull upstream every release. Our changes are additive --
a new storage backend, not a rewrite. When upstream changes `GraphEntity`,
our shim adapts. The investment is in the shim layer, not in maintaining a
divergent fork.

### 20.8 Key Unknowns (to resolve before Phase 2)

1. **Bit budget at scale.** Each HLLSet is 4KB fixed. A billion-node graph =
   4TB of node storage. Plus R-link edges (4KB each). At what scale does this
   become prohibitive vs. RedisGraph's current variable-length property storage?

2. **BSS precision vs. Cypher recall.** BSS is a probabilistic filter.
   BSS(A, B) = 0.8 means "A contains 80% of B's bit pattern" -- not "the
   Cypher query will return 80% of the expected results." What's the empirical
   recall/precision curve for enterprise graph workloads?

3. **Delta propagation latency.** D/R/N deltas must reach all shards for the
   Noether convergence guarantee. What's the convergence time for a graph
   with 1M mutations/second across 100 shards? Does the time pyramid
   compression (60:1 at L0→L1) keep up?

4. **Cypher → BSS query planning.** Which Cypher patterns have efficient BSS
   equivalents? "Find all nodes within 3 hops" = 3 successive BSS operations
   along R-link chains. "Find the shortest path" = Dijkstra over R-link
   popcounts (integer weights). What subset of Cypher maps cleanly?

These unknowns are research questions, not blockers. Phase 1 answers them
empirically without touching RedisGraph internals.

---

## 21. Acknowledgment

This architecture emerged through dialogue between Alex Mylnikov, Deependra Kumar and DeepSeek
Code on June 27, 2026. The core ideas — HLLSet algebra, IICA principles, FPGA
mapping — existed before the session. But the specific discoveries (rank-based
learning, temporal lattice layers, fire-and-forget communication, system
lifecycle via reproduction) were collaborative insights that neither participant
possessed at the start. The dialogue itself was the design process.

## References

1. [MDBS_DDL_](https://bitsavers.trailing-edge.com/pdf/microDatabaseSystems/MDBS_DDL_Manual_Dec1985.pdf)
2. [Real-Time Systems Design and Analysis](https://staff.emu.edu.tr/alexanderchefranov/Documents/CMSE443/CMSE443%20Spring2020/Laplante2012%20Real-Time%20Systems%20Design%20and%20Analysis.pdf)