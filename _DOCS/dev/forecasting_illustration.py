"""
HLLSet Forecasting Illustrated — Without Forecasting Anything

This is NOT a forecaster. It's a demonstration of the MECHANISM:
  1. HLLSets are immutable (fixed token sets)
  2. The TF vector is yours to position
  3. Pre-positioning TF changes ranks BEFORE any new HLLSet arrives
  4. When a matching HLLSet does arrive, it's already correctly ranked

The "forecast" here is manual — we just move the TF peak. In production,
a constraint propagator determines where to move it (see FORECASTING.md).

Run: python3 forecasting_illustration.py
"""

# ── Simulated HLLSet Dictionary (token sets, immutable) ──
# In production these are 32,768-bit HLLSets via MurmurHash3.
# Here we use token sets as a readable proxy.

dictionary = {
    "explore":    {"unknown", "search", "learn", "discover"},
    "consume":    {"buy", "want", "need", "shop"},
    "rest":       {"sleep", "pause", "idle", "recover"},
    "defend":     {"threat", "danger", "guard", "shield"},
    "build":      {"create", "construct", "make", "assemble"},
    "navigate":   {"map", "route", "direction", "path"},
    "communicate":{"signal", "send", "receive", "message"},
}

# ── Simulated TF Vector ──
# In production: 32,768-entry vector tracking per-bit-position activity.
# Here: per-token weight. The TF says which concepts are "hot" right now.

def make_tf(weights):
    """Create a TF that returns weight for a token, 0.0 for unknown."""
    return lambda token: weights.get(token, 0.0)

def rank(hllset_tokens, tf):
    """rank(H) = sum of TF[token] for all tokens in H."""
    return sum(tf(t) for t in hllset_tokens)

def ranked_dictionary(dictionary, tf):
    """Return (name, rank) pairs sorted by descending rank."""
    scored = [(name, rank(tokens, tf)) for name, tokens in dictionary.items()]
    scored.sort(key=lambda x: -x[1])
    return scored

# ── Scene 1: Current State ──
# The system is in "explore and navigate" mode.
# High TF on exploration and navigation tokens.

print("=" * 65)
print("SCENE 1: CURRENT STATE — exploring, navigating")
print("=" * 65)
print()

tf_now = make_tf({
    "unknown": 0.9, "search": 0.8, "learn": 0.7, "discover": 0.6,
    "map": 0.5, "route": 0.5, "direction": 0.4, "path": 0.4,
    "create": 0.2, "construct": 0.1,
    "buy": 0.05, "want": 0.05,
    "sleep": 0.03, "pause": 0.03,
    "threat": 0.02, "danger": 0.02,
    "signal": 0.01, "send": 0.01,
})

print("TF now (top weights):")
for token, w in sorted(tf_now.__closure__[0].cell_contents.items(),
                       key=lambda x: -x[1])[:8]:
    print(f"  TF[{token}] = {w:.2f}")
print()

print("Dictionary ranks (TF_now ⊙ bitmask):")
current_ranks = ranked_dictionary(dictionary, tf_now)
for name, r in current_ranks:
    bar = "█" * int(r * 15)
    print(f"  {name:<15s} rank={r:.2f}  {bar}")
print()

top_now = current_ranks[0][0]
print(f"Top-ranked word: '{top_now}' — the system would {top_now} right now.")
print()

# ── Scene 2: Pre-Position for a Different Future ──
# Something is shifting. TF is manually moved toward "consume" and "defend"
# tokens. This is the "forecast" — but we're not predicting, we're preparing.
# The HLLSets (dictionary entries) are completely unchanged.

print("=" * 65)
print("SCENE 2: TF PRE-POSITIONED — preparing for consumption + defense")
print("=" * 65)
print()

tf_forecast = make_tf({
    "unknown": 0.15, "search": 0.10, "learn": 0.10, "discover": 0.08,
    "map": 0.10, "route": 0.08, "direction": 0.05, "path": 0.05,
    "create": 0.10, "construct": 0.08,
    "buy": 0.90, "want": 0.85, "need": 0.80, "shop": 0.75,
    "sleep": 0.05, "pause": 0.03,
    "threat": 0.70, "danger": 0.65, "guard": 0.60, "shield": 0.55,
    "signal": 0.40, "send": 0.35, "receive": 0.30, "message": 0.25,
})

print("TF forecast (top weights):")
closure_dict = tf_forecast.__closure__[0].cell_contents
for token, w in sorted(closure_dict.items(), key=lambda x: -x[1])[:8]:
    print(f"  TF[{token}] = {w:.2f}")
print()

print("Dictionary ranks (TF_forecast ⊙ bitmask):")
forecast_ranks = ranked_dictionary(dictionary, tf_forecast)
for name, r in forecast_ranks:
    bar = "█" * int(r * 10)
    print(f"  {name:<15s} rank={r:.2f}  {bar}")
print()

top_forecast = forecast_ranks[0][0]
print(f"Top-ranked word: '{top_forecast}' — system is now oriented toward {top_forecast}.")
print()

# ── Scene 3: Rank Migration ──
# Show how each word's rank changed between the two TF vectors.
# The HLLSets never changed. Only the interpretation shifted.

print("=" * 65)
print("SCENE 3: RANK MIGRATION — same HLLSets, different interpretation")
print("=" * 65)
print()

print(f"{'Word':<15s} {'Rank (now)':>10s} {'Rank (forecast)':>16s} {'Δ':>8s}")
print("-" * 52)
rank_now_map = {name: r for name, r in current_ranks}
rank_fc_map = {name: r for name, r in forecast_ranks}
for name in dictionary:
    r_now = rank_now_map[name]
    r_fc = rank_fc_map[name]
    delta = r_fc - r_now
    arrow = "↑" if delta > 0.05 else ("↓" if delta < -0.05 else "→")
    print(f"{name:<15s} {r_now:>10.2f} {r_fc:>16.2f} {delta:>+7.2f} {arrow}")
print()

risers = [name for name in dictionary if rank_fc_map[name] - rank_now_map[name] > 0.1]
fallers = [name for name in dictionary if rank_now_map[name] - rank_fc_map[name] > 0.1]
print(f"Rising:  {risers}")
print(f"Falling: {fallers}")
print()

# ── Scene 4: The Payoff ──
# Now a scan arrives that matches the "consume" pattern.
# Because TF was pre-positioned, this HLLSet is ALREADY top-ranked.
# No warm-up. No transient misranking.

print("=" * 65)
print("SCENE 4: THE PAYOFF — a consume-pattern scan arrives")
print("=" * 65)
print()

# Simulate: a new scan creates an HLLSet matching "consume"
incoming_tokens = {"buy", "want", "purchase", "sale", "discount"}

# This HLLSet didn't exist before. It's being created right now.
# But TF_forecast already has high weights on {"buy", "want"}.
incoming_rank = rank(incoming_tokens, tf_forecast)

print(f"Incoming scan:  {incoming_tokens}")
print(f"Rank under TF_now:      {rank(incoming_tokens, tf_now):.2f}")
print(f"Rank under TF_forecast: {incoming_rank:.2f}")
print()

if incoming_rank > max(r for _, r in forecast_ranks):
    print("The new HLLSet is instantly top-ranked.")
    print("The system was already oriented toward it.")
    print()
else:
    # Find where it places
    placement = 1
    for name, r in forecast_ranks:
        if r > incoming_rank:
            placement += 1
        else:
            break
    print(f"The new HLLSet places #{placement} in the ranking.")
    print(f"The system is ready for it — no warm-up needed.")
    print()

# ── The Point ──
print("=" * 65)
print("WHAT JUST HAPPENED")
print("=" * 65)
print()
print("1. The dictionary of HLLSets NEVER CHANGED.")
print("   Same tokens, same entries in all three scenes.")
print()
print("2. The TF vector was PRE-POSITIONED between Scene 1 and Scene 2.")
print("   In production, a constraint propagator determines the shift.")
print("   Here, we moved it manually to demonstrate the mechanism.")
print()
print("3. This is NOT forecasting what HLLSets will be created.")
print("   It's pre-positioning the INTERPRETER so that WHATEVER arrives")
print("   is already correctly ranked relative to the expected future.")
print()
print("4. The reduction: 'forecast HLLSets' → 'forecast TF vector'.")
print("   TF is fixed-size (32,768 integers), monotonic, sparse,")
print("   constraint-rich, and its derivatives are already computed.")
print()
print("5. The system doesn't predict the world.")
print("   It predicts its own interpretation of the world.")
print("   When the world conforms → already oriented.")
print("   When the world surprises → surprise is quantified.")
print()
print("rank(H) = TF ⊙ bitmask(H)")
print("The bitmask is immutable. The TF is yours. Position it wisely.")
