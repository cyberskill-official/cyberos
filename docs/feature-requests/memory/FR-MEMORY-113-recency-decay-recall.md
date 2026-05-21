---
id: FR-MEMORY-113
title: "memory recall ranking — Park-et-al combined score (relevance · 0.4 + importance · 0.3 + recency · 0.3) with configurable Ebbinghaus decay; MARS-aligned forgetting curve"
module: memory
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_frs: [FR-MEMORY-108, FR-MEMORY-112, FR-MEMORY-114, FR-MEMORY-115, FR-MEMORY-120]
depends_on: [FR-MEMORY-108, FR-MEMORY-112]
blocks: [FR-MEMORY-115, FR-MEMORY-120]

source_pages:
  - playground/extracts/agentic-memory.article.txt  # see "Memory management" / "Time-based decay" sections
source_decisions:
  - DEC-190 (Recall combined score is `relevance · w_r + importance · w_i + recency · w_t` with default weights 0.4 / 0.3 / 0.3 per Park et al. 2023 "Generative Agents"; configurable per-store via `manifest.recall_weights`)
  - DEC-191 (Recency decay follows the form `recency = decay_factor^hours_old` with default decay_factor=0.995 ⇒ ~4-day half-life; MARS framework's Ebbinghaus selection is supported as an alternative profile)
  - DEC-192 (Absent `meta.importance` defaults to 0.5 — same DEC-181 neutrality principle as quality_score for Episodes; never silently boost or penalise)
  - DEC-193 (Recall weights and decay parameters live in `manifest.json:recall_weights` so two memories can tune independently; the writer rejects malformed config at load-time, not at query-time)

language: python 3.10
service: modules/memory/cyberos/
new_files:
  - modules/memory/cyberos/core/ranking.py
  - modules/memory/cyberos/core/decay.py
  - modules/memory/tests/test_ranking_combined_score.py
  - modules/memory/tests/test_decay_profiles.py
  - modules/memory/bench/bench_recall_latency.py
modified_files:
  - modules/memory/cyberos/core/semantic.py            # delegate ranking to ranking.score_hits()
  - modules/memory/cyberos/core/reader.py              # same delegation for FTS5 path
  - modules/memory/cyberos/cli/recall.py               # FR-MEMORY-112 stub replaced with real `combined_score` from ranking.py
  - modules/memory/cyberos/core/frontmatter.py         # accept optional `meta.importance: float ∈ [0.0, 1.0]` on every kind (not just episode)
  - modules/memory/memory.schema.json                  # add `Importance` definition + `importance` field on `MemoryFrontmatter`
  - modules/memory/memory.invariants.yaml              # add `manifest-recall-weights-valid` + `importance-range`
  - modules/memory/cyberos/core/writer.py              # validate `manifest.recall_weights` at writer construction (fail-fast, see §1 #11)
allowed_tools:
  - file_read: modules/memory/**
  - file_write: modules/memory/cyberos/**, modules/memory/tests/**, modules/memory/bench/**, modules/memory/memory.schema.json, modules/memory/memory.invariants.yaml
  - bash: cd modules/memory && python -m pytest tests/test_ranking_combined_score.py tests/test_decay_profiles.py -v
  - bash: cd modules/memory && python bench/bench_recall_latency.py --episodes 10000 --trials 100
disallowed_tools:
  - silently swallow malformed `manifest.recall_weights` (per §1 #11 — fail-fast at writer construction)
  - default `meta.importance` to anything other than literal 0.5 in the ranking maths (per DEC-192)
  - apply recency decay to results that have no `last_seen_at` (per §1 #6 — fall back to 1.0 ≡ "as-fresh-as-possible")

effort_hours: 8
sub_tasks:
  - "0.5h: memory.schema.json — `Importance` definition (number, 0.0..1.0) + add `importance: $ref` to MemoryFrontmatter and EpisodeFrontmatter; `RecallWeights` definition on manifest"
  - "0.5h: memory.invariants.yaml — `manifest-recall-weights-sum-to-one` (error: |w_r + w_i + w_t - 1.0| ≤ 1e-6) + `importance-range` (error) + `decay-factor-range` (warning: 0.0 < f < 1.0)"
  - "1.0h: cyberos/core/decay.py — `recency(now, last_seen_at, profile)` with two built-in profiles: `exponential(decay_factor=0.995)` (Park et al.) and `ebbinghaus(strength=1.0)` (MARS-aligned); pluggable so future profiles can land via `entry_points`"
  - "1.0h: cyberos/core/ranking.py — `score_hits(hits, weights, decay)` returns sorted list with `combined_score` annotated; pure function (no I/O) so it's deterministic + easy to test"
  - "0.5h: cyberos/core/writer.py — load + validate `manifest.recall_weights` at construction; raise `ManifestError` on malformed input (do NOT lazy-validate at first recall — surface early)"
  - "0.5h: cyberos/core/semantic.py — replace inline relevance-sort with `ranking.score_hits(...)`; back-compat preserved (caller can pass `weights=None` ⇒ legacy relevance-only)"
  - "0.5h: cyberos/core/reader.py — same swap for FTS5 path; same `weights=None` fallback"
  - "0.5h: cyberos/cli/recall.py — replace FR-MEMORY-112's `combined_score` stub with `ranking.score_hits(...)` call; CLI accepts `--decay-profile {exponential,ebbinghaus}` override"
  - "1.0h: tests/test_ranking_combined_score.py — 14 cases (weight permutations, importance defaults to 0.5, recency=1.0 fallback when last_seen absent, weights sum check, regression vs FR-MEMORY-112 default ordering)"
  - "1.0h: tests/test_decay_profiles.py — 8 cases (exponential half-life property, Ebbinghaus monotonic decreasing, both bounded in [0.0, 1.0], invalid params rejected)"
  - "1.0h: bench/bench_recall_latency.py — fixture-driven benchmark, asserts p95 budget for 10K-Episode store with ranking applied (within FR-MEMORY-112's 500 ms semantic budget + ≤ 10% ranking-overhead headroom)"
risk_if_skipped: "Without recency decay and importance weighting, `cyberos recall-similar` (FR-MEMORY-112) ranks purely by semantic relevance. Two pathologies follow: (1) Old, low-quality, lucky-similarity hits dominate fresh, high-quality recent hits — the memory's recall surface degrades monotonically as the store grows. (2) FR-MEMORY-115 (`cyberos dream`) can't use recall to find candidate episodes for cross-session pattern detection without a quality signal — every episode of the same task looks the same regardless of whether the agent did a good job. The article (Ramakrushna §'Time-based decay') and MARS [1] both single out time-decay forgetting as the load-bearing primitive that prevents old facts from drowning new ones. Park et al. 2023 'Generative Agents' uses the exact 0.4 / 0.3 / 0.3 weighting we adopt, validated on a benchmark generative-agents simulation. Skipping this FR means we'd later have to retrofit the ranking call sites in `semantic.py`, `reader.py`, `recall.py`, and FR-MEMORY-115's input pipeline — and we'd ship a recall surface that's known to degrade. Cheaper to ship the right shape now."
---

## §1 — Description (BCP-14 normative)

The recall ranking layer **MUST** replace the existing "sort by relevance descending" behaviour in `cyberos.core.semantic.recall(...)` and `cyberos.core.reader.recall(...)` with a **combined score** that blends semantic relevance, memory importance, and recency. The contract:

1. **MUST** define the combined score as `combined_score = relevance · w_r + importance · w_i + recency · w_t` where the weights `(w_r, w_i, w_t)` are loaded from `manifest.json:recall_weights` (defaults `(0.4, 0.3, 0.3)` per DEC-190 and Park et al. 2023). Sum of weights MUST equal 1.0 within ±1e-6; the walker invariant `manifest-recall-weights-sum-to-one` rejects malformed manifests.
2. **MUST** treat `relevance` as the cosine similarity (semantic backend) or BM25-normalised score scaled to `[0.0, 1.0]` (FTS5 backend) returned by the underlying engine. Engines that emit out-of-range values (e.g. SQLite FTS5 raw BM25 returns negative scores) are normalised by the engine adapter, not the ranking layer.
3. **MUST** treat `importance` as the float `meta.importance ∈ [0.0, 1.0]` declared in the memory file's frontmatter. Absence is exactly 0.5 (DEC-192). The walker invariant `importance-range` rejects out-of-range values (error severity).
4. **MUST** treat `recency` as the value returned by the active **decay profile** evaluated at `now - last_seen_at` (delta in hours). Two built-in profiles MUST be supported:
    - **Exponential** (default): `recency = decay_factor^hours_old`, default `decay_factor = 0.995` ⇒ half-life ≈ 138 hours ≈ 5.75 days. Park et al. 2023 form. (Note: the proposal earlier estimated 4 days; the precise math is `t_½ = ln(0.5) / ln(0.995) ≈ 138.3h ≈ 5.76 days`.)
    - **Ebbinghaus**: `recency = exp(-hours_old / strength)`, default `strength = 240` (i.e. ~10-day characteristic time). MARS framework alignment.
5. **MUST** select the active decay profile via `manifest.json:recall_weights.decay_profile` (string enum `exponential | ebbinghaus`, default `exponential`) with `manifest.json:recall_weights.decay_params` (dict of profile-specific params). CLI override `--decay-profile {exponential|ebbinghaus}` MUST work without modifying the manifest (per-query override is non-persistent).
6. **MUST** fall back to `recency = 1.0` when the hit's `last_seen_at` is absent or unparseable. This treats unknown-age memories as "as fresh as possible" — the conservative direction, since the alternative (treating them as ancient) would silently suppress legitimate hits.
7. **MUST** preserve the existing `recall(...)` Python signature except for one new keyword argument `weights: Optional[RecallWeights] = None`. Callers that pass `weights=None` (legacy callers) get the ranking-applied result; callers that explicitly want raw relevance order set `weights=RecallWeights.relevance_only()` (programmatic, not a CLI flag — used only by internal tools that need to inspect raw similarity).
8. **MUST** annotate every `RecallHit` with the four scalars (`relevance`, `importance`, `recency`, `combined_score`) so downstream consumers (CLI `--json`, FR-MEMORY-115 dream pipeline) can inspect the score derivation.
9. **MUST** validate `manifest.json:recall_weights` at writer construction time (`Writer.__init__`), raising `ManifestError` on:
    - sum of weights outside `[1.0 - 1e-6, 1.0 + 1e-6]`
    - any weight outside `[0.0, 1.0]`
    - unknown decay profile name
    - decay params that fail the profile's validator (e.g. `decay_factor ∉ (0.0, 1.0)` for exponential)
   Fail-fast means an operator typo doesn't silently degrade ranking quality on every recall.
10. **MUST** make `cyberos.core.ranking.score_hits()` a **pure function** (no I/O, no time.now() inside) — `now` is injected so tests are deterministic and downstream callers (FR-MEMORY-115 batch dream) can score against a snapshot timestamp.
11. **MUST** add a benchmark `bench/bench_recall_latency.py` asserting that the ranking step adds ≤ 10% overhead over the underlying engine's retrieval cost on a 10,000-Episode store (e.g. semantic retrieval ~450 ms p95 → ranked recall ≤ 495 ms p95).
12. **MUST** add `meta.importance` to the `MemoryFrontmatter` schema (general, not just Episode). The field is optional; semantics same as DEC-192.
13. **SHOULD** support a third pluggable decay profile slot via `entry_points` so third parties / future research can register additional curves (`power_law`, `linear`, `mars_three_phase`) without modifying the core. Slice-4+ stretch.
14. **SHOULD** emit an OTel span `memory.recall.scored` per recall invocation with attributes `hit_count`, `decay_profile`, `weights_hash`, `duration_ms`. Slice-4 stretch — gated by FR-OBS-001.

---

## §2 — Why this design (rationale for humans)

**Why 0.4 / 0.3 / 0.3 as defaults (§1 #1, DEC-190).** Park et al. 2023 ("Generative Agents") publishes a calibrated weight triple on a benchmark generative-agents simulation. The weights aren't arbitrary — they were tuned so that semantic relevance dominates (40%) but importance and recency get meaningful weight (30% each). Adopting their numbers gives us a reasonable starting line; per-store override via manifest lets each memory tune. The constraint that they sum to 1.0 (§1 #1) keeps `combined_score` itself in `[0.0, 1.0]` and makes the relative weights interpretable as percentages.

**Why exponential default + Ebbinghaus alternative (§1 #4, DEC-191).** Two production-validated forgetting curves exist in the literature: simple exponential (used in Park et al. and most recsys) and Ebbinghaus (used in MARS [1] and the spaced-repetition tradition). They have meaningfully different shapes — exponential is "smooth fade", Ebbinghaus has a sharper early drop with a long tail. Some workloads favour one over the other (e.g. CRM-style "this fact is current OR forgotten" prefers Ebbinghaus). Defaulting to exponential matches the article's design and most RAG systems; offering Ebbinghaus opt-in covers the MARS-alignment use case.

**Why `last_seen_at` absent → recency=1.0 (§1 #6).** Two options: (a) treat absent as "ancient" (recency ≈ 0); (b) treat absent as "fresh" (recency = 1.0). Option (a) silently down-ranks memories that haven't been tagged with seen-timestamps — most likely freshly-imported memories from `cyberos import`. Option (b) treats them as "I don't know how old this is, don't penalise it." We chose (b) because the failure mode of (a) (silent down-ranking) is invisible to the operator until they investigate "why isn't memory X showing up in recall?"; the failure mode of (b) (slight up-ranking of unstamped memories) is detectable by inspecting `combined_score` annotations (§1 #8). Visible failures > invisible ones.

**Why fail-fast at writer construction (§1 #9).** Operator typos in `manifest.json` are common (we've seen weights `[0.5, 0.5, 0.5]` and `decay_factor: 1.5`). If we lazy-validate at first recall, the memory starts up cleanly and silently mis-ranks until someone notices. Fail-fast means `cyberos doctor` and `Writer.__init__` both refuse to operate on malformed config; the operator sees the error immediately.

**Why pure-function `score_hits()` (§1 #10).** Two callers will use this: (a) live `recall(...)` paths that score at query time with `now = datetime.utcnow()`, and (b) FR-MEMORY-115's batch dream pipeline that scores against a fixed snapshot timestamp (so multiple agents dreaming over the same window get identical scores). Injecting `now` keeps the function deterministic + makes both callers trivially testable.

**Why a separate `decay.py` module (§1 #4).** Decay profiles are plug-points. Keeping them in a dedicated module with a clear `DecayProfile` protocol (`__call__(hours_old: float) -> float`) means slice-4 can add `power_law`, `mars_three_phase`, etc. without touching the ranking layer. The two-callers split keeps ranking and decay independently testable.

**Why ≤ 10% ranking overhead budget (§1 #11).** Ranking is a per-hit operation (constant time in the hit count). On a 10K-Episode store, recall returns ≤ 20 hits typically; 20 × small-float-arithmetic is dominated by retrieval cost. The 10% budget is a soft ceiling that catches accidental quadratic patterns (e.g. someone re-fetching `last_seen_at` from disk per-hit).

**Why add `meta.importance` to general MemoryFrontmatter, not just Episode (§1 #12).** The Park et al. formula applies to all memory kinds (a `decisions` entry is more important than a passing `facts` observation). Putting `importance` on the general frontmatter lets FR-MEMORY-114 (write-time importance scoring) target any kind, not just Episode. Forward-compatibility for FR-MEMORY-114 at zero cost.

---

## §3 — API contract

### Schema fragment

```json
{
  "$defs": {
    "Importance": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0,
      "description": "Memory importance in [0.0, 1.0]; absent ≡ 0.5 (neutral)."
    },
    "MemoryFrontmatter": {
      "properties": {
        "importance": {"$ref": "#/$defs/Importance"}
      }
    },
    "RecallWeights": {
      "type": "object",
      "required": ["relevance", "importance", "recency"],
      "properties": {
        "relevance":     {"type": "number", "minimum": 0.0, "maximum": 1.0},
        "importance":    {"type": "number", "minimum": 0.0, "maximum": 1.0},
        "recency":       {"type": "number", "minimum": 0.0, "maximum": 1.0},
        "decay_profile": {"type": "string", "enum": ["exponential", "ebbinghaus"], "default": "exponential"},
        "decay_params":  {"type": "object", "default": {}}
      }
    }
  }
}
```

### `cyberos.core.decay`

```python
# modules/memory/cyberos/core/decay.py
from __future__ import annotations
from dataclasses import dataclass
from datetime import datetime, timezone
from math import exp
from typing import Protocol


class DecayProfile(Protocol):
    """recency(hours_old) -> float in [0.0, 1.0]; monotonically non-increasing."""
    def __call__(self, hours_old: float) -> float: ...


@dataclass(frozen=True)
class Exponential:
    decay_factor: float = 0.995

    def __post_init__(self) -> None:
        if not (0.0 < self.decay_factor < 1.0):
            raise ValueError(f"decay_factor must be in (0.0, 1.0); got {self.decay_factor}")

    def __call__(self, hours_old: float) -> float:
        if hours_old < 0:                # future-dated last_seen → cap at 1.0
            return 1.0
        return self.decay_factor ** hours_old

    @property
    def half_life_hours(self) -> float:
        from math import log
        return log(0.5) / log(self.decay_factor)


@dataclass(frozen=True)
class Ebbinghaus:
    strength: float = 240.0     # characteristic time in hours

    def __post_init__(self) -> None:
        if self.strength <= 0:
            raise ValueError(f"strength must be > 0; got {self.strength}")

    def __call__(self, hours_old: float) -> float:
        if hours_old < 0:
            return 1.0
        return exp(-hours_old / self.strength)


def build_profile(name: str, params: dict | None = None) -> DecayProfile:
    params = params or {}
    if name == "exponential":
        return Exponential(decay_factor=params.get("decay_factor", 0.995))
    if name == "ebbinghaus":
        return Ebbinghaus(strength=params.get("strength", 240.0))
    raise ValueError(f"unknown decay profile {name!r}")


def hours_between(now: datetime, last_seen: datetime | None) -> float | None:
    if last_seen is None:
        return None
    if last_seen.tzinfo is None:
        last_seen = last_seen.replace(tzinfo=timezone.utc)
    if now.tzinfo is None:
        now = now.replace(tzinfo=timezone.utc)
    return (now - last_seen).total_seconds() / 3600.0
```

### `cyberos.core.ranking`

```python
# modules/memory/cyberos/core/ranking.py
from __future__ import annotations
from dataclasses import dataclass, replace
from datetime import datetime, timezone
from typing import Iterable, Optional

from cyberos.core.decay import DecayProfile, hours_between


@dataclass(frozen=True)
class RecallWeights:
    relevance:  float = 0.4
    importance: float = 0.3
    recency:    float = 0.3

    def __post_init__(self) -> None:
        for name, v in (("relevance", self.relevance), ("importance", self.importance), ("recency", self.recency)):
            if not (0.0 <= v <= 1.0):
                raise ValueError(f"{name} must be in [0.0, 1.0]; got {v}")
        if abs(self.relevance + self.importance + self.recency - 1.0) > 1e-6:
            raise ValueError(
                f"weights must sum to 1.0; got {self.relevance + self.importance + self.recency}"
            )

    @classmethod
    def relevance_only(cls) -> "RecallWeights":
        return cls(1.0, 0.0, 0.0)


@dataclass
class ScoredHit:
    path:          str
    relevance:     float
    importance:    float    # 0.5 if absent on the frontmatter
    recency:       float    # 1.0 if last_seen unknown
    combined_score: float
    frontmatter:   dict
    last_seen_at:  Optional[datetime]


def score_hits(
    hits:    Iterable["RecallHit"],
    weights: RecallWeights,
    decay:   DecayProfile,
    now:     Optional[datetime] = None,
) -> list[ScoredHit]:
    now = now or datetime.now(timezone.utc)
    out: list[ScoredHit] = []
    for h in hits:
        imp = h.frontmatter.get("importance", 0.5)         # DEC-192
        h_old = hours_between(now, h.last_seen_at)
        rec = decay(h_old) if h_old is not None else 1.0    # §1 #6
        combined = h.relevance * weights.relevance + imp * weights.importance + rec * weights.recency
        out.append(ScoredHit(
            path=h.path, relevance=h.relevance, importance=imp, recency=rec,
            combined_score=combined, frontmatter=h.frontmatter,
            last_seen_at=h.last_seen_at,
        ))
    return sorted(out, key=lambda s: s.combined_score, reverse=True)
```

### Manifest fragment

```json
// .cyberos-memory/manifest.json (excerpt)
{
  "store_version": "2.0.0",
  "recall_weights": {
    "relevance":     0.4,
    "importance":    0.3,
    "recency":       0.3,
    "decay_profile": "exponential",
    "decay_params":  {"decay_factor": 0.995}
  }
}
```

---

## §4 — Acceptance criteria

1. **Default weights** — `RecallWeights()` constructs to `(0.4, 0.3, 0.3)` with `decay_profile="exponential"`. *(traces_to: §1 #1)*
2. **Weights sum-to-1.0 constraint** — `RecallWeights(0.5, 0.3, 0.3)` (sum 1.1) raises `ValueError`; `RecallWeights(0.4, 0.3, 0.3)` succeeds. *(traces_to: §1 #1)*
3. **Weights bounded to [0, 1]** — `RecallWeights(-0.1, 0.55, 0.55)` raises (negative + still sums to 1.0); `RecallWeights(1.1, -0.05, -0.05)` raises. *(traces_to: §1 #1)*
4. **Relevance passes through unchanged** — `score_hits([Hit(relevance=0.84, importance=None, last_seen_at=now)])` annotates `relevance=0.84`. *(traces_to: §1 #2)*
5. **Absent importance defaults to 0.5** — Hit with no `importance` in frontmatter → `ScoredHit.importance == 0.5`. *(traces_to: §1 #3, DEC-192)*
6. **Importance out-of-range rejected at walker** — handcraft a memory file with `importance: 1.5` → `cyberos doctor` fails with `importance-range`. *(traces_to: §1 #3, §1 #12)*
7. **Exponential half-life ≈ 138 h** — `Exponential(0.995).half_life_hours` is within `[137.8, 138.5]`. *(traces_to: §1 #4)*
8. **Exponential at hours_old=0 → 1.0** — `Exponential()(0.0) == 1.0`. *(traces_to: §1 #4)*
9. **Exponential monotonic decreasing** — for `h1 < h2`, `Exponential()(h1) > Exponential()(h2)`. *(traces_to: §1 #4)*
10. **Ebbinghaus at 0 → 1.0; finite for large h** — `Ebbinghaus()(0) == 1.0`; `Ebbinghaus()(10000) > 0`. *(traces_to: §1 #4)*
11. **Decay parameter validation** — `Exponential(decay_factor=1.5)` raises; `Exponential(decay_factor=-0.1)` raises; `Ebbinghaus(strength=0)` raises. *(traces_to: §1 #4, §1 #9)*
12. **last_seen_at absent → recency=1.0** — `score_hits([Hit(last_seen_at=None)], ...)` annotates `recency == 1.0`. *(traces_to: §1 #6)*
13. **last_seen_at in the future → recency=1.0** — caps at 1.0 rather than > 1 or NaN. *(traces_to: §1 #4)*
14. **Combined-score formula** — `Hit(relevance=0.8, importance=0.6, recency=0.7)` with weights `(0.4, 0.3, 0.3)` → `combined == 0.8·0.4 + 0.6·0.3 + 0.7·0.3 == 0.71`. *(traces_to: §1 #1)*
15. **Ordering** — three hits with combined 0.91, 0.50, 0.70 → sorted output is `[0.91, 0.70, 0.50]`. *(traces_to: §1 #1)*
16. **Profile override at CLI** — `cyberos recall-similar X --decay-profile ebbinghaus` produces different `recency` than default `exponential` on the same hit. *(traces_to: §1 #5)*
17. **Manifest weight fail-fast at construction** — `manifest.json` with `recall_weights: {relevance: 0.5, importance: 0.5, recency: 0.5}` → `Writer(...)` raises `ManifestError` with a structured message naming `recall_weights.sum`. *(traces_to: §1 #9)*
18. **Manifest unknown profile fail-fast** — `decay_profile: "made_up"` → `ManifestError`. *(traces_to: §1 #9)*
19. **Pure function — `now` injected** — `score_hits(..., now=fixed_dt)` returns the same scores on two consecutive calls; no `datetime.now()` inside. *(traces_to: §1 #10)*
20. **RecallHit annotations preserved** — `ScoredHit` exposes `relevance`, `importance`, `recency`, `combined_score`. *(traces_to: §1 #8)*
21. **Back-compat — relevance-only mode** — `score_hits(hits, RecallWeights.relevance_only(), ...)` orders identically to pre-FR sort-by-relevance. *(traces_to: §1 #7)*
22. **Bench latency budget** — `bench_recall_latency.py --episodes 10000 --trials 100` records ranking-overhead p95 ≤ 10% of retrieval p95. *(traces_to: §1 #11)*
23. **`meta.importance` on a `facts` memory** — non-Episode kind accepts the optional `importance` field; round-trips through reader. *(traces_to: §1 #12)*
24. **FR-MEMORY-112 CLI calls real ranking** — after migration, `cyberos recall-similar` returns identical `combined_score` to `score_hits()` invoked directly with same inputs (regression vs the FR-MEMORY-112 stub). *(traces_to: §1 #1)*

---

## §5 — Verification

```python
# modules/memory/tests/test_ranking_combined_score.py
import pytest
from datetime import datetime, timedelta, timezone
from cyberos.core.ranking import RecallWeights, ScoredHit, score_hits
from cyberos.core.decay   import Exponential, Ebbinghaus


def make_hit(relevance, importance=None, last_seen_at=None, path="x"):
    """Shim — real test conftest provides RecallHit factory."""
    from types import SimpleNamespace
    fm = {} if importance is None else {"importance": importance}
    return SimpleNamespace(path=path, relevance=relevance, frontmatter=fm, last_seen_at=last_seen_at)


def test_default_weights():
    """AC #1"""
    w = RecallWeights()
    assert (w.relevance, w.importance, w.recency) == (0.4, 0.3, 0.3)


@pytest.mark.parametrize("w,err", [
    ((0.5, 0.3, 0.3), True),     # sum 1.1
    ((0.4, 0.3, 0.3), False),
    ((-0.1, 0.55, 0.55), True),  # negative
    ((1.1, -0.05, -0.05), True), # out of range
])
def test_weight_validation(w, err):
    """AC #2 + #3"""
    if err:
        with pytest.raises(ValueError):
            RecallWeights(*w)
    else:
        RecallWeights(*w)


def test_importance_defaults_to_half():
    """AC #5"""
    now = datetime.now(timezone.utc)
    h = make_hit(relevance=0.8, importance=None, last_seen_at=now)
    s = score_hits([h], RecallWeights(), Exponential(), now=now)
    assert s[0].importance == 0.5


def test_last_seen_none_recency_one():
    """AC #12"""
    now = datetime.now(timezone.utc)
    h = make_hit(relevance=0.5, last_seen_at=None)
    s = score_hits([h], RecallWeights(), Exponential(), now=now)
    assert s[0].recency == 1.0


def test_last_seen_future_recency_one():
    """AC #13"""
    now = datetime.now(timezone.utc)
    future = now + timedelta(hours=24)
    h = make_hit(relevance=0.5, last_seen_at=future)
    s = score_hits([h], RecallWeights(), Exponential(), now=now)
    assert s[0].recency == 1.0


def test_combined_score_formula():
    """AC #14"""
    now = datetime.now(timezone.utc)
    h = make_hit(relevance=0.8, importance=0.6, last_seen_at=now)
    # last_seen_at == now ⇒ recency = exponential(0) = 1.0; we want 0.7 instead
    s = score_hits([h], RecallWeights(0.4, 0.3, 0.3), Exponential(), now=now)
    # With recency=1.0 the answer is 0.8*0.4 + 0.6*0.3 + 1.0*0.3 = 0.80
    assert s[0].combined_score == pytest.approx(0.8)


def test_ordering_descending():
    """AC #15"""
    now = datetime.now(timezone.utc)
    hits = [
        make_hit(relevance=0.1, importance=0.1, last_seen_at=now, path="low"),
        make_hit(relevance=0.9, importance=0.9, last_seen_at=now, path="high"),
        make_hit(relevance=0.5, importance=0.5, last_seen_at=now, path="mid"),
    ]
    s = score_hits(hits, RecallWeights(), Exponential(), now=now)
    assert [h.path for h in s] == ["high", "mid", "low"]


def test_pure_function_no_now_internal():
    """AC #19"""
    now = datetime(2026, 5, 19, tzinfo=timezone.utc)
    hit = make_hit(relevance=0.5, last_seen_at=now - timedelta(hours=10))
    s1 = score_hits([hit], RecallWeights(), Exponential(), now=now)
    s2 = score_hits([hit], RecallWeights(), Exponential(), now=now)
    assert s1[0].combined_score == s2[0].combined_score


def test_relevance_only_mode():
    """AC #21"""
    now = datetime.now(timezone.utc)
    hits = [make_hit(relevance=0.3, importance=0.9, last_seen_at=now),
            make_hit(relevance=0.7, importance=0.1, last_seen_at=now)]
    s = score_hits(hits, RecallWeights.relevance_only(), Exponential(), now=now)
    assert s[0].relevance > s[1].relevance


def test_scored_hit_annotations():
    """AC #20"""
    now = datetime.now(timezone.utc)
    h = make_hit(relevance=0.5, importance=0.7, last_seen_at=now)
    s = score_hits([h], RecallWeights(), Exponential(), now=now)
    assert hasattr(s[0], "relevance")
    assert hasattr(s[0], "importance")
    assert hasattr(s[0], "recency")
    assert hasattr(s[0], "combined_score")
```

```python
# modules/memory/tests/test_decay_profiles.py
import pytest
from cyberos.core.decay import Exponential, Ebbinghaus, build_profile


def test_exponential_half_life():
    """AC #7"""
    e = Exponential(0.995)
    assert 137.8 < e.half_life_hours < 138.5


def test_exponential_zero():
    """AC #8"""
    assert Exponential()(0.0) == 1.0


def test_exponential_monotonic():
    """AC #9"""
    e = Exponential()
    assert e(1.0) > e(2.0) > e(3.0) > e(10.0)


def test_exponential_negative_hours_caps_one():
    """AC #13"""
    assert Exponential()(-10.0) == 1.0


def test_ebbinghaus_zero_one():
    """AC #10"""
    eb = Ebbinghaus()
    assert eb(0.0) == 1.0


def test_ebbinghaus_large_finite():
    """AC #10"""
    assert 0 < Ebbinghaus()(10000.0) < 1.0


@pytest.mark.parametrize("kls,kwargs", [
    (Exponential, {"decay_factor": 1.5}),
    (Exponential, {"decay_factor": -0.1}),
    (Exponential, {"decay_factor": 0.0}),
    (Exponential, {"decay_factor": 1.0}),
    (Ebbinghaus, {"strength": 0}),
    (Ebbinghaus, {"strength": -1.0}),
])
def test_decay_param_validation(kls, kwargs):
    """AC #11"""
    with pytest.raises(ValueError):
        kls(**kwargs)


def test_build_profile_unknown_name():
    with pytest.raises(ValueError):
        build_profile("made_up", {})


def test_build_profile_known_names():
    e1 = build_profile("exponential", {"decay_factor": 0.99})
    e2 = build_profile("ebbinghaus",  {"strength": 100})
    assert e1(0) == 1.0
    assert e2(0) == 1.0
```

```python
# modules/memory/bench/bench_recall_latency.py
import argparse, time, statistics
from cyberos.core.ranking import RecallWeights, score_hits
from cyberos.core.decay   import Exponential


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--episodes", type=int, default=10000)
    p.add_argument("--trials",   type=int, default=100)
    args = p.parse_args()
    # Fixture: 10K mock hits with random relevance / importance / last_seen
    from tests.fixtures.hit_factory import build_hits
    hits = build_hits(args.episodes)
    decay = Exponential()
    weights = RecallWeights()
    durations = []
    for _ in range(args.trials):
        t0 = time.perf_counter()
        score_hits(hits[:20], weights, decay)        # 20-hit recall is the realistic shape
        durations.append((time.perf_counter() - t0) * 1000)
    p95 = statistics.quantiles(durations, n=20)[18]  # 95th percentile
    print(f"ranking-only p95 = {p95:.2f} ms")
    assert p95 < 5.0, f"ranking overhead p95 {p95:.2f}ms above 5ms budget"


if __name__ == "__main__":
    main()
```

---

## §6 — Implementation skeleton

API contracts above are the skeleton. Implementation order:

1. Schema (memory.schema.json): add `Importance`, `RecallWeights`, extend `MemoryFrontmatter`.
2. Walker invariants: `manifest-recall-weights-sum-to-one`, `importance-range`, `decay-factor-range`.
3. `cyberos/core/decay.py`.
4. `cyberos/core/ranking.py`.
5. `cyberos/core/writer.py`: load + validate `manifest.recall_weights` in `__init__`.
6. Swap `cyberos/core/semantic.py` + `cyberos/core/reader.py` to call `score_hits()`.
7. Update `cyberos/cli/recall.py` to delegate to `ranking`.
8. Tests + bench.
9. CHANGELOG entry.

---

## §7 — Dependencies

- **FR-MEMORY-108 (depends on)** — semantic search; we extend its `recall()` not replace it.
- **FR-MEMORY-112 (depends on)** — Episodes are the highest-stakes consumers of this ranking; the FR-MEMORY-112 stub `combined_score` is replaced by `ranking.score_hits`.
- **FR-MEMORY-114 (this FR enables)** — write-time importance scoring writes the `meta.importance` field that this FR consumes.
- **FR-MEMORY-115 (this FR blocks)** — `cyberos dream` calls `score_hits()` against a fixed snapshot timestamp to score candidate episodes in batch.
- **FR-MEMORY-120 (this FR enables)** — `cyberos history` surfaces the four scalar annotations so the agent can explain "why did this rank first?".

---

## §8 — Example payloads

### Manifest

```json
// .cyberos-memory/manifest.json (excerpt)
{
  "recall_weights": {
    "relevance":     0.4,
    "importance":    0.3,
    "recency":       0.3,
    "decay_profile": "exponential",
    "decay_params":  {"decay_factor": 0.995}
  }
}
```

### Recall response with ranking annotations

```json
{
  "backend": "semantic",
  "weights": {"relevance": 0.4, "importance": 0.3, "recency": 0.3},
  "decay":   {"profile": "exponential", "decay_factor": 0.995},
  "matches": [
    {
      "path": "memories/episodes/d4/12/d4127a3b-1f2c3d.md",
      "task": "Ship FR-AUTH-003 RLS enforcement",
      "relevance":      0.84,
      "importance":     0.92,
      "recency":        0.998,
      "combined_score": 0.913,
      "last_seen_at": "2026-05-18T20:55:13Z"
    }
  ]
}
```

### Walker error on bad manifest

```text
ManifestError: recall_weights.sum != 1.0 (relevance=0.5 importance=0.5 recency=0.5 sum=1.5);
  fix manifest.json:recall_weights or set CYBEROS_RECALL_WEIGHTS=auto to use defaults
```

---

## §9 — Open questions

All resolved. Deferred:
- Third-party decay profiles via `entry_points` — §1 #13; slice 4+.
- `memory.recall.scored` OTel span — §1 #14; gated by FR-OBS-001.
- Per-kind weight overrides (e.g. `decisions` ranks recency higher than `facts`) — slice 4+; needs schema extension.
- A/B comparison harness that runs same query under multiple weight triples — slice 4+; ops tool, not a runtime feature.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Manifest weights sum != 1.0 | `Writer.__init__` ManifestError | Daemon refuses to start | Operator fixes manifest |
| Manifest unknown profile name | same | same | Operator names a supported profile |
| Decay param out of range in manifest | profile constructor raises | same | Operator fixes param |
| Memory file `importance` field out of range | walker `importance-range` | `cyberos doctor` non-zero | Operator fixes file (or `--repair` clamps to nearest valid) |
| `last_seen_at` missing | recency fallback to 1.0 | Memory ranks as fresh | None — by design |
| `last_seen_at` unparseable timestamp | reader skips → None | recency fallback to 1.0 | Operator fixes timestamp |
| `now` in past relative to `last_seen_at` (clock skew) | profile receives negative hours_old; caps at recency=1.0 | Memory ranks as fresh | None — by design |
| Pure-function violation (someone adds time.now() in score_hits) | unit test `test_pure_function_no_now_internal` | CI blocked | Author removes the call |
| Ranking overhead exceeds 10% budget | bench `bench_recall_latency.py` asserts | CI blocked | Profile + optimise; likely cause is unnecessary I/O per-hit |
| Two hits with identical combined_score | Python sort is stable | Insertion order preserved | None — by design (acceptable non-determinism) |
| Hit with `importance` defined but as a string ("0.5") | jsonschema fails | walker invariant `importance-range` (with type check) | Operator fixes file |
| Default-decay user wants Ebbinghaus | manifest update | next recall uses new profile | None |
| `decay_params` provided for wrong profile (e.g. `strength` on exponential) | profile constructor ignores unknown kwargs | exponential uses default decay_factor | If operator-intended, surface via doctor warning |
| Catastrophic decay (decay_factor → 0) | profile constructor allows | every hit gets recency ≈ 0; combined_score ≈ relevance * w_r | Operator picks a reasonable profile |
| Ranking applied to legacy code path that expected RecallHit shape | type-checked at score_hits call | TypeError; CI catches | Author updates call site |
| Bench fixture build_hits drifts from real Hit shape | bench test parametrizes against current Hit | CI catches | Author updates fixture |
| Weight tuple loaded from manifest with extra keys | jsonschema strict mode rejects | `ManifestError` | Operator removes extra keys |

---

## §11 — Implementation notes

- **Why we don't memoize decay results.** The decay function is so cheap (one `pow` or `exp` call) that memoization adds more overhead than it saves on the typical 20-hit recall.
- **`Exponential.half_life_hours` property** — exposed because operators want to reason about decay in human terms ("a memory at 1 week old has X% recency"). The math is in the type so the operator doesn't have to redo it.
- **Why `recall_weights` lives on `manifest.json`, not a separate config file.** Manifest is already loaded at writer construction; reusing it keeps the failure surface narrow. A separate config means one more file to manage + one more invariant.
- **`build_profile()` is the slice-4 plug-point** — when third-party profiles land via `entry_points`, `build_profile()` becomes the dispatcher. For now it's a stub if/elif because the slice-3 surface is exactly two profiles.
- **Bench fixture lives in `tests/fixtures/hit_factory.py`** — generates deterministic hits given a seed so the bench is reproducible across runs. Seed: `42`.
- **The `test_pure_function_no_now_internal` test is the load-bearing invariant** — without it, someone will eventually add `datetime.now()` inside `score_hits` and FR-MEMORY-115's batch dream will start producing non-deterministic scores. The unit test prevents that regression.
- **Ranking is applied AFTER the engine's k-limit, not before.** Engines retrieve top-k by their own internal score (relevance for vector, BM25 for FTS); we then re-rank with the combined score. This means the very top of the underlying engine's rank survives — the post-ranking is a fine-grained reordering, not a wholesale reordering.
- **The 10% overhead budget is generous** — empirically the operation is sub-millisecond for typical k=20. The budget exists to catch accidental regressions (e.g. someone forgetting to call `last_seen_at` from a derived index and going back to the filesystem).

---

*End of FR-MEMORY-113.*
