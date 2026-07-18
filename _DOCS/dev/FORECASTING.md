# HLLSet Forecasting: Pre-Positioning the Interpreter

> **Session:** July 17, 2026
> **Status:** Architectural design — no new algebraic operations required
> **Prerequisite:** `DIMENSIONAL_NESTING.md`, `SELF_REPROGRAMMING_ARCHITECTURE.md`

---

## 0. The Inversion

Traditional forecasting asks: **"What will happen next?"**

HLLSet forecasting asks: **"How should I be oriented when it does?"**

The difference is not rhetorical. The HLLSet is created by the world at
ingestion time — you cannot predict which tokens will arrive, because you
cannot predict which MurmurHash3 buckets they will land in. But you can
predict **which bit positions will be relevant**, because bit-position
relevance has temporal structure that the TF pyramid already captures.

```text
Traditional:           predict S(t+1)  →  generate H(t+1)  →  rank it
HLLSet:       predict TF(t+1)  →  project onto existing H  →  ranks pre-align
                       ↑                              ↑
                 you own this                  these are immutable
```

The HLLSets are immutable. The TF vector is yours. Forecasting the TF vector
means **pre-allocating relevance** so that when any HLLSet arrives — expected
or surprising — the rank algebra already has its weights in position.

---

## 1. Why This Works

The rank of any HLLSet is the Hadamard product of the TF vector and the
HLLSet's bitmask:

$$\text{rank}(H) = \text{TF} \odot \text{bitmask}(H) = \sum_{b \in H} \text{TF}[b]$$

The bitmask is fixed (IICA property). The TF vector evolves (monotonic CRDT).
If you can predict $\text{TF}(t+1)$, you can pre-compute the rank of every
HLLSet in the dictionary — past, present, and future — before the next scan
arrives.

You don't know which HLLSet will be created. You don't need to. You know
which bit positions will carry weight, and that's sufficient to pre-rank
every HLLSet that might appear, including ones created by future scans whose
token content is entirely unknown.

---

## 2. The Forecasting Reduction

The problem reduces from:

> "Predict the next HLLSet (32,768 unstructured bits with complex hash
> dependencies)"

to:

> "Predict the next TF vector (32,768 integers with known temporal structure)"

The TF vector has structure that generic time-series lacks:

### 2.1 Monotonicity

$$\text{TF}[b](t+1) \geq \text{TF}[b](t) \quad \forall b$$

The token-LUT only accumulates. TF never decreases. The forecast must be
monotonic — a constraint, not a statistical tendency.

### 2.2 Sparsity

Only a small fraction of the 32,768 positions are active at any time. Most
positions stay at zero. The forecast is sparse by construction.

### 2.3 Fisher Coupling

The Fisher matrix (Section 17.1 of the architecture doc):

$$F_{bb'} = \sum_{i=0}^{6} B^{(i)}_b \cdot B^{(i)}_{b'} + \sum_{t \in \text{history}} B^{(t)}_b \cdot B^{(t)}_{b'}$$

encodes which bits co-activate across temporal layers. If bit $b$ is forecast
to rise, bits $\{b' : F_{bb'} > 0\}$ have elevated probability. These are
hard structural constraints — bits that always co-occur in the Fisher matrix
cannot diverge in the forecast.

### 2.4 Layer Consistency

The temporal pyramid imposes hierarchical constraints:

```text
ΔTF₀ forecast must be consistent with TF₀ history
ΔTF₁ forecast must be consistent with TF₁ history
...

Cross-layer BSS: τ(L_i, L_j) must stay within historical bounds
```

A forecast that violates cross-layer consistency is physically impossible —
it describes a system whose short-term and long-term patterns contradict.

### 2.5 Existing Derivatives

The system already computes:

```text
ΔTF_i   = TF_i - TF_{i-1}     (first derivative — what changed?)
Δ²TF_i  = ΔTF_i - ΔTF_{i-1}   (second derivative — is change accelerating?)
```

These are the training signal. No external supervision required. The forecast
model trains on the same TF stack that serves the holographic memory.

---

## 3. Constraint Programming Formulation

The TF forecasting problem maps naturally to constraint programming:

```text
Variables:   TF[b] ∈ [0, max_tf]  for b ∈ [0, 32767]

Constraints:
  C1. Monotonicity:     TF[b](t+1) ≥ TF[b](t)
  C2. Sparsity:         |{b : TF[b] > 0}| ≤ active_threshold
  C3. Fisher coupling:  |TF[b] - TF[b']| ≤ δ  if F_{bb'} > θ
  C4. Layer bounds:     TF[b] ∈ [L_i_min[b], L_i_max[b]] per layer
  C5. Derivative bounds:|ΔTF[b]| ≤ max_velocity
                         |Δ²TF[b]| ≤ max_acceleration

Objective: find the feasible region.
           Center of region = point forecast.
           Extent of region = uncertainty bounds.
```

The constraints are all integer-linear (or can be linearized). The search
space is 32,768 dimensions but the active subspace is sparse — typically
a few hundred to a few thousand active positions.

### Why Constraint Programming?

- **Monotonicity is native.** CP solvers handle $x \geq y$ as a single
  constraint propagation. No gradient descent, no loss function.

- **The Fisher matrix IS the constraint graph.** $F_{bb'}$ directly
  defines which variables constrain each other. The dependency structure
  is given, not learned.

- **Feasibility is the answer.** We don't need the optimal forecast — we
  need the *possible* forecasts. The region tells us what can happen; the
  center tells us what's most likely.

- **FPGA compatibility.** Constraint propagation is integer arithmetic.
  The same AND/OR/popcount/cmp operations that run the lattice can run
  a lightweight constraint propagator.

---

## 4. Pre-Positioning: The Operational Loop

```text
At time t:
  1. TF_forecast(t+1) ← constraint_propagate(TF_history, F, layer_bounds)
  2. Pre-load TF_forecast into the rank algebra
     → rank(H) = TF_forecast ⊙ bitmask(H) for all H in dictionary
     → ranks are now aligned with the EXPECTED future
  3. S(t+1) arrives → HLLSet created → immediately correctly ranked
  4. TF_actual(t+1) computed from S(t+1)
  5. Δ = TF_actual - TF_forecast
     → Δ ≈ 0: forecast was accurate, system was ready
     → Δ ≫ 0: surprise — update constraint model, feed back to Noether
  6. The difference IS the learning signal
```

**Key property:** Step 2 completes before Step 3. The system is oriented
before the world acts. This is possible because TF pre-alignment is a
projection — it touches only the TF vector, not the HLLSets. No bitmasks
are created, modified, or predicted.

---

## 5. What "Ready" Means

### 5.1 Familiar Patterns Rank Instantly

If the forecast is accurate and the incoming scan matches a known pattern,
the corresponding HLLSet already has high rank. No warm-up period. No
transient misranking while the TF catches up.

### 5.2 Surprise Is Precisely Quantified

When $\text{TF}_{\text{actual}}$ diverges from $\text{TF}_{\text{forecast}}$,
the system knows two things simultaneously:

- **That** something unexpected happened (DRN decomposition)
- **Why** — which bit positions deviated, by how much, and which Fisher
  couplings were violated (TF delta analysis)

The Noether controller gets a ranked list of *which bits surprised the
system*, not just a scalar divergence signal.

### 5.3 The Materializer Is Pre-Tuned

The materializer uses TF to disambiguate bit positions with multiple
candidate tokens. A pre-aligned TF means the materializer selects tokens
that are temporally coherent — the ones that *will* matter — rather than
whichever tokens happen to have the highest current count.

### 5.4 Planning Without Rollouts

Given a set of candidate actions (each expressed as an HLLSet or token set),
the forecasted TF gives the expected rank of each action's HLLSet:

$$\text{expected\_rank}(\text{action}) = \text{TF}_{\text{forecast}} \odot \text{bitmask}(\text{action})$$

Select the highest-ranked action. This is a planner — not one that simulates
future states, but one that evaluates actions against the expected relevance
distribution. Same mechanism as the content-addressable program counter
(Section 7 of the architecture doc): the next instruction is whichever word's
HLLSet best matches the current input. The next action is whichever candidate's
HLLSet best matches the forecasted relevance.

---

## 6. Degradation Modes

### 6.1 Forecast = Actual (Stable Environment)

The system runs at peak efficiency. Ranks are pre-aligned. The materializer
is pre-tuned. Surprise is zero. The constraint model tightens its bounds.

### 6.2 Small Divergence (Adaptation)

$\Delta$TF is non-zero but within bounds. The constraint model widens its
feasible region. Ranks adjust within a few cycles. The system adapts.

### 6.3 Large Divergence (Novelty)

$\Delta$TF exceeds bounds. The Fisher coupling constraints are violated.
The constraint model resets — the feasible region was wrong. The system
enters exploration mode (Section 18 of the architecture doc: temperature
increases). Ranks reshuffle. The forecast model relearns from scratch.

### 6.4 Chaotic Divergence (L0-Only Mode)

$\Delta$TF is unbounded and persistent. The constraint model cannot converge.
The system fragments into perpetual L0 — pure reactivity, no forecasting
possible. This is correct: in a truly chaotic environment, pre-positioning
is harmful (you'd be oriented toward the wrong future). The architecture
degrades gracefully — stop forecasting, just react.

---

## 7. Implementation Notes

### 7.1 What Already Exists

| Component | Status | Location |
|---|---|---|
| TF vector (32,768 × f64) | Specified, simulated in notebooks | `TFSimulator` in notebooks 08, 10 |
| ΔTF, Δ²TF derivatives | Specified | Section 4 of proposal, Section 17.1 of architecture doc |
| Fisher matrix $F_{bb'}$ | Specified | Section 17.1 of architecture doc |
| Monotonicity guarantee | Inherent (CRDT union) | IICA properties |
| Layer consistency (BSS) | Implemented | `TemporalLattice` in notebook 06 |

### 7.2 What Needs Building

| Component | Effort | Description |
|---|---|---|
| Constraint propagator | ~500 lines Rust | Lightweight CP engine over sparse 32,768-var domain |
| Forecast-TF projection | ~100 lines | Apply TF_forecast to dictionary, return ranked HLLSets |
| Surprise quantification | ~200 lines | Structured delta: per-bit deviation + Fisher violation report |
| Noether integration | ~150 lines | Feed surprise signal into controller decision logic |
| Planner | ~300 lines | Action selection via TF_forecast ⊙ bitmask(action) |

### 7.3 What We Don't Need

- A neural network. The constraints are structural, not learned.
- A time-series database. The TF pyramid IS the time-series.
- A separate training pipeline. The derivatives ARE the training signal.
- New algebraic operations. Everything is projection of existing structures.

---

## 8. Iterative Forecasting: The Flashlight Model

### 8.1 The Metaphor

A car driving at night. The headlights illuminate the road ahead — clearly
near, dimly far. Each meter driven, the headlights reposition. The beam
doesn't predict the entire journey; it predicts the next segment, and each
new segment becomes the starting point for the next prediction.

HLLSet forecasting operates the same way:

```text
Step 1:  TF_forecast(t+1) = propagate(TF_actual(t), F, bounds)
         → system pre-positions for t+1
         → S(t+1) arrives → TF_actual(t+1) computed

Step 2:  TF_forecast(t+2) = propagate(TF_actual(t+1), F, bounds)
         → starting point is NOW the actual t+1, not the old forecast
         → system pre-positions for t+2
         → S(t+2) arrives → TF_actual(t+2) computed

Step 3:  TF_forecast(t+3) = propagate(TF_actual(t+2), F, bounds)
         ...
```

Each step resets the origin. The forecast never drifts on its own errors —
every new actual TF corrects the trajectory. You don't forecast t+1 through
t+10 in one shot. You forecast t+1, observe, then forecast t+2 from the
corrected position. The headlights move with the car.

### 8.2 Degradation with Distance

The flashlight degrades with distance — and so does the forecast. After $k$
propagation steps without an intervening actual observation, the feasible
region for each variable expands:

$$\text{TF}[b](t+k) \in [\text{TF}[b](t) + k \cdot v_{\min}(b),\; \text{TF}[b](t) + k \cdot v_{\max}(b)]$$

where $v_{\min}(b)$ and $v_{\max}(b)$ are the historically-observed minimum
and maximum velocity of bit position $b$. At $k=1$, the bounds are tight. At
$k=10$, they're wide. At $k \rightarrow \infty$, they span the full domain —
the flashlight has dissipated into ambient darkness.

The degradation is not uniform. Different bit positions degrade at different
rates, and the Fisher matrix is what determines which degrade slowly and
which degrade fast.

### 8.3 Fisher Matrix as Native Degradation Manager

The Fisher matrix $F_{bb'}$ counts how often bits $b$ and $b'$ co-occur
across temporal layers. This is exactly the information needed to control
degradation:

**Tightly coupled bits (high $F_{bb'}$).** These bits always appear together.
If bit $b$ is forecast to rise, bit $b'$ is constrained to rise with it.
Their relative positions are locked — even at long horizons, the forecast
for the pair is narrow. These are the bright center of the beam: the road
markings you can see far ahead.

**Weakly coupled bits (low $F_{bb'}$).** These bits co-occur occasionally but
not reliably. The forecast for $b$ says little about $b'$. Each propagation
step, their domains expand independently. These are the edges of the beam:
shapes you can make out nearby but not at distance.

**Uncoupled bits ($F_{bb'} \approx 0$).** These bits never co-occur. They
degrade at their own intrinsic rate, independent of everything else. These
are the darkness beyond the beam: you know something is there, but no
structural constraint helps you see it.

The Fisher matrix IS the beam pattern. The constraint propagator doesn't
need a separate "degradation model." The Fisher couplings determine which
variables remain tightly bounded and which expand freely. The propagation
step is the same for all variables — the Fisher matrix determines how much
each variable's domain widens per step.

### 8.4 Iterative Propagation Algorithm

```text
function flashlight_forecast(TF_actual, F, horizon_k):
    domains = {b: [TF_actual[b], TF_actual[b]] for b in active_bits}

    for step in 1..horizon_k:
        for each active bit b:
            # Intrinsic expansion: widen by historical velocity bounds
            domains[b].min -= v_min(b)
            domains[b].max += v_max(b)

            # Fisher coupling: tighten by co-occurrence constraints
            for each b' where F[b][b'] > coupling_threshold:
                # b and b' move together → their difference is bounded
                max_delta = historical_max(|TF[b] - TF[b']|)
                domains[b].max = min(domains[b].max, domains[b'].max + max_delta)
                domains[b].min = max(domains[b].min, domains[b'].min - max_delta)

        # Layer consistency: cross-layer BSS bounds
        for each layer L_i in temporal pyramid:
            constrain_by_layer(domains, L_i)

    # Point forecast = center of each domain
    TF_forecast = {b: midpoint(domains[b]) for b in active_bits}
    return TF_forecast, domains  # domains = uncertainty bounds
```

The key property: each propagation step widens domains (degradation), and
the Fisher couplings fight the widening (structural constraints). The balance
between these two forces determines the beam width at each distance.

### 8.5 Beam Width as Actionable Horizon

The beam width is not just a confidence measure — it's an **actionable
horizon**. A bit position whose domain has widened beyond a threshold is
"out of the beam" — its forecast is too uncertain to drive decisions.

This naturally partitions the dictionary:

```text
Active horizon (domain_width < θ_active):
    → HLLSets dominated by these bits are reliably ranked
    → The system can commit to actions based on these ranks

Dim horizon (θ_active ≤ domain_width < θ_dim):
    → HLLSets with these bits have fuzzy ranks
    → The system prepares but doesn't commit
    → Temperature rises (Section 18 of architecture doc)

Dark (domain_width ≥ θ_dim):
    → No useful constraints on these bits
    → The system treats these HLLSets as unknown
    → Falls back to reactive L0 behavior for these
```

The car doesn't steer by what's in the darkness. It steers by what's in the
beam. The Fisher matrix determines the beam pattern. The constraint
propagator determines the beam width at each distance. The system's behavior
adapts naturally to the quality of its forecast.

### 8.6 Why This Is Native, Not Added

The flashlight model doesn't introduce new machinery. It describes how the
existing components interact under iterative forecasting:

| Flashlight concept | Existing HLLSet component |
|---|---|
| Beam pattern | Fisher matrix $F_{bb'}$ |
| Beam width at distance $k$ | Variable domain after $k$ propagation steps |
| Degradation rate per bit | Historical velocity bounds $[v_{\min}(b), v_{\max}(b)]$ |
| Tightly coupled bits (bright center) | High $F_{bb'}$ pairs |
| Weakly coupled bits (dim edges) | Low $F_{bb'}$ pairs |
| Active horizon | Domain width < θ_active |
| Repositioning | New TF_actual resets all domains to zero-width |
| Temperature adaptation | Noether controller reads domain widths, adjusts T |

The Fisher matrix was already computed for the Noether steering equation
(Section 17.1). The velocity bounds are already tracked by the ΔTF pyramid.
The constraint propagation is the only new component — and it's integer
arithmetic on the same bitmask data structures.

---

## 9. Relationship to Holographic Memory

The holographic memory equation:

$$\text{past\_state}(t) \approx H_{\text{world}}(\text{now}) \odot \text{TF}_{\text{stack}}[t]$$

has a natural forward extension:

$$\text{future\_state}(t+1) \approx H_{\text{world}}(\text{now}) \odot \text{TF}_{\text{forecast}}(t+1)$$

The same mechanism — applying a TF lens to the world top — serves both
retrospection and prospection. The only difference is whether the TF vector
comes from the historical stack or from the constraint propagator.

```text
Past:     H_world(now) ⊙ TF_stack[t]       ← stored, deterministic
Present:  H_world(now) ⊙ TF_stack[now]     ← current, observed
Future:   H_world(now) ⊙ TF_forecast(t+1)  ← projected, constrained
```

Three time directions. One operation. The TF lens doesn't care which
direction in time it's pointing.

---

## 10. Summary

HLLSet forecasting is not about predicting what HLLSets will be created.
It's about pre-positioning the interpreter so that whatever HLLSets arrive,
the system is already oriented.

The reduction from "forecast HLLSets" to "forecast TF vector" is what makes
this feasible. The TF vector is a fixed-size, monotonic, sparse, constraint-
rich integer vector whose temporal derivatives are already computed by the
existing architecture. The Fisher matrix provides the coupling constraints.
The temporal pyramid provides the layer bounds. The Noether controller
provides the feedback signal.

Nothing was added to the algebra. The only new component is a constraint
propagator — integer arithmetic on a sparse subset of 32,768 variables.
The same operations the FPGA already runs for everything else.

```text
Forecasting in one equation:

  TF_forecast(t+1) = propagate(TF_history, F, layer_constraints)

Forecasting in one insight:

  You don't predict the world. You predict your own interpretation of it.
```
