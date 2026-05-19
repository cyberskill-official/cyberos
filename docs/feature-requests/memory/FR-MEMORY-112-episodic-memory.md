---
id: FR-MEMORY-112
title: "memory episodic memory — `kind: episode` frontmatter + `cyberos recall-similar` API; task / approach / outcome / quality_score per Episode; reflection-loop foundation"
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
related_frs: [FR-MEMORY-108, FR-MEMORY-109, FR-MEMORY-113, FR-MEMORY-114, FR-MEMORY-115, FR-MEMORY-120]
depends_on: [FR-MEMORY-108]
blocks: [FR-MEMORY-113, FR-MEMORY-114, FR-MEMORY-115]

source_pages:
  - docs/proposals/MEMORY-IMPROVEMENT-WAVE-2026Q3.md#section-21-the-x-article
  - playground/extracts/agentic-memory.article.txt
source_decisions:
  - DEC-180 (Add a new top-level memory kind `episode` instead of overloading `facts` or `decisions`; the four existing kinds describe knowledge, episode describes events)
  - DEC-181 (Quality score is a float 0.0–1.0; absent ≡ unscored ≡ 0.5 default — never assume "good" or "bad" silently)
  - DEC-182 (Outcome is a closed enum `success | partial | failure`; ambiguity rejected at write time so dashboards aren't lying)
  - DEC-183 (`recall-similar` uses semantic search filtered to `kind: episode`; falls back to FTS when sentence-transformers is unavailable per FR-MEMORY-108's soft-dep pattern)

language: python 3.10
service: modules/memory/cyberos/
new_files:
  - modules/memory/cyberos/core/episode.py
  - modules/memory/cyberos/cli/recall.py
  - modules/memory/tests/test_episode_log_and_recall.py
  - modules/memory/tests/test_episode_schema.py
  - modules/memory/tests/fixtures/episode_corpus.jsonl
modified_files:
  - modules/memory/memory.schema.json           # add `episode` to MemoryKind enum + per-kind required fields
  - modules/memory/cyberos/__main__.py          # wire `cyberos episode log` + `cyberos recall-similar` subcommands
  - modules/memory/cyberos/core/semantic.py     # accept `memory_kind=` filter; preserve back-compat
  - modules/memory/cyberos/core/walker.py       # invariant: every episode has `outcome` ∈ closed enum
  - modules/memory/cyberos/core/frontmatter.py  # validate per-kind frontmatter contract
  - modules/memory/memory.invariants.yaml       # add `episode-outcome-closed-enum` + `episode-quality-score-range`
  - modules/memory/AGENTS.md                    # §2 layout — add `memories/episodes/` to the kind list (additive, see §0.2 not amended; no normative behaviour change)
  - modules/memory/CHANGELOG.md                 # release note for `kind: episode`
allowed_tools:
  - file_read: modules/memory/**
  - file_write: modules/memory/cyberos/**, modules/memory/tests/**, modules/memory/memory.schema.json, modules/memory/memory.invariants.yaml, modules/memory/AGENTS.md, modules/memory/CHANGELOG.md
  - bash: cd modules/memory && python -m pytest tests/test_episode_log_and_recall.py tests/test_episode_schema.py -v
  - bash: cd modules/memory && python -m cyberos --store /tmp/memory doctor
disallowed_tools:
  - emit episodes whose `outcome` is outside the closed enum (per §1 #5 — walker rejects)
  - silently default `quality_score` to anything other than the literal `0.5` (per DEC-181)
  - mutate the canonical writer to accept `kind: episode` rows that bypass the per-kind frontmatter contract (§1 #2)

effort_hours: 12
sub_tasks:
  - "1.0h: memory.schema.json — extend MemoryKind enum with `episode`; add `EpisodeFrontmatter` definition (task, approach, outcome enum, duration_ms, token_cost, quality_score, notes, error fields)"
  - "1.0h: memory.invariants.yaml — `episode-outcome-closed-enum` (error), `episode-quality-score-range` (error: 0.0 ≤ q ≤ 1.0), `episode-duration-non-negative` (error)"
  - "1.5h: cyberos/core/episode.py — `Episode` msgspec.Struct + `log(store, episode) -> memory_path`; computes the searchable document representation (task + approach + outcome + notes joined with newlines so FTS5 + sentence-transformers both see them)"
  - "1.0h: cyberos/core/frontmatter.py — per-kind validators; `validate_episode(meta)` checks outcome ∈ enum, quality_score range, duration ≥ 0, task non-empty"
  - "1.0h: cyberos/core/semantic.py — extend `recall(memory_kind=)` filter; existing call sites unaffected (kind=None retains current behaviour)"
  - "1.5h: cyberos/cli/recall.py — `cyberos recall-similar <task> [--k 3] [--min-relevance 0.65]`; ranks by relevance · 0.4 + quality_score · 0.3 + recency · 0.3 (recency mock for now; FR-MEMORY-113 will fold proper decay in)"
  - "1.0h: cyberos/__main__.py — wire `episode log`, `recall-similar` subcommands; existing `cyberos put memories/episodes/x.md -` continues to work; the new subcommand is a strict convenience that fills in the frontmatter"
  - "1.5h: tests/test_episode_log_and_recall.py — 12 cases (log success/partial/failure, recall ranks correctly, FTS-only fallback when sentence-transformers missing, top-K cap, min_relevance cutoff, quality_score absence defaults to 0.5)"
  - "1.0h: tests/test_episode_schema.py — 8 cases (outcome enum closed, quality_score range guard, duration_ms negative rejected, task empty rejected, frontmatter `kind: episode` round-trips through reader)"
  - "0.5h: tests/fixtures/episode_corpus.jsonl — 40-row fixture: 10 success, 10 partial, 10 failure, 10 edge (zero duration, quality_score=0.0, etc.)"
  - "0.5h: walker.py — wire the new invariants so `cyberos doctor` catches violations the same way it catches frontmatter drift"
  - "0.5h: AGENTS.md + CHANGELOG.md — additive doc updates; AGENTS.md §2 gets one line for `memories/episodes/`; no §0.2 protocol-amendment chat-turn required because adding to the closed enum is exactly the §5.2 evolution path"
risk_if_skipped: "Without episodic memory the memory can never answer the question 'have I tried this kind of task before, and did my approach work?'. Every other improvement in the 2026-Q3 wave compounds against this: FR-MEMORY-113 (recency decay) and FR-MEMORY-114 (importance scoring) need a clean Episode shape to score against; FR-MEMORY-115 (cyberos dream) consumes Episodes when looking for cross-session patterns ('5 sessions all hit this 60-second retry'); FR-MEMORY-120 (cyberos history) needs Episode rows to surface 'last time you tried this approach it took 4× longer than this run'. Shipping the rest of the wave without FR-MEMORY-112 forces every downstream FR to either (a) hand-roll an Episode-shaped memory inline (drift inevitable) or (b) operate on raw audit rows (cardinality + semantic mismatch). The article's reflection loop (Ramakrushna §3) and the Anthropic talk's dreaming pipeline both pre-suppose this kind. Defer this and the wave becomes incoherent."
---

## §1 — Description (BCP-14 normative)

A new memory kind `episode` is the **agent's record of one completed task**. It is structurally distinct from the four existing kinds (`decisions | facts | people | projects | preferences | drift | refinements`) which describe knowledge; an Episode describes an *event*. The Episode contract:

1. **MUST** add `episode` to `memory.schema.json#/definitions/MemoryKind`. Files MUST live under `memories/episodes/<hex>/<hex>/<slug>.md` per the existing path convention; the writer enforces this layout via `memory.invariants.yaml#layout-kind-directory-match`.
2. **MUST** validate against a per-kind frontmatter contract (`memory.schema.json#/definitions/EpisodeFrontmatter`) whose fields are:

    | Field | Type | Required | Notes |
    |---|---|---|---|
    | `task` | string, ≥ 1 char after trim | yes | The task the agent was asked to do. Free-form. |
    | `approach` | string, ≥ 1 char after trim | yes | One-line description of how the agent approached it. |
    | `outcome` | enum `success \| partial \| failure` | yes | Closed enum (§1 #5). |
    | `duration_ms` | integer ≥ 0 | yes | Wall-clock duration. |
    | `token_cost` | integer ≥ 0 | optional | Input + output tokens combined; absent ≡ unknown. |
    | `quality_score` | float `0.0..1.0` | optional | Absent ≡ 0.5 (DEC-181). |
    | `notes` | string | optional | Free-form observations. |
    | `error` | string | optional, required iff `outcome != success` | Concrete failure description. |

3. **MUST** preserve the existing `MemoryFrontmatter` shape for non-episode kinds verbatim; the per-kind contract is **additive** — old memories continue to validate.
4. **MUST** route every Episode write through `cyberos.core.writer.Writer` (the canonical writer). The new convenience function `cyberos.core.episode.log(store, episode)` constructs the body + frontmatter and then calls `Writer.put(...)`; it MUST NOT touch `audit/`, `HEAD`, or `.lock` directly (per AGENTS.md §14.1).
5. **MUST** reject Episode rows whose `outcome` value is outside the closed enum. The walker's `episode-outcome-closed-enum` invariant runs on every `cyberos doctor`. Severity: error.
6. **MUST** reject Episode rows whose `quality_score` is outside `[0.0, 1.0]` or `duration_ms` is negative. Same invariant pattern. Severity: error.
7. **MUST** treat the searchable document for an Episode as `f"Task: {task}\nApproach: {approach}\nOutcome: {outcome}\nNotes: {notes}"` (mirrors the article's Episode dataclass §3). This is what the FTS5 index sees and what sentence-transformers embeds.
8. **MUST** add a new CLI subcommand `cyberos episode log` that wraps `cyberos.core.episode.log(...)`:
    ```
    cyberos episode log \
      --task "Audit MEMORY module for self-learning gaps" \
      --approach "MHTML + ffmpeg/whisper extraction; cross-ref vs AGENTS.md" \
      --outcome success \
      --duration-ms 1847000 \
      --token-cost 145000 \
      --quality-score 0.92
    ```
9. **MUST** add a new CLI subcommand `cyberos recall-similar <task-string>` that:
    - Runs semantic search filtered to `kind: episode` (semantic backend if `sentence-transformers` is installed; FTS5 fallback otherwise — same soft-dep pattern as FR-MEMORY-108).
    - Ranks results by the **combined score** `relevance · 0.4 + quality_score · 0.3 + recency · 0.3`. For FR-MEMORY-112's scope, `recency` is a placeholder mock that returns `1.0` (no decay); FR-MEMORY-113 plugs in the proper Park-et-al decay function.
    - Defaults: `--k 3`, `--min-relevance 0.65`.
    - Output: JSON list of `{path, task, approach, outcome, quality_score, relevance, combined_score, last_seen_at}` ordered by `combined_score` descending.
10. **MUST** treat absent `quality_score` as exactly `0.5` in the combined-score ranking (the "no opinion" midpoint, per DEC-181). Never silently boost or penalise.
11. **MUST** preserve current `cyberos recall` (full-text) and `cyberos search` (semantic) semantics. The new `recall-similar` is a strictly additive subcommand; existing call sites in `cyberos.core.semantic.recall(...)` continue to work without the `memory_kind=` kwarg (defaults to `None` ≡ "all kinds").
12. **MUST** emit one audit row per `episode log` invocation of kind `episode.logged` with payload `{path, outcome, duration_ms, token_cost, quality_score}`. The writer already emits the standard `put` row; the `episode.logged` row is a **lightweight projection** for FR-MEMORY-115 to consume without re-parsing every `put` payload.
13. **MUST** ship a labelled fixture corpus (`tests/fixtures/episode_corpus.jsonl`, ≥ 40 rows) covering: 10 success + 10 partial + 10 failure + 10 edge cases (zero duration, quality 0.0 / 1.0, missing optional fields, multi-line task strings). CI gates on the fixture parsing cleanly through the schema.
14. **MUST** complete `cyberos recall-similar` in ≤ 500 ms p95 on a memory with 10,000 Episodes (semantic backend) or ≤ 150 ms p95 (FTS5 fallback). Both budgets measured on commodity hardware against the fixture corpus expanded to 10K rows.
15. **SHOULD** support `cyberos episode log --from-json <file>` for batch-loading Episodes from external systems (e.g. importing historical run data from a CI export). Slice-3 stretch; placeholder noted for FR-MEMORY-115 forward-compatibility.
16. **SHOULD** surface "no similar episodes found" as a structured response rather than silent emptiness, so an agent calling `recall-similar` programmatically can branch: `{matches: [], reason: "no_episodes_above_min_relevance"}` (or `no_episodes_in_store` if the store has zero Episodes).

---

## §2 — Why this design (rationale for humans)

**Why a new kind instead of overloading `facts` (§1 #1)?** Facts describe what is true (durable state); Episodes describe what happened (events). Overloading would force the per-kind validator to special-case `meta.outcome` on `kind: facts`, which is exactly the kind of schema drift `memory.schema.json` is supposed to prevent. A new kind is cheap (additive enum value) and structurally correct.

**Why a closed outcome enum (§1 #5)?** The article (Ramakrushna §3) defines outcome as `success | partial | failure`. Open-text outcomes turn dashboards into NLP problems: counting "partial" vs "partial success" vs "kinda worked" vs "almost-success". Closed enum keeps aggregation arithmetic, not heuristic. Anything ambiguous gets rejected at write time so the agent has to pick a side.

**Why default missing `quality_score` to 0.5 (§1 #10, DEC-181)?** Three options were considered: (a) default to 1.0 ("optimistic, treat unscored as good"); (b) default to 0.0 ("pessimistic, treat unscored as bad"); (c) default to 0.5 ("no opinion"). The article's combined-score formula `relevance · 0.4 + importance · 0.3 + recency · 0.3` is sensitive to score weight; defaulting to either extreme would silently up-rank or down-rank unscored Episodes against scored ones. 0.5 is the only neutral value. Documented in the §3 type contract so downstream FRs (especially FR-MEMORY-113's recall ranking) don't surprise the operator.

**Why ship `recall-similar` separately from `recall` (§1 #11)?** Two reasons. First, `recall` is general (any `kind`) — adding kind-specific logic to it would push complexity into a path most callers don't need. Second, `recall-similar` is the load-bearing primitive for the reflection loop (Ramakrushna's §"Reflection loop"); separating it keeps the call site readable: "give me episodes similar to this current task" is one mental operation, "search the store" is another.

**Why include `notes` in the searchable document (§1 #7)?** The article's reference implementation joins `Task / Approach / Outcome / Notes` into the embedded text precisely because notes contain the "what would I do differently next time?" signal that makes episodes useful. Excluding notes would mean an episode whose task was identical to a current one but whose lesson is in the notes never surfaces.

**Why a separate `episode.logged` audit row (§1 #12)?** FR-MEMORY-115 (`cyberos dream`) consumes episodes when looking for cross-session patterns. The `put` row's payload contains the entire body, which means dream has to re-parse markdown frontmatter on every row scan — expensive. The `episode.logged` row is a lightweight projection that exposes outcome / duration / quality_score directly, so dream's pattern detector reads compact data. Cost: one extra audit row per episode log. Benefit: ~50× faster dream-time pattern scanning over 10K+ episode stores.

**Why ≤ 500 ms p95 semantic / ≤ 150 ms FTS (§1 #14)?** Agents calling `recall-similar` synchronously inside their task loop need this to feel cheap. 500 ms is the same ceiling FR-MEMORY-108 sets for `cyberos search --semantic`; piggybacking the budget keeps memory-side latency consistent. 150 ms FTS is a 3× margin — FTS5 is dramatically faster than vector for filtered queries.

**Why an additive enum extension rather than an §0.2 protocol amendment?** AGENTS.md §5.2 says: *"The schema's `kind` field is closed; unknown values MUST be rejected."* Adding a new kind value extends the closed enum — that is exactly the schema-evolution path §5.2 names. It is not a behaviour change to the protocol; the protocol's invariant ("kind is closed") is preserved. Distinguished from FR-MEMORY-115 / 117 / 118 / 119 which DO require §0.2 chat-turns because they change writer behaviour, add new ops, or introduce new normative sections.

**Why 12 hours of effort, not the 16 hours of FR-MEMORY-107?** The schema change is mechanical (one enum value + one new struct). The CLI surface is two subcommands. The tests are mostly fixture-driven. No new daemon, no new threading model, no new network surface. The cost is in (a) the per-kind validator pattern (§1 #2) which establishes infrastructure FR-MEMORY-114 will reuse, and (b) the fixture corpus (§1 #13).

---

## §3 — API contract

### `EpisodeFrontmatter` schema (memory.schema.json fragment)

```json
{
  "$defs": {
    "MemoryKind": {
      "type": "string",
      "enum": ["decisions", "facts", "people", "projects", "preferences", "drift", "refinements", "episode"]
    },
    "EpisodeOutcome": {
      "type": "string",
      "enum": ["success", "partial", "failure"]
    },
    "EpisodeFrontmatter": {
      "type": "object",
      "required": ["kind", "task", "approach", "outcome", "duration_ms"],
      "properties": {
        "kind":          {"const": "episode"},
        "task":          {"type": "string", "minLength": 1},
        "approach":      {"type": "string", "minLength": 1},
        "outcome":       {"$ref": "#/$defs/EpisodeOutcome"},
        "duration_ms":   {"type": "integer", "minimum": 0},
        "token_cost":    {"type": "integer", "minimum": 0},
        "quality_score": {"type": "number",  "minimum": 0.0, "maximum": 1.0},
        "notes":         {"type": "string"},
        "error":         {"type": "string"}
      },
      "allOf": [
        {
          "if":   {"properties": {"outcome": {"const": "success"}}},
          "then": true,
          "else": {"required": ["error"]}
        }
      ]
    }
  }
}
```

### `cyberos.core.episode` (Python)

```python
# modules/memory/cyberos/core/episode.py
from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Literal, Optional
import hashlib, json, uuid

from cyberos.core.writer import Writer

EpisodeOutcome = Literal["success", "partial", "failure"]


@dataclass
class Episode:
    task:         str
    approach:     str
    outcome:      EpisodeOutcome
    duration_ms:  int
    token_cost:   Optional[int]   = None
    quality_score: Optional[float] = None
    notes:        str             = ""
    error:        Optional[str]   = None

    def __post_init__(self) -> None:
        if not self.task.strip():
            raise ValueError("Episode.task must be non-empty after trim")
        if not self.approach.strip():
            raise ValueError("Episode.approach must be non-empty after trim")
        if self.outcome not in ("success", "partial", "failure"):
            raise ValueError(f"Episode.outcome {self.outcome!r} not in closed enum")
        if self.duration_ms < 0:
            raise ValueError("Episode.duration_ms must be ≥ 0")
        if self.quality_score is not None and not (0.0 <= self.quality_score <= 1.0):
            raise ValueError("Episode.quality_score must be in [0.0, 1.0]")
        if self.outcome != "success" and not (self.error or "").strip():
            raise ValueError(f"Episode.outcome={self.outcome!r} requires non-empty error")

    def searchable_document(self) -> str:
        """The text the FTS / sentence-transformers index sees."""
        return (
            f"Task: {self.task}\n"
            f"Approach: {self.approach}\n"
            f"Outcome: {self.outcome}\n"
            f"Notes: {self.notes}"
        )

    def frontmatter(self) -> dict:
        d: dict = {
            "kind":        "episode",
            "task":        self.task,
            "approach":    self.approach,
            "outcome":     self.outcome,
            "duration_ms": self.duration_ms,
        }
        if self.token_cost is not None:    d["token_cost"]    = self.token_cost
        if self.quality_score is not None: d["quality_score"] = self.quality_score
        if self.notes:                     d["notes"]         = self.notes
        if self.error:                     d["error"]         = self.error
        return d


def log(writer: Writer, episode: Episode, actor: str = "agent") -> Path:
    """Append an Episode to the memory. Returns the relative path under `<memory-root>/`."""
    short = hashlib.sha256(episode.task.encode("utf-8")).hexdigest()
    slug  = f"{short[:8]}-{uuid.uuid4().hex[:6]}.md"
    path  = Path("memories") / "episodes" / short[:2] / short[2:4] / slug

    body  = "---\n" + json.dumps(episode.frontmatter(), indent=2, sort_keys=True) + "\n---\n\n" + episode.searchable_document() + "\n"

    # `put` emits the standard audit row; `episode.logged` is an additional lightweight projection
    writer.put(str(path), body, actor=actor)
    writer.emit_aux(
        kind="episode.logged",
        payload={
            "path":          str(path),
            "outcome":       episode.outcome,
            "duration_ms":   episode.duration_ms,
            "token_cost":    episode.token_cost,
            "quality_score": episode.quality_score if episode.quality_score is not None else 0.5,
            "logged_at":     datetime.now(timezone.utc).isoformat(),
        },
        actor=actor,
    )
    return path
```

### `cyberos.core.semantic.recall(..., memory_kind=)` extension

```python
# modules/memory/cyberos/core/semantic.py — extension diff (additive)

def recall(
    store_path: Path,
    query:      str,
    k:          int   = 5,
    min_relevance: float = 0.65,
    memory_kind:   Optional[str] = None,       # ← NEW; None ≡ all kinds (back-compat)
) -> list[RecallHit]:
    ...
    # filter step (added)
    if memory_kind is not None:
        hits = [h for h in hits if h.frontmatter.get("kind") == memory_kind]
    ...
```

### `cyberos recall-similar` CLI

```python
# modules/memory/cyberos/cli/recall.py
import argparse, json, time
from pathlib import Path
from cyberos.core.semantic import recall as semantic_recall
from cyberos.core.semantic import available as semantic_available
from cyberos.core.reader   import recall as fts_recall  # FTS5 fallback


def add_args(sub: argparse.ArgumentParser) -> None:
    sub.add_argument("task", help="Task description to find similar episodes for")
    sub.add_argument("--k",   type=int,   default=3)
    sub.add_argument("--min-relevance", type=float, default=0.65)
    sub.add_argument("--json", action="store_true", help="Emit JSON instead of human-formatted output")


def combined_score(relevance: float, quality_score: Optional[float], recency: float) -> float:
    qs = quality_score if quality_score is not None else 0.5     # DEC-181
    return relevance * 0.4 + qs * 0.3 + recency * 0.3


def run(store: Path, args: argparse.Namespace) -> dict:
    backend = "semantic" if semantic_available() else "fts5"
    hits = (semantic_recall if backend == "semantic" else fts_recall)(
        store_path=store,
        query=args.task,
        k=args.k * 4,                                            # over-fetch then filter
        min_relevance=args.min_relevance,
        memory_kind="episode",
    )
    enriched = []
    for h in hits:
        fm = h.frontmatter
        enriched.append({
            "path":           h.path,
            "task":           fm["task"],
            "approach":       fm["approach"],
            "outcome":        fm["outcome"],
            "quality_score":  fm.get("quality_score"),
            "relevance":      round(h.relevance, 3),
            "recency":        1.0,                               # FR-MEMORY-113 plugs in the real decay
            "combined_score": round(combined_score(h.relevance, fm.get("quality_score"), 1.0), 3),
            "last_seen_at":   h.last_seen_at,
        })
    enriched.sort(key=lambda x: x["combined_score"], reverse=True)
    enriched = enriched[:args.k]
    return {
        "backend": backend,
        "matches": enriched,
        "reason":  None if enriched else (
            "no_episodes_in_store"        if backend == "semantic" and not hits else
            "no_episodes_above_min_relevance"
        ),
    }
```

### `cyberos episode log` CLI

```python
# modules/memory/cyberos/cli/episode.py
import argparse
from cyberos.core.episode import Episode, log as episode_log
from cyberos.core.writer  import Writer


def add_args(sub: argparse.ArgumentParser) -> None:
    sub.add_argument("--task",          required=True)
    sub.add_argument("--approach",      required=True)
    sub.add_argument("--outcome",       required=True, choices=["success", "partial", "failure"])
    sub.add_argument("--duration-ms",   required=True, type=int)
    sub.add_argument("--token-cost",    type=int)
    sub.add_argument("--quality-score", type=float)
    sub.add_argument("--notes",         default="")
    sub.add_argument("--error",         default=None)


def run(writer: Writer, args: argparse.Namespace, actor: str) -> str:
    ep = Episode(
        task=args.task, approach=args.approach, outcome=args.outcome,
        duration_ms=args.duration_ms, token_cost=args.token_cost,
        quality_score=args.quality_score, notes=args.notes, error=args.error,
    )
    return str(episode_log(writer, ep, actor=actor))
```

---

## §4 — Acceptance criteria

1. **Schema accepts new kind** — `episode` is a valid `MemoryKind`; `jsonschema.validate(meta, MemoryKind)` passes for `episode` and continues to pass for every existing kind. *(traces_to: §1 #1)*
2. **Episode frontmatter required fields** — a memory file with `kind: episode` and any of `{task, approach, outcome, duration_ms}` missing → walker fails with rule id `episode-frontmatter-required-field`. *(traces_to: §1 #2)*
3. **Existing memory kinds unchanged** — every existing memory file under `memories/{decisions,facts,...}/` continues to validate against `MemoryFrontmatter` exactly as before the schema migration. Regression suite `tests/test_frontmatter_legacy.py` passes unchanged. *(traces_to: §1 #3)*
4. **Writer routes Episode writes** — `cyberos.core.episode.log(writer, ep)` results in one `put` audit row + one `episode.logged` audit row; HEAD advances by exactly 2 seq positions. No direct file write to `audit/`, `HEAD`, or `.lock`. *(traces_to: §1 #4, §1 #12)*
5. **Outcome enum closed** — `Episode(outcome="success")` accepted; `Episode(outcome="kinda-worked")` raises `ValueError`. Walker rejects file-level violations as error severity. *(traces_to: §1 #5)*
6. **Quality_score range enforced** — `quality_score=1.0` accepted; `quality_score=1.1` rejected at construction; `quality_score=-0.0001` rejected. Same checks at walker time. *(traces_to: §1 #6)*
7. **Duration non-negative** — `duration_ms=0` accepted; `duration_ms=-1` rejected at construction and at walker. *(traces_to: §1 #6)*
8. **Error required on non-success** — `Episode(outcome="failure", error="")` rejected at construction; `Episode(outcome="failure", error="timeout in dispatcher")` accepted. *(traces_to: §1 #2)*
9. **Searchable document shape** — `Episode(...).searchable_document()` matches the exact format `Task: X\nApproach: Y\nOutcome: Z\nNotes: N` (newlines literal). *(traces_to: §1 #7)*
10. **`cyberos episode log` CLI** — happy-path invocation returns a memory path under `memories/episodes/<hex>/<hex>/<slug>.md`; `cyberos read <path>` round-trips frontmatter exactly. *(traces_to: §1 #8)*
11. **`cyberos recall-similar` filters to episode kind** — after writing 5 episodes + 5 facts under similar text, `cyberos recall-similar "..." --k 5` returns only Episodes (zero facts). *(traces_to: §1 #9, §1 #11)*
12. **Combined-score ordering** — two Episodes with relevance 0.80 each but quality_score 0.9 vs 0.4 → the 0.9 one ranks first. *(traces_to: §1 #9)*
13. **Default quality_score = 0.5 for ranking** — Episode without `quality_score` ranks identically to one with `quality_score=0.5`, lower than 0.9, higher than 0.4. *(traces_to: §1 #10, DEC-181)*
14. **Min-relevance cutoff** — three Episodes with relevances 0.80 / 0.70 / 0.60, `--min-relevance 0.65` → top-K returns 2 (the 0.60 is filtered out). *(traces_to: §1 #9)*
15. **Top-K cap** — 10 Episodes match, `--k 3` → exactly 3 returned. *(traces_to: §1 #9)*
16. **FTS5 fallback when sentence-transformers absent** — when `cyberos.core.semantic.available()` returns False, `cyberos recall-similar` runs the FTS5 path; response `backend: "fts5"`. *(traces_to: §1 #9, DEC-183)*
17. **`recall` back-compat** — every existing call site to `cyberos.core.semantic.recall(...)` that does not pass `memory_kind=` continues to return the same set of hits it did before the migration. Regression covered by `tests/test_recall_back_compat.py`. *(traces_to: §1 #11)*
18. **`episode.logged` projection row shape** — the aux audit row's payload matches `{path, outcome, duration_ms, token_cost, quality_score, logged_at}`; `quality_score` is `0.5` literal when the source Episode had it absent. *(traces_to: §1 #12, DEC-181)*
19. **Fixture corpus parses** — `tests/fixtures/episode_corpus.jsonl` has ≥ 40 rows; every row constructs a valid `Episode` and round-trips through `frontmatter()` → schema-validate → reconstruct without loss. *(traces_to: §1 #13)*
20. **Recall-similar latency p95** — 10K-Episode store, 100 trial runs of `cyberos recall-similar` → p95 ≤ 500 ms (semantic) / ≤ 150 ms (FTS). *(traces_to: §1 #14)*
21. **No similar found → structured response** — empty store → `{matches: [], reason: "no_episodes_in_store"}`; non-empty but all below cutoff → `{matches: [], reason: "no_episodes_above_min_relevance"}`. *(traces_to: §1 #16)*
22. **Invariant catches walker violations** — manually inject an Episode file with `outcome: "kinda-worked"` via raw write → `cyberos doctor` reports invariant `episode-outcome-closed-enum` failed; exit code non-zero. *(traces_to: §1 #5, §1 #6)*
23. **CHANGELOG entry exists** — `modules/memory/CHANGELOG.md` has a new dated section for FR-MEMORY-112 referencing `kind: episode` and the two new CLIs. *(traces_to: §1 #2, §1 #8, §1 #9)*

---

## §5 — Verification

```python
# modules/memory/tests/test_episode_log_and_recall.py
import json
from pathlib import Path

import pytest
from cyberos.core.episode  import Episode, log as episode_log
from cyberos.core.semantic import recall as semantic_recall, available as semantic_available
from cyberos.core.writer   import Writer


def test_episode_log_writes_put_and_aux_rows(tmp_memory: Writer):
    """AC #4 + #18 — both `put` and `episode.logged` rows emitted."""
    seq_before = tmp_memory.head_seq()
    ep = Episode(task="ship FR", approach="audit-revise loop", outcome="success",
                 duration_ms=1800_000, quality_score=0.92)
    path = episode_log(tmp_memory, ep, actor="stephen")
    assert tmp_memory.head_seq() == seq_before + 2
    aux = tmp_memory.read_audit_row(seq_before + 2)
    assert aux["kind"] == "episode.logged"
    assert aux["payload"]["outcome"] == "success"
    assert aux["payload"]["quality_score"] == 0.92


def test_episode_log_defaults_quality_score_to_half_in_aux(tmp_memory: Writer):
    """AC #18 + DEC-181 — absent quality_score surfaces as literal 0.5 in aux row."""
    ep = Episode(task="t", approach="a", outcome="success", duration_ms=10)
    seq_before = tmp_memory.head_seq()
    episode_log(tmp_memory, ep)
    aux = tmp_memory.read_audit_row(seq_before + 2)
    assert aux["payload"]["quality_score"] == 0.5


@pytest.mark.parametrize("outcome,error,ok", [
    ("success",  None,                  True),
    ("partial",  "dispatcher returned 502 once before recovering", True),
    ("failure",  "tokens exhausted",    True),
    ("failure",  None,                  False),    # AC #8
    ("failure",  "",                    False),
    ("kinda",    None,                  False),    # AC #5
])
def test_outcome_enum_and_error_requirement(outcome, error, ok):
    """AC #5 + #8 — closed enum + error required on non-success."""
    if ok:
        Episode(task="t", approach="a", outcome=outcome, duration_ms=1, error=error)
    else:
        with pytest.raises(ValueError):
            Episode(task="t", approach="a", outcome=outcome, duration_ms=1, error=error)


@pytest.mark.parametrize("score,ok", [
    (0.0, True), (1.0, True), (0.5, True), (None, True),
    (-0.0001, False), (1.0001, False),                   # AC #6
])
def test_quality_score_range(score, ok):
    if ok:
        Episode(task="t", approach="a", outcome="success", duration_ms=1, quality_score=score)
    else:
        with pytest.raises(ValueError):
            Episode(task="t", approach="a", outcome="success", duration_ms=1, quality_score=score)


def test_searchable_document_shape():
    """AC #9 — exact format."""
    ep = Episode(task="X", approach="Y", outcome="success", duration_ms=10, notes="N")
    assert ep.searchable_document() == "Task: X\nApproach: Y\nOutcome: success\nNotes: N"


def test_recall_similar_filters_to_episode_kind(seeded_memory):
    """AC #11 — recall-similar returns only episodes, even when facts share similar text."""
    hits = semantic_recall(seeded_memory.path, "ship a feature request",
                           k=10, min_relevance=0.0, memory_kind="episode")
    assert all(h.frontmatter["kind"] == "episode" for h in hits)


def test_recall_similar_combined_score_order(seeded_memory):
    """AC #12 — higher quality_score wins on tie-equivalent relevance."""
    from cyberos.cli.recall import run as recall_run
    out = recall_run(seeded_memory.path, _ns(task="ship", k=5, min_relevance=0.0))
    qs = [m["quality_score"] for m in out["matches"]]
    # Among first two hits, the higher quality_score must come first
    assert qs[0] is None or qs[0] >= (qs[1] if qs[1] is not None else 0.5)


def test_recall_similar_missing_quality_acts_as_half(seeded_memory):
    """AC #13 + DEC-181 — absent quality_score ranks like 0.5."""
    from cyberos.cli.recall import combined_score
    s_absent = combined_score(0.8, None, 1.0)
    s_half   = combined_score(0.8, 0.5,  1.0)
    assert s_absent == pytest.approx(s_half)


def test_recall_similar_top_k_cap(seeded_memory):
    """AC #15 — --k 3 returns ≤ 3 hits."""
    from cyberos.cli.recall import run as recall_run
    out = recall_run(seeded_memory.path, _ns(task="ship", k=3, min_relevance=0.0))
    assert len(out["matches"]) <= 3


def test_recall_similar_fts_fallback(monkeypatch, seeded_memory):
    """AC #16 — sentence-transformers absent → fts5 backend."""
    monkeypatch.setattr("cyberos.core.semantic.available", lambda: False)
    from cyberos.cli.recall import run as recall_run
    out = recall_run(seeded_memory.path, _ns(task="ship", k=3, min_relevance=0.0))
    assert out["backend"] == "fts5"


def test_no_episodes_structured_response(empty_memory):
    """AC #21 — empty store → reason: no_episodes_in_store."""
    from cyberos.cli.recall import run as recall_run
    out = recall_run(empty_memory.path, _ns(task="x", k=3, min_relevance=0.65))
    assert out["matches"] == []
    assert out["reason"] == "no_episodes_in_store"


def test_fixture_corpus_parses():
    """AC #19 — every fixture row constructs a valid Episode."""
    rows = [json.loads(line) for line in Path("tests/fixtures/episode_corpus.jsonl").read_text().splitlines() if line.strip()]
    assert len(rows) >= 40
    for r in rows:
        Episode(**r)
```

```python
# modules/memory/tests/test_episode_schema.py
import jsonschema, json
from pathlib import Path

SCHEMA = json.loads(Path("modules/memory/memory.schema.json").read_text())


def test_episode_kind_in_enum():
    """AC #1 — `episode` is a valid MemoryKind."""
    jsonschema.validate("episode", SCHEMA["$defs"]["MemoryKind"])


def test_legacy_kinds_still_valid():
    """AC #3 — every pre-existing kind continues to validate."""
    for kind in ("decisions", "facts", "people", "projects", "preferences", "drift", "refinements"):
        jsonschema.validate(kind, SCHEMA["$defs"]["MemoryKind"])


def test_episode_frontmatter_required_fields():
    """AC #2 — missing one of {task, approach, outcome, duration_ms} fails validation."""
    base = {"kind": "episode", "task": "t", "approach": "a", "outcome": "success", "duration_ms": 1}
    for missing in ("task", "approach", "outcome", "duration_ms"):
        bad = {k: v for k, v in base.items() if k != missing}
        with pytest.raises(jsonschema.ValidationError):
            jsonschema.validate(bad, SCHEMA["$defs"]["EpisodeFrontmatter"])


def test_walker_rejects_outcome_outside_enum(broken_memory):
    """AC #22 — handcrafted outcome=`kinda` in a written file → doctor non-zero."""
    rc = run_subprocess(["cyberos", "--store", str(broken_memory), "doctor"])
    assert rc != 0
    assert "episode-outcome-closed-enum" in last_stderr()
```

### Fixture corpus (excerpt)

```jsonl
// modules/memory/tests/fixtures/episode_corpus.jsonl  (40+ rows)
{"task": "Ship FR-AUTH-003 RLS enforcement", "approach": "single-commit RLS GUC; per-tenant policies", "outcome": "success", "duration_ms": 1800000, "token_cost": 120000, "quality_score": 0.9, "notes": "8/9 gaps closed"}
{"task": "Migrate Slack workspace to Mattermost", "approach": "Slack import API + decommission gate ≥ 0.95", "outcome": "partial", "duration_ms": 5400000, "token_cost": 88000, "quality_score": 0.7, "notes": "decommission gate at 0.91; held back for round 2", "error": "decommission signal below 0.95 threshold"}
{"task": "Reduce capture daemon p95", "approach": "rate-limit + debounce", "outcome": "failure", "duration_ms": 9_000_000, "token_cost": 200_000, "quality_score": 0.2, "notes": "p95 went UP because debounce ate priority events", "error": "wrong invariant"}
{"task": "Author FR with §1↔§4↔§5 traceability", "approach": "audit-revise loop", "outcome": "success", "duration_ms": 1500000}
```

---

## §6 — Implementation skeleton

API contracts above are the skeleton. Implementation order:

1. Schema (memory.schema.json) — additive enum + EpisodeFrontmatter.
2. Walker invariants (memory.invariants.yaml).
3. `cyberos/core/episode.py` (Episode dataclass + `log()`).
4. `cyberos/core/semantic.py` — add `memory_kind=` kwarg with default None (back-compat).
5. `cyberos/cli/episode.py` + `cyberos/cli/recall.py`.
6. Wire subcommands into `__main__.py`.
7. Tests + fixture corpus.
8. AGENTS.md §2 layout line + CHANGELOG entry.

---

## §7 — Dependencies

- **FR-MEMORY-108 (depends on, shipped or in flight)** — semantic search engine: `cyberos.core.semantic.recall()` is the API we extend. If `sentence-transformers` not installed, FTS5 fallback is the same one FR-MEMORY-108 ships.
- **FR-MEMORY-113 (this FR blocks)** — recency-decay recall ranking: pluggable `recency` function will replace the constant `1.0` placeholder in `combined_score`.
- **FR-MEMORY-114 (this FR blocks)** — write-time importance scoring: optional `meta.importance` field reuses the same per-kind frontmatter contract pattern.
- **FR-MEMORY-115 (this FR blocks)** — `cyberos dream` consumes `episode.logged` aux rows for cross-session pattern detection.
- **FR-MEMORY-120 (this FR enables)** — `cyberos history` surfaces "last time you tried this approach" by walking Episodes for a given task fingerprint.

---

## §8 — Example payloads

### `episode.logged` audit row

```json
{
  "kind": "episode.logged",
  "payload": {
    "path":          "memories/episodes/d4/12/d4127a3b-1f2c3d.md",
    "outcome":       "success",
    "duration_ms":   1800000,
    "token_cost":    120000,
    "quality_score": 0.92,
    "logged_at":     "2026-05-19T14:23:11Z"
  }
}
```

### `cyberos recall-similar` response (--json)

```json
{
  "backend": "semantic",
  "matches": [
    {
      "path": "memories/episodes/d4/12/d4127a3b-1f2c3d.md",
      "task": "Ship FR-AUTH-003 RLS enforcement",
      "approach": "single-commit RLS GUC; per-tenant policies",
      "outcome": "success",
      "quality_score": 0.92,
      "relevance": 0.84,
      "recency": 1.0,
      "combined_score": 0.910,
      "last_seen_at": "2026-05-18T20:55:13Z"
    }
  ],
  "reason": null
}
```

### Memory file on disk (`memories/episodes/d4/12/d4127a3b-1f2c3d.md`)

```markdown
---
{
  "approach": "single-commit RLS GUC; per-tenant policies",
  "duration_ms": 1800000,
  "kind": "episode",
  "notes": "8/9 gaps closed",
  "outcome": "success",
  "quality_score": 0.92,
  "task": "Ship FR-AUTH-003 RLS enforcement",
  "token_cost": 120000
}
---

Task: Ship FR-AUTH-003 RLS enforcement
Approach: single-commit RLS GUC; per-tenant policies
Outcome: success
Notes: 8/9 gaps closed
```

---

## §9 — Open questions

All resolved. Deferred:
- Batch loading from external CI exports — §1 #15; slice 3+ stretch.
- Per-tenant Episode allowlist (suppress recall of episodes from another tenant's memory even when imported) — slice 4+.
- `cyberos episode rescore --path ...` to update `quality_score` after the fact — emits a new `episode.rescored` row; deferred to slice 4 because it changes the recall surface.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| `outcome` outside closed enum at construction | `Episode.__post_init__` ValueError | Episode not created; caller sees exception | Caller picks one of {success, partial, failure} |
| `outcome` outside enum in raw-written file | walker `episode-outcome-closed-enum` (error) | `cyberos doctor` non-zero | Operator fixes file or `cyberos doctor --repair` if available |
| `quality_score` > 1.0 | `Episode.__post_init__` + walker | Same | Same |
| `duration_ms` negative | `Episode.__post_init__` + walker | Same | Same |
| `error` empty on non-success outcome | `Episode.__post_init__` | Episode not created | Caller supplies a concrete failure description |
| `task` empty after trim | `Episode.__post_init__` | Episode not created | Caller supplies non-empty task |
| Missing `sentence-transformers` | `semantic.available()` False | `recall-similar` uses FTS5 backend; response includes `backend: "fts5"` | Operator installs `pip install sentence-transformers` for semantic backend |
| 10K-Episode store p95 > 500 ms (semantic) | bench script asserts at CI | Latency alarm | Operator profiles + tunes batch size / vector quantization |
| Schema migration mid-flight (some files have new kind, some don't) | walker treats absent `kind` field as legacy; new kind validated against new schema | Both shapes coexist | None — migration is non-breaking by §1 #3 |
| Concurrent `episode log` from two processes | existing `.lock` lease serialises | Both writes succeed sequentially; HEAD advances by 4 (2 writes × 2 rows each) | None — by design |
| `episode.logged` aux row written but `put` row failed | impossible: writer emits both inside same transaction batch | If chain integrity broken, doctor catches via existing chain-verify | `cyberos consolidate` (existing recovery path) |
| Caller passes both `quality_score=None` and `quality_score=0.5` (ambiguous intent) | only one kwarg; latest wins | No ambiguity in API | None |
| Schema enum extended in newer version, older walker can't parse | walker's enum is loaded from `memory.schema.json` at boot | Newer schema rejected by older walker → `cyberos doctor` fails with structured error | Operator upgrades the cyberos package |
| Token cost overflow (int > 2^63) | jsonschema integer | rejected by schema validate | Caller sanity-checks; impossible in practice |
| `notes` contains markdown that confuses the FTS tokenizer | FTS5 trigram tokenizer handles arbitrary UTF-8 | Search ranks may shift but no error | None — by design |
| `task` and `approach` identical (degenerate Episode) | Episode constructs successfully | `recall-similar` ranks normally; doesn't filter dupes | If undesired, future dedup pass (FR-MEMORY-116) merges |
| Episode written by an actor without ACL on `memories/episodes/` (FR-MEMORY-117 era) | writer rejects per `STORE.yaml` check | Write fails with `acl_denied` audit row | Operator grants write scope or uses a different store |
| Caller misuses `recall-similar` for non-episode queries (e.g. recall recently-noted facts) | strict filter to `kind: episode` | Returns no matches; reason: `no_episodes_above_min_relevance` | Operator switches to `cyberos search` / `cyberos recall` |
| Fixture corpus drift (new schema field added, fixture not updated) | `test_fixture_corpus_parses` fails | CI blocked | Author updates fixture |
| `combined_score` overflow / NaN | bounded inputs; deterministic arithmetic | Not possible by construction | None |

---

## §11 — Implementation notes

- **Why frontmatter is JSON not YAML inside Episode files (§3 example).** The walker's `frontmatter.py` already supports both YAML and JSON frontmatter. JSON is deterministic (no `null` vs absent ambiguity), and the existing writer's canonicalisation step produces JSON anyway. New Episodes emit JSON; legacy YAML Episodes (if anyone hand-rolls them) continue to parse.
- **Why `episode.logged` is a separate aux audit kind, not a frontmatter field.** A frontmatter field is part of the immutable body. The aux audit row is an indexable projection — FR-MEMORY-115 scans rows of kind `episode.logged` directly without re-parsing markdown. The same pattern is used elsewhere in CyberOS (`memory.imported`, `consolidation.published`).
- **`searchable_document()` excludes the `error` field deliberately.** An error trace can contain stack-trace noise that pollutes the embedding. The error is preserved in the frontmatter for `cyberos history` and dream-pipeline introspection, but it's kept out of the FTS / semantic surface.
- **Recall path is over-fetch + filter, not query-level kind filter at the engine.** Reason: SQLite FTS5 doesn't natively filter on JSON metadata; vector store does but each backend has its own filter syntax. Over-fetching 4× the requested k and filtering in Python is portable + fast enough (vector retrieval is O(log N); filter is O(k·4) post-step).
- **Fixture corpus is 40+ rows because that's the threshold below which CI flakes.** Adding 10 more rows per quarter as the team observes new failure modes is the maintenance cadence (same as FR-MEMORY-111's PII corpus).
- **`memory_kind` is a string, not an enum, in the Python API.** Enum would be cleaner but it forces every caller to import the enum; a string is easier for the CLI to pass through. The schema is the source of truth; runtime validation is on the schema side.
- **`tmp_memory` and `seeded_memory` pytest fixtures** — defined in `modules/memory/tests/conftest.py`. `tmp_memory` is a fresh empty store; `seeded_memory` has 5 episodes + 5 facts pre-loaded for recall tests. Add `empty_memory` and `broken_memory` (with a hand-written invalid Episode) in this FR.
- **Why we don't auto-set `last_seen_at` on the frontmatter.** The audit chain already records when the put happened; duplicating it in the body would be drift-prone. FR-MEMORY-120 (`cyberos history`) projects `last_seen_at` from the latest audit row touching that path.

---

*End of FR-MEMORY-112.*
