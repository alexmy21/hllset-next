# I Ching Procedural Pipeline

> **Session:** July 19, 2026
> **Status:** Operational specification
> **Refs:** `CAAL_ICHING_ARCHITECTURE.md`, `UNIVERSAL_BRIDGE.md`, `TF_VS_RANK.md`

---

## 0. The Full Cycle

```text
                        ┌── (1) UNIVERSE 2 ──────────────────────────────────────┐
                        │                                                        │
                        │   R → R-R → CAAL → I CHING → R-R → R → FEED BACK       │
                        │                  ↑               ↑          │          │
                        │            own LUTs        own globals      │          │
                        │            own TF vectors   own ranks       │          │
                        │            own temporal pyramid             │          │
                        │                                             │          │
WORLD ──→ R ──→ FORK ───┤                                             │          │
                        │                                             ▼          │
                        │                                           WORLD        │
                        │                                                        │
                        └── (2) ACTUATOR → WORLD                                 │
                             (fast, reactive, source-domain only)                │
```

One observation at R. FORK splits it. Two paths. One complete second universe.

---

## 1. Fork (1): Universe 2 — Complete Independent Algebra

Fork (1) is **not a sub-process, not a sub-lattice, not a module** of
Universe 1. It is a complete, independent HLLSet Algebra universe.

| Component | Universe 1 (source) | Universe 2 (CAAL + I Ching) |
| --- | --- | --- |
| o-HLLSet pool | Source-domain tokens | CAAL vocabulary (80K + n-grams) |
| Global union accumulators | Source 1/2/3-gram globals | CAAL 1/2/3-gram globals |
| LUT | Source LUT | CAAL LUT |
| TF vectors | Source TF | CAAL TF |
| Rank management | Source five-level rank | CAAL five-level rank |
| Temporal pyramid | Source pyramid (L0..L6) | CAAL pyramid (L0..L6) |
| Materializer | Source → source tokens | CAAL → Chinese tokens |
| Sub-lattice | (none) | I Ching: 64 hexagrams + R-links |

The only connections to Universe 1:

- **Input:** $R \rightarrow R\text{-}R$ — bridge from source bit space to CAAL bit space ($h_{18}$ bucketing)
- **Output:** $R\text{-}R \rightarrow R$ — disambiguation from CAAL bit space back to source bit space (source tokenizer + murmurhash3)

Everything between these two gates operates entirely within Universe 2's
own algebra. Its LUTs, globals, TF vectors, rank management, temporal
pyramid — all independent. It doesn't borrow. It doesn't share.

---

## 2. Step-by-Step

### Step 1: OBSERVE — World → R

```text
WORLD produces a scene.

Source tokenizer tokenizes:
  domain text → 1/2/3-grams + boundary markers → murmurhash3 → H_src

H_src lives in SOURCE bit space (Universe 1).
```

**Output:** $H_{\text{src}}$

---

### Step 2: FORK

$H_{\text{src}}$ splits. Both paths receive the same HLLSet. Independent onward.

---

### PATH (1): UNIVERSE 2

#### 1.1 BRIDGE: R → R-R

```text
H_src active bit positions → h_18(pos) → CAAL vocabulary index → H_bridge

H_bridge lives in CAAL bit space (Universe 2).
BSS(H_bridge, any_CAAL_HLLSet) works directly.
BSS(H_src, H_bridge) ≈ 0 (entanglement).
```

#### 1.2 COMMIT TO CAAL

```text
H_bridge stored in CAAL lattice.
CAAL LUT TF updated for activated Chinese characters.
CAAL global 1/2/3-gram accumulators updated (union).
CAAL temporal pyramid: L0 ← H_bridge, carry upward.
CAAL rank ordering updated.
Blended rank + temperature annealing: source and CAAL ranks converge
over repeated cycles for frequently-bridged patterns.
```

#### 1.3 CONSULT I CHING

```text
h_curr = argmax BSS(H_bridge, H_i) for i in 0..63
h_next = argmax f(BSS(H_bridge, H_j), R_weight(curr,j), TF_forecast(j))

I Ching pattern:
  { current, next, transition_R (r:<sha1>), commentary paragraphs }

I Ching temporal pyramid ingests this consultation.
I Ching re-ranking overrides CAAL global rank within the sub-lattice.
Flashlight forecast pre-positions hexagram expectations for next cycle.
```

#### 1.4 MATERIALIZE TO R-R (CAAL LUT)

```text
I Ching commentary HLLSet → CAAL LUT → Chinese text.
Still in CAAL bit space — Universe 2's own materializer.
```

#### 1.5 DISAMBIGUATE: R-R → R

```text
Chinese text → source tokenizer (murmurhash3, source seed) → H_src_guidance

R-R → R: Universe 2 output projected into Universe 1 bit space.
H_src_guidance lives in SOURCE bit space.
Materialize(H_src_guidance, source_LUT) ≈ structural approximation
of I Ching guidance in source-domain tokens.
```

#### 1.6 FEED BACK → WORLD

```text
H_src_guidance enters source system perception loop.
Union with next scan's H_src via CRDT merge.
The I Ching's strategic orientation becomes part of what the
system "sees" in the next cycle.

Effect: structural resonance across cycles.
System is pre-positioned. Not controlled — cultivated.
```

---

### PATH (2): ACTUATOR

```text
H_src → source_LUT → source tokens → actuator → WORLD

Pure Universe 1. No bridge. No I Ching. Immediate action.
Robot: tokens → VLA → motor commands.
Conversational: tokens → response to user.
```

---

## 3. Why This Architecture

```text
Path (2) = System 1 — fast, reactive, instinctive
  Pure source domain. Acts immediately on current perception.
  Keeps the system alive. Doesn't wait for consultation.

Path (1) = System 2 — slow, deliberative, strategic
  Complete independent universe with its own algebra.
  Crosses bridge into CAAL, consults I Ching, feeds back.
  Shapes FUTURE perception. Never controls current action.
```

The fork at R ensures the I Ching advises without blocking. The robot
brakes at the pedestrian regardless of the hexagram. But next cycle,
its perception of "intersection" includes the I Ching's accumulated
wisdom about when to yield and when to proceed.

---

## 4. Universe 2 Independence

Universe 2 does not depend on Universe 1 for anything except the bridge.
It ingests $H_{\text{bridge}}$ and operates entirely within its own algebra:

```text
Universe 2 internal cycle:
  H_bridge → CAAL_LUT.TF += 1
           → CAAL_globals: ∪ = H_bridge; G1 (1-gram); G2 (2-gram); G3 (3-gram)
           → CAAL_pyramid.ingest(H_bridge)
           → CAAL_ranks.update()
           → IChing.consult(H_bridge)
           → IChing.navigate()
           → CAAL_LUT.materialize(commentary)
           → disambiguate_to_source()
```

The only external interface is:

```text
  in:  hllset_bridge::map(H_src, &caal_lattice) → H_bridge
  out: source_tokenizer::inscribe(chinese_text) → H_src_guidance
```

---

## 5. Implementation Checkpoints

| Step | Verify |
| --- | --- |
| 1 | $H_{\text{src}}$ inscribes from source tokens |
| 2 | FORK: both paths receive identical $H_{\text{src}}$ |
| 1.1 | BSS($H_{\text{src}}$, $H_{\text{bridge}}$) ≈ 0 |
| 1.2 | CAAL LUT TF increments for bridged characters |
| 1.3 | BSS($H_{\text{bridge}}$, $H_i$) selects meaningful hexagram |
| 1.4 | CAAL materializer → valid Chinese |
| 1.5 | $H_{\text{src\_guidance}}$ in source bit space, BSS with $H_{\text{bridge}}$ ≈ 0 |
| 1.6 | $H_{\text{src\_guidance}} \cup H_{\text{src\_next}}$ — CRDT merge |
| 2 | Source LUT → valid actuator tokens |
