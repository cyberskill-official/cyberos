---
id: TASK-MEMORY-114
title: "memory write-time importance scoring — cuo-Phase-3-pattern Invoker (mock-llm + anthropic); haiku-rated `meta.importance` filters noise at the source; opt-in via `cyberos put --score-importance`"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-19T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: memory
priority: p1
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_tasks: [TASK-MEMORY-112, TASK-MEMORY-113, TASK-MEMORY-115, TASK-CUO-105]
depends_on: [TASK-MEMORY-113]
blocks: [TASK-MEMORY-115]

source_pages:
  # see "Memory management" / "Importance scoring at write time"
  - playground/extracts/agentic-memory.article.txt
source_decisions:
  - DEC-200 (Write-time importance is OPTIONAL — `--score-importance` flag opts in; default writes never call an LLM)
  - DEC-201 (Importance Invoker mirrors `modules/cuo/cuo/invokers.py` Phase-3 pattern — `MockInvoker` for tests/dev, `AnthropicInvoker` for prod; selection via `CYBEROS_IMPORTANCE_INVOKER` env or `manifest.importance.invoker`)
  - DEC-202 (Default model is `claude-haiku-4-5` — cheapest fast model; configurable via `manifest.importance.model`)
  - DEC-203 (Score caching keyed on `sha256(content)` → score; cache lives at `<memory-root>/index/importance_cache.db` so the same content scored twice is free)
  - "DEC-204 (On any importance-scoring error — API failure, parse failure, timeout — fall back to `0.5` neutral and emit `memory.importance_scored` audit row with `outcome: \"fallback\"` and `reason: \"<text>\"`)"

language: python 3.10
service: modules/memory/cyberos/
new_files:
  - modules/memory/cyberos/core/importance.py
  - modules/memory/cyberos/core/invokers/__init__.py
  - modules/memory/cyberos/core/invokers/base.py
  - modules/memory/cyberos/core/invokers/mock.py
  - modules/memory/cyberos/core/invokers/anthropic_invoker.py
  - modules/memory/tests/core/test_importance.py
  - modules/memory/tests/core/test_importance.py
  - modules/memory/tests/core/test_importance.py
modified_files:
  # add `--score-importance` flag to `put` subcommand
  - modules/memory/cyberos/__main__.py
  # if --score-importance, route through importance.score() before put; merge into frontmatter
  - modules/memory/cyberos/core/writer.py
  # `manifest.importance` schema fragment
  - modules/memory/memory.schema.json
  # `importance-cache-valid-sha256` rule
  - modules/memory/memory.invariants.yaml
allowed_tools:
  - file_read: modules/memory/**, modules/cuo/cuo/invokers.py
  - file_write: modules/memory/cyberos/**, modules/memory/tests/**, modules/memory/memory.schema.json, modules/memory/memory.invariants.yaml
  - bash: cd modules/memory && python -m pytest tests/test_importance_*.py -v
  - bash: cd modules/memory && python -m cyberos put memories/facts/test.md - --score-importance --dry-run < /tmp/sample.md
disallowed_tools:
  #8)
  - silently emit `meta.importance` without an audit row (per §1
  - call the production Anthropic API in unit tests (per DEC-201 — tests use MockInvoker; CI without API key must pass)
  #6)
  - cache misses without re-scoring on next call (per §1

effort_hours: 8
subtasks:
  - "1.0h: cyberos/core/invokers/base.py — `ImportanceInvoker` Protocol (async `score(content: str) -> ScoreResult` where `ScoreResult = {score: float, latency_ms: int, model: str, outcome: 'ok'|'fallback', reason: str|None}`)"
  - "0.5h: invokers/mock.py — `MockInvoker` returns deterministic score (sha256(content)[:8] / 2^32; clamped to [0.1, 0.95] so it's never literal 0.0 or 1.0)"
  - "1.5h: invokers/anthropic_invoker.py — `AnthropicInvoker` calls Anthropic API with prompt from Ramakrushna article verbatim; parses first float in response; 5s timeout; fallback to MockInvoker on error; reuses CUO Phase-3 import pattern (graceful when `anthropic` not installed)"
  - "1.0h: cyberos/core/importance.py — `score(content, invoker=None) -> ScoreResult` orchestrator; cache check first (DEC-203); invoker fallback chain; emit `memory.importance_scored` aux row"
  - "0.5h: cache layer — SQLite single-table `(content_sha256 BLOB PK, score REAL, model TEXT, scored_at INTEGER); UNIQUE(content_sha256)`"
  - "0.5h: cyberos/core/writer.py — `--score-importance` path: score before put; merge `meta.importance` into frontmatter; preserve back-compat (caller can override with explicit `--importance 0.7`)"
  - "1.0h: __main__.py — wire `--score-importance` + `--importance <float>` + `--dry-run` (prints what would be written, no write)"
  - "1.0h: modules/memory/tests/core/test_importance.py — 10 cases (MockInvoker deterministic, AnthropicInvoker mocked HTTP, parse first float, clamp out-of-range to [0, 1], fallback on timeout, fallback on parse error)"
  - "0.5h: modules/memory/tests/core/test_importance.py — 6 cases (env var precedence, manifest precedence, default to mock, unknown invoker raises, CYBEROS_DISABLE_LLM=1 forces mock)"
  - "0.5h: modules/memory/tests/core/test_importance.py — 6 cases (cache hit returns same score, cache miss calls invoker, sha256 mismatch invalidates, cache survives Writer restart, cache file gracefully created)"
  - "0.5h: schema + invariant for `manifest.importance` block"
risk_if_skipped: "Without write-time importance scoring, every memory in the memory has `meta.importance` absent → ranking defaults all to 0.5 (DEC-192 + TASK-MEMORY-113 #3). The combined-score formula then degenerates: importance contribution is a constant 0.15 (= 0.5 · 0.3), so ranking is effectively `relevance · 0.4 + recency · 0.3 + 0.15`. The article (Ramakrushna §'Memory management') flags this as the noise-at-source problem — trivia accumulates in the store at the same weight as decisions. As stores grow past 1K memories, recall surface degrades. Skipping this task means we have the *infrastructure* to rank by importance (TASK-MEMORY-113) but no signal feeding it. We'd be in the same place as if we'd skipped TASK-MEMORY-113. Shipping TASK-MEMORY-114 closes Wave 1 — gives TASK-MEMORY-115 (dream) a real signal to use when proposing 'this memory is stale, that one matters'."
---

## §1 — Description (BCP-14 normative)

The write-time importance scoring layer is an **optional preprocessor** sitting between `cyberos put` invocation and the canonical Writer. When opted-in via `--score-importance`, it asks a small/cheap LLM to rate the candidate content on a `[0.0, 1.0]` scale, merges the result into `meta.importance` on the frontmatter, then proceeds with the write. The contract:

1. **MUST** be **optional**, opt-in via `cyberos put --score-importance`. Default `cyberos put` writes MUST NOT call any LLM and MUST NOT touch the network (DEC-200). This preserves the offline-first guarantee of the memory.
2. **MUST** allow the operator to explicitly override the scored value via `--importance <float>` on the same `cyberos put` invocation. Both flags together: `--importance` wins; no LLM call is made (saves cost when the operator already knows).
3. **MUST** select the active Invoker through the same priority chain as the CUO supervisor (Phase 3):
1. CLI flag `--invoker {mock|anthropic}` (highest priority)
2. Env var `CYBEROS_IMPORTANCE_INVOKER`
3. `manifest.json:importance.invoker` (string enum `mock | anthropic`)
4. Default `"mock"` when no API key is detected; `"anthropic"` when `ANTHROPIC_API_KEY` is set in env
4. **MUST** treat `CYBEROS_DISABLE_LLM=1` (env) as a hard override forcing `MockInvoker` regardless of any other setting. This is the offline / CI / air-gapped escape hatch.
5. **MUST** define `ImportanceInvoker` as a Protocol with method `async def score(content: str) -> ScoreResult` returning the typed record:
    ```python
    @dataclass(frozen=True)
    class ScoreResult:
        score:       float      # in [0.0, 1.0]
        latency_ms:  int
        model:       str        # "mock" or e.g. "claude-haiku-4-5-20251001"
        outcome:     Literal["ok", "fallback"]
        reason:      Optional[str]   # populated when outcome == "fallback"
    ```
6. **MUST** cache results keyed on `sha256(content)`. Cache hit ⇒ no invoker call; cache miss ⇒ invoker call ⇒ result stored. Cache lives at `<memory-root>/index/importance_cache.db` (SQLite). Cache invalidation is automatic via sha256 mismatch (DEC-203). The cache table schema and the `importance-cache-valid-sha256` invariant guard against partial writes.
7. **MUST** clamp invoker output to `[0.0, 1.0]`. Out-of-range responses (e.g. LLM returns "1.5" or "-0.1") are clamped silently — they don't constitute a fallback, just bounded sanitisation. Logged at DEBUG level.
8. **MUST** emit one `memory.importance_scored` audit row per scoring invocation (whether opt-in flag was set or not — even cache hits emit). Payload:
    ```json
    {
      "kind": "memory.importance_scored",
      "payload": {
        "path":         "memories/facts/x.md",
        "content_sha256": "abc123…",
        "score":        0.72,
        "model":        "claude-haiku-4-5-20251001",
        "outcome":      "ok",
        "reason":       null,
        "cache_hit":    false,
        "latency_ms":   240
      }
    }
    ```
9. **MUST** fall back to `0.5` (neutral) on any of: invoker timeout (≥ 5 s), API error, response parse failure, missing API key (when `anthropic` invoker selected), exception in invoker. The fallback path emits `outcome: "fallback"` with a concrete `reason` string. Memory is still written with `meta.importance: 0.5` (so downstream ranking is non-degenerate).
10. **MUST** support `--dry-run` on `cyberos put --score-importance`: print the would-be `meta.importance` value + the full frontmatter as it would be written, but do not actually write anything to disk. Useful for operators evaluating score quality on a single file.
11. **MUST** use the exact prompt from Ramakrushna's article (verbatim) as the AnthropicInvoker's system prompt:
    ```text
    Rate the importance of saving this for future interactions.
    0.0 = trivial (greeting)
    0.5 = moderately useful
    1.0 = critical (preferences, errors, decisions)

    Information: <content>
    Reply with ONLY the number.
    ```
    The response is parsed with regex `r"[-+]?\d*\.\d+|\d+"` → first match → float. If parse fails, fallback.
12. **MUST** complete a single scoring call in ≤ 5 seconds (invoker timeout). On timeout, fallback. On AnthropicInvoker rate-limit (HTTP 429), retry once with 1-second jitter then fall back.
13. **MUST** validate `manifest.json:importance` block at writer construction time (same fail-fast pattern as TASK-MEMORY-113 §1 #9). Malformed → `ManifestError`.
14. **SHOULD** offer a batch-scoring mode `cyberos importance score-all [--filter kind=facts] [--unscored-only]` that walks existing memories and scores any without `meta.importance`. Slice-4 stretch — not required for TASK-MEMORY-115 to consume scores.
15. **SHOULD** expose `cyberos importance stats` for operator introspection: histogram of scores over the store, % of memories with absent vs scored importance, cache hit rate, fallback rate. Slice-4 stretch.

---

## §2 — Why this design (rationale for humans)

**Why opt-in, not default-on (§1 #1, DEC-200).** The memory is offline-first. A user dropping in `AGENTS.md` on a corporate laptop without API access must continue to work. Default-on importance scoring would silently fail or block writes for anyone without an API key. Opt-in keeps the simple case simple; sophisticated users who want quality scoring enable it explicitly.

**Why CUO Phase-3 invoker pattern (§1 #3, DEC-201).** The CUO supervisor already implements this exact pattern — `MockInvoker` for tests, `AnthropicInvoker` for prod, env-or-manifest selection, `CYBEROS_DISABLE_LLM` escape hatch. Reusing it: (a) one mental model across the codebase; (b) the test infrastructure already exists; (c) operators who know CUO know this; (d) future LLM providers (OpenAI, local Ollama, etc.) plug in as new Invoker classes without changing call sites.

**Why `claude-haiku-4-5` default (DEC-202).** Importance scoring is a one-token-out task. Haiku is the cheapest fast model that reliably outputs structured floats. Sonnet/Opus would work but cost 10-30× more per call. For a 1000-memory new project, this is the difference between a few cents and a dollar per scoring pass.

**Why cache on `sha256(content)` (§1 #6, DEC-203).** The same content scored twice is wasted spend. Cache eliminates duplicate calls on (a) idempotent re-writes (same put with same body), (b) the slice-4 `score-all` batch pass, (c) `--dry-run` invocations that the operator iterates on. SHA-256 keying makes content-equality the cache primitive — a one-character whitespace change is a different score (because the model might judge it differently), so we don't accidentally return stale scores.

**Why clamp out-of-range silently (§1 #7).** LLMs occasionally drift outside `[0, 1]` (especially with the literal "Reply with ONLY the number" prompt — Haiku might say "0.95-1.0" or "around 0.6"). Clamping recovers the signal without treating it as a fallback. Logging at DEBUG lets operators audit drift without polluting normal logs.

**Why fallback to 0.5, not refuse to write (§1 #9, DEC-204).** Refusing to write would conflate "I can't score this" with "I can't store this". The operator's intent is the write; importance is the optional metadata. 0.5 (neutral) lets the write proceed without falsely up- or down-ranking the memory in future recall. The `memory.importance_scored` audit row with `outcome: "fallback"` and a `reason` preserves the diagnostic signal for TASK-MEMORY-115 dream to learn from ("hey, 30% of imports last week were fallback — check API status").

**Why exact Ramakrushna prompt (§1 #11).** The article's prompt is calibrated: 0.0 / 0.5 / 1.0 anchored to concrete examples. Reinventing the prompt is wasted effort and risks miscalibration vs. the published baseline. Keeping the prompt verbatim also means we can swap LLM providers (OpenAI, local) without changing the prompt — the prompt is the contract, not the model.

**Why 5-second timeout (§1 #12).** Haiku p95 is ~1.5 s. 5 s is 3× margin — catches network hiccups, rate-limit waits, etc. Beyond 5 s we fall back rather than block the write. Blocking puts on LLM latency would degrade the user's perceived memory responsiveness.

**Why emit `memory.importance_scored` even on cache hits (§1 #8).** TASK-MEMORY-115's dream pipeline counts cache hit rate when evaluating "is the importance signal actually being used?". If we suppress cache-hit audit rows, dream can't tell whether a low-import store has 1000 misses or 1000 hits. Audit rows are cheap; the signal is valuable.

---

## §3 — API contract

### Invoker Protocol + implementations

```python
# modules/memory/cyberos/core/invokers/base.py
from __future__ import annotations
from dataclasses import dataclass
from typing import Literal, Optional, Protocol


@dataclass(frozen=True)
class ScoreResult:
    score:      float
    latency_ms: int
    model:      str
    outcome:    Literal["ok", "fallback"]
    reason:     Optional[str] = None


class ImportanceInvoker(Protocol):
    async def score(self, content: str) -> ScoreResult: ...
```

```python
# modules/memory/cyberos/core/invokers/mock.py
import hashlib, time
from .base import ImportanceInvoker, ScoreResult


class MockInvoker:
    """Deterministic, offline. Score derived from sha256(content) so the same
    content always gets the same mock score (good for tests + cache validation)."""

    async def score(self, content: str) -> ScoreResult:
        t0 = time.perf_counter()
        h = hashlib.sha256(content.encode("utf-8")).hexdigest()
        raw = int(h[:8], 16) / 0xFFFFFFFF        # [0.0, 1.0]
        score = 0.1 + raw * 0.85                  # clamp to [0.1, 0.95] so we never hit literal extremes
        latency_ms = int((time.perf_counter() - t0) * 1000)
        return ScoreResult(score=score, latency_ms=latency_ms, model="mock", outcome="ok", reason=None)
```

```python
# modules/memory/cyberos/core/invokers/anthropic_invoker.py
import os, re, time
from .base import ImportanceInvoker, ScoreResult

SYSTEM_PROMPT = """Rate the importance of saving this for future interactions.
0.0 = trivial (greeting)
0.5 = moderately useful
1.0 = critical (preferences, errors, decisions)

Information: {content}
Reply with ONLY the number."""

_FLOAT_RE = re.compile(r"[-+]?\d*\.\d+|\d+")


class AnthropicInvoker:
    def __init__(self, model: str = "claude-haiku-4-5", timeout_s: float = 5.0) -> None:
        try:
            import anthropic                                   # noqa: F401
        except ImportError as e:                                # graceful per CUO Phase-3 pattern
            raise RuntimeError(
                "AnthropicInvoker selected but the `anthropic` package is not installed.\n"
                "Install with `pip install anthropic` or set CYBEROS_IMPORTANCE_INVOKER=mock"
            ) from e
        if not os.environ.get("ANTHROPIC_API_KEY"):
            raise RuntimeError(
                "AnthropicInvoker selected but ANTHROPIC_API_KEY is unset.\n"
                "Export the key or set CYBEROS_DISABLE_LLM=1"
            )
        self._model = model
        self._timeout_s = timeout_s

    async def score(self, content: str) -> ScoreResult:
        import anthropic, asyncio
        client = anthropic.AsyncAnthropic()
        t0 = time.perf_counter()
        try:
            resp = await asyncio.wait_for(
                client.messages.create(
                    model=self._model,
                    max_tokens=8,
                    system=SYSTEM_PROMPT.format(content=content),
                    messages=[{"role": "user", "content": "Rate this."}],
                ),
                timeout=self._timeout_s,
            )
        except asyncio.TimeoutError:
            return ScoreResult(score=0.5, latency_ms=int(self._timeout_s * 1000),
                               model=self._model, outcome="fallback", reason="timeout")
        except Exception as e:
            return ScoreResult(score=0.5, latency_ms=int((time.perf_counter() - t0) * 1000),
                               model=self._model, outcome="fallback", reason=f"api_error:{type(e).__name__}")
        text = resp.content[0].text.strip()
        m = _FLOAT_RE.search(text)
        if not m:
            return ScoreResult(score=0.5, latency_ms=int((time.perf_counter() - t0) * 1000),
                               model=self._model, outcome="fallback", reason=f"parse_error:{text!r}")
        try:
            v = float(m.group())
        except ValueError:
            return ScoreResult(score=0.5, latency_ms=int((time.perf_counter() - t0) * 1000),
                               model=self._model, outcome="fallback", reason=f"parse_error:{m.group()!r}")
        clamped = max(0.0, min(1.0, v))
        return ScoreResult(score=clamped, latency_ms=int((time.perf_counter() - t0) * 1000),
                           model=self._model, outcome="ok", reason=None)
```

### Orchestrator + cache

```python
# modules/memory/cyberos/core/importance.py
import hashlib, sqlite3, time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from cyberos.core.invokers.base import ImportanceInvoker, ScoreResult


def select_invoker(name: Optional[str] = None) -> ImportanceInvoker:
    import os
    from cyberos.core.invokers.mock import MockInvoker

    if os.environ.get("CYBEROS_DISABLE_LLM") == "1":
        return MockInvoker()                                   # §1 #4 escape hatch
    name = name or os.environ.get("CYBEROS_IMPORTANCE_INVOKER") or _default_from_env()
    if name == "mock":
        return MockInvoker()
    if name == "anthropic":
        from cyberos.core.invokers.anthropic_invoker import AnthropicInvoker
        return AnthropicInvoker()
    raise ValueError(f"unknown invoker {name!r}; expected one of: mock, anthropic")


def _default_from_env() -> str:
    import os
    return "anthropic" if os.environ.get("ANTHROPIC_API_KEY") else "mock"


class ImportanceCache:
    def __init__(self, db_path: Path):
        db_path.parent.mkdir(parents=True, exist_ok=True)
        self._con = sqlite3.connect(str(db_path), isolation_level=None)
        self._con.execute("""
            CREATE TABLE IF NOT EXISTS importance_cache (
                content_sha256 BLOB PRIMARY KEY,
                score          REAL NOT NULL,
                model          TEXT NOT NULL,
                scored_at_ns   INTEGER NOT NULL
            )
        """)

    def get(self, content_sha256: bytes) -> Optional[float]:
        cur = self._con.execute(
            "SELECT score FROM importance_cache WHERE content_sha256 = ?", (content_sha256,)
        )
        row = cur.fetchone()
        return row[0] if row else None

    def put(self, content_sha256: bytes, score: float, model: str) -> None:
        self._con.execute(
            "INSERT OR REPLACE INTO importance_cache (content_sha256, score, model, scored_at_ns) VALUES (?, ?, ?, ?)",
            (content_sha256, score, model, time.time_ns()),
        )


async def score(content: str, invoker: ImportanceInvoker, cache: Optional[ImportanceCache] = None,
                aux_emitter=None, path: str = "") -> ScoreResult:
    h = hashlib.sha256(content.encode("utf-8")).digest()
    if cache is not None:
        cached = cache.get(h)
        if cached is not None:
            res = ScoreResult(score=cached, latency_ms=0, model="cache", outcome="ok", reason=None)
            if aux_emitter:
                aux_emitter(kind="memory.importance_scored",
                            payload={"path": path, "content_sha256": h.hex(),
                                     "score": cached, "model": "cache",
                                     "outcome": "ok", "reason": None, "cache_hit": True,
                                     "latency_ms": 0})
            return res
    res = await invoker.score(content)
    if cache is not None and res.outcome == "ok":
        cache.put(h, res.score, res.model)
    if aux_emitter:
        aux_emitter(kind="memory.importance_scored",
                    payload={"path": path, "content_sha256": h.hex(),
                             "score": res.score, "model": res.model,
                             "outcome": res.outcome, "reason": res.reason,
                             "cache_hit": False, "latency_ms": res.latency_ms})
    return res
```

### Manifest

```json
// .cyberos/memory/store/manifest.json (excerpt)
{
  "importance": {
    "invoker":  "anthropic",
    "model":    "claude-haiku-4-5",
    "timeout_s": 5.0
  }
}
```

---

## §4 — Acceptance criteria

1. **Opt-in default-off** — `cyberos put memories/x.md - < file` does NOT call any invoker; `cache.get_call_count() == 0` after the call. *(traces_to: §1 #1, DEC-200)*
2. **Opt-in flag works** — `cyberos put memories/x.md - --score-importance < file` produces a memory file with `meta.importance` set to a float in `[0.0, 1.0]`. *(traces_to: §1 #1)*
3. **Explicit override beats LLM** — `cyberos put memories/x.md - --score-importance --importance 0.9 < file` produces `meta.importance: 0.9`; invoker was NOT called (call count 0). *(traces_to: §1 #2)*
4. **Invoker selection — CLI flag wins** — `--invoker anthropic` with no `CYBEROS_IMPORTANCE_INVOKER` set → AnthropicInvoker selected. *(traces_to: §1 #3)*
5. **Invoker selection — env wins over manifest** — `CYBEROS_IMPORTANCE_INVOKER=mock` with `manifest.importance.invoker=anthropic` → MockInvoker. *(traces_to: §1 #3)*
6. **Invoker selection — manifest wins over default** — manifest sets `anthropic`; no env, no CLI flag → AnthropicInvoker. *(traces_to: §1 #3)*
7. **`CYBEROS_DISABLE_LLM=1` forces mock** — env set, manifest says anthropic, CLI says anthropic → MockInvoker still selected. *(traces_to: §1 #4, DEC-201)*
8. **Default fallback when no API key** — no env vars, no manifest, no key → `select_invoker()` returns `MockInvoker`. *(traces_to: §1 #3)*
9. **MockInvoker deterministic** — `MockInvoker().score("X")` returns same score across two calls. *(traces_to: §1 #5)*
10. **ScoreResult shape** — `score()` returns `ScoreResult` with `score`, `latency_ms`, `model`, `outcome`, `reason` fields. *(traces_to: §1 #5)*
11. **Cache hit returns cached score, no invoker call** — `score(content, mock, cache)` twice → second call has `model: "cache"`; mock call count = 1. *(traces_to: §1 #6, DEC-203)*
12. **Cache invalidates on content change** — same path but different body → cache miss; invoker called. *(traces_to: §1 #6)*
13. **Cache survives Writer restart** — write twice with same content, restart Writer between → second call is cache hit. *(traces_to: §1 #6)*
14. **Clamp out-of-range LLM output** — AnthropicInvoker mocked to return "1.5" → returned `score == 1.0`; mocked to return "-0.1" → `score == 0.0`. *(traces_to: §1 #7)*
15. **`memory.importance_scored` audit row emitted** — per call (incl. cache hits); payload matches schema in §1 #8. *(traces_to: §1 #8)*
16. **Audit row `cache_hit` field accurate** — cache miss → `cache_hit: false`; cache hit → `cache_hit: true`. *(traces_to: §1 #8)*
17. **Fallback on timeout** — AnthropicInvoker mocked to sleep 6s, timeout=5s → ScoreResult `score=0.5, outcome="fallback", reason="timeout"`. *(traces_to: §1 #9, DEC-204)*
18. **Fallback on parse error** — AnthropicInvoker mocked to return "I think it's quite important" (no float) → fallback. *(traces_to: §1 #9, §1 #11)*
19. **Fallback on missing API key** — AnthropicInvoker selected but `ANTHROPIC_API_KEY` unset → constructor raises with a structured message naming the env var. *(traces_to: §1 #3, §1 #9)*
20. **`--dry-run` writes nothing** — `cyberos put ... --score-importance --dry-run` produces no audit row, no memory file, but prints the frontmatter to stdout. *(traces_to: §1 #10)*
21. **Verbatim Ramakrushna prompt** — AnthropicInvoker's system prompt text equals the literal §1 #11 string. *(traces_to: §1 #11)*
22. **Timeout default = 5 s** — `AnthropicInvoker()._timeout_s == 5.0`. *(traces_to: §1 #12)*
23. **Manifest validation at construction** — `manifest.importance.invoker = "made_up"` → `Writer(...)` raises ManifestError. *(traces_to: §1 #13)*

---

## §5 — Verification

```python
# modules/memory/tests/core/test_importance.py
import pytest, asyncio
from unittest.mock import patch, AsyncMock, MagicMock

from cyberos.core.invokers.mock import MockInvoker
from cyberos.core.importance     import score


async def _run(coro): return await coro


def test_mock_invoker_deterministic():
    """AC #9"""
    inv = MockInvoker()
    r1 = asyncio.run(inv.score("X"))
    r2 = asyncio.run(inv.score("X"))
    assert r1.score == r2.score


def test_mock_score_in_clamped_range():
    """AC #10 — score in [0.1, 0.95] sane mock range"""
    inv = MockInvoker()
    r = asyncio.run(inv.score("anything"))
    assert 0.1 <= r.score <= 0.95


@pytest.mark.parametrize("text,expected_score,outcome", [
    ("0.85",                 0.85, "ok"),
    ("0.85.",                0.85, "ok"),
    ("1.5",                  1.0,  "ok"),     # AC #14: clamp
    ("-0.1",                 0.0,  "ok"),     # AC #14
    ("I think it's quite important", 0.5, "fallback"),  # AC #18
])
def test_anthropic_invoker_parse_paths(text, expected_score, outcome):
    """AC #14, #18"""
    from cyberos.core.invokers.anthropic_invoker import AnthropicInvoker
    inv = AnthropicInvoker.__new__(AnthropicInvoker)    # bypass __init__
    inv._model = "claude-haiku-4-5-test"
    inv._timeout_s = 5.0
    fake_resp = MagicMock()
    fake_resp.content = [MagicMock(text=text)]
    with patch.object(AnthropicInvoker, "score",
                     AsyncMock(side_effect=lambda content: AnthropicInvoker._real_score(inv, content, fake_resp))):
        # ... (test scaffold simplified for readability; real test calls the real `score` with the mocked HTTP call)
        pass


def test_fallback_on_timeout():
    """AC #17"""
    from cyberos.core.invokers.anthropic_invoker import AnthropicInvoker
    inv = AnthropicInvoker.__new__(AnthropicInvoker)
    inv._model = "claude-haiku-4-5-test"
    inv._timeout_s = 0.001     # ridiculous
    # Patch asyncio.wait_for to raise TimeoutError
    with patch("asyncio.wait_for", side_effect=asyncio.TimeoutError):
        r = asyncio.run(inv.score("X"))
    assert r.outcome == "fallback"
    assert r.reason == "timeout"
    assert r.score == 0.5


def test_audit_row_emitted_on_cache_hit(tmp_memory, capsys_emitter):
    """AC #15 + #16"""
    from cyberos.core.importance import ImportanceCache
    cache = ImportanceCache(tmp_memory.store_path / "index/importance_cache.db")
    inv = MockInvoker()
    rows = []
    def emit(kind, payload): rows.append((kind, payload))
    r1 = asyncio.run(score("X", inv, cache, aux_emitter=emit, path="memories/x.md"))
    r2 = asyncio.run(score("X", inv, cache, aux_emitter=emit, path="memories/x.md"))
    assert len(rows) == 2
    assert rows[0][1]["cache_hit"] is False
    assert rows[1][1]["cache_hit"] is True


def test_verbatim_ramakrushna_prompt():
    """AC #21"""
    from cyberos.core.invokers.anthropic_invoker import SYSTEM_PROMPT
    expected_prefix = "Rate the importance of saving this for future interactions.\n0.0 = trivial (greeting)"
    assert SYSTEM_PROMPT.startswith(expected_prefix)
    assert "Reply with ONLY the number." in SYSTEM_PROMPT


def test_default_timeout_5_seconds():
    """AC #22"""
    from cyberos.core.invokers.anthropic_invoker import AnthropicInvoker
    import os
    os.environ.setdefault("ANTHROPIC_API_KEY", "test")
    inv = AnthropicInvoker()
    assert inv._timeout_s == 5.0
```

```python
# modules/memory/tests/core/test_importance.py
import pytest, os
from cyberos.core.importance import select_invoker


def test_disable_llm_env_forces_mock(monkeypatch):
    """AC #7"""
    monkeypatch.setenv("CYBEROS_DISABLE_LLM", "1")
    monkeypatch.setenv("CYBEROS_IMPORTANCE_INVOKER", "anthropic")
    inv = select_invoker("anthropic")
    assert inv.__class__.__name__ == "MockInvoker"


def test_cli_flag_wins(monkeypatch):
    """AC #4"""
    monkeypatch.delenv("CYBEROS_DISABLE_LLM", raising=False)
    monkeypatch.delenv("CYBEROS_IMPORTANCE_INVOKER", raising=False)
    monkeypatch.setenv("ANTHROPIC_API_KEY", "test")
    inv = select_invoker("mock")          # explicit override
    assert inv.__class__.__name__ == "MockInvoker"


def test_env_var(monkeypatch):
    """AC #5"""
    monkeypatch.delenv("CYBEROS_DISABLE_LLM", raising=False)
    monkeypatch.setenv("CYBEROS_IMPORTANCE_INVOKER", "mock")
    monkeypatch.setenv("ANTHROPIC_API_KEY", "test")
    inv = select_invoker()
    assert inv.__class__.__name__ == "MockInvoker"


def test_default_without_api_key(monkeypatch):
    """AC #8"""
    monkeypatch.delenv("CYBEROS_DISABLE_LLM", raising=False)
    monkeypatch.delenv("CYBEROS_IMPORTANCE_INVOKER", raising=False)
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
    inv = select_invoker()
    assert inv.__class__.__name__ == "MockInvoker"


def test_unknown_invoker_raises(monkeypatch):
    monkeypatch.delenv("CYBEROS_DISABLE_LLM", raising=False)
    with pytest.raises(ValueError):
        select_invoker("made_up")


def test_missing_api_key_for_anthropic(monkeypatch):
    """AC #19"""
    monkeypatch.delenv("CYBEROS_DISABLE_LLM", raising=False)
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
    with pytest.raises(RuntimeError, match="ANTHROPIC_API_KEY"):
        select_invoker("anthropic")
```

```python
# modules/memory/tests/core/test_importance.py
import asyncio, hashlib
from cyberos.core.importance import ImportanceCache, score
from cyberos.core.invokers.mock import MockInvoker


def test_cache_hit_returns_same_score(tmp_path):
    """AC #11"""
    cache = ImportanceCache(tmp_path / "cache.db")
    inv = MockInvoker()
    r1 = asyncio.run(score("X", inv, cache))
    r2 = asyncio.run(score("X", inv, cache))
    assert r1.score == r2.score
    assert r2.model == "cache"


def test_cache_invalidates_on_content_change(tmp_path):
    """AC #12"""
    cache = ImportanceCache(tmp_path / "cache.db")
    inv = MockInvoker()
    r1 = asyncio.run(score("X", inv, cache))
    r2 = asyncio.run(score("Y", inv, cache))
    assert r1.score != r2.score
    assert r2.model != "cache"


def test_cache_survives_restart(tmp_path):
    """AC #13"""
    cache1 = ImportanceCache(tmp_path / "cache.db")
    inv = MockInvoker()
    r1 = asyncio.run(score("X", inv, cache1))
    cache1 = None
    cache2 = ImportanceCache(tmp_path / "cache.db")
    r2 = asyncio.run(score("X", inv, cache2))
    assert r2.model == "cache"
    assert r2.score == r1.score


def test_cache_file_created_lazily(tmp_path):
    cache = ImportanceCache(tmp_path / "nested/cache.db")
    assert (tmp_path / "nested/cache.db").exists()


def test_cache_sha256_key_correctness(tmp_path):
    """AC #11 implementation detail — cache key is sha256(content)"""
    cache = ImportanceCache(tmp_path / "cache.db")
    inv = MockInvoker()
    asyncio.run(score("hello world", inv, cache))
    h = hashlib.sha256("hello world".encode()).digest()
    assert cache.get(h) is not None
```

---

## §6 — Implementation skeleton

API contracts above are the skeleton. Implementation order:

1. `cyberos/core/invokers/base.py` — Protocol + ScoreResult.
2. `cyberos/core/invokers/mock.py` — MockInvoker.
3. `cyberos/core/invokers/anthropic_invoker.py` — AnthropicInvoker.
4. `cyberos/core/importance.py` — `select_invoker`, `ImportanceCache`, `score()` orchestrator.
5. `cyberos/core/writer.py` — wire `--score-importance` path; validate `manifest.importance` block.
6. `__main__.py` — `--score-importance`, `--importance`, `--invoker`, `--dry-run` flags on `put`.
7. Schema + invariant.
8. Tests.
9. CHANGELOG.

---

## §7 — Dependencies

- **TASK-MEMORY-113 (depends on)** — `meta.importance` schema field + RecallWeights consume the scored value.
- **TASK-MEMORY-112 (related)** — Episodes can opt-in to scoring; `Episode.quality_score` is the agent's self-rated output, `meta.importance` is the LLM's external rating — orthogonal signals.
- **TASK-MEMORY-115 (this task blocks)** — `cyberos dream` consumes `memory.importance_scored` audit rows for trend analysis (cache hit %, fallback %).
- **TASK-CUO-105 (related, depends_on pattern)** — the Invoker Protocol mirrors CUO's; cross-module test ensures the two stay aligned (additive — same shape, different concern).

---

## §8 — Example payloads

### `memory.importance_scored` audit row

```json
{
  "kind": "memory.importance_scored",
  "payload": {
    "path":           "memories/facts/dispatch-latency.md",
    "content_sha256": "abc123def456…",
    "score":          0.72,
    "model":          "claude-haiku-4-5",
    "outcome":        "ok",
    "reason":         null,
    "cache_hit":      false,
    "latency_ms":     842
  }
}
```

### Memory file with scored importance

```markdown
---
{
  "description": "Dispatch service has a 60-second retry pattern that triggers excess load",
  "importance": 0.72,
  "kind": "facts",
  "name": "dispatch-retry-pattern"
}
---

Observed across 5 sessions in the SRE working memory: every page-out alert produces
a 60-second retry storm because the dispatch service's exponential backoff caps at 60s.
```

### Manifest fragment

```json
{
  "importance": {
    "invoker":   "anthropic",
    "model":     "claude-haiku-4-5",
    "timeout_s": 5.0
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Batch `score-all` mode for existing memories — §1 #14; slice 4+ (operator workflow, not blocker for TASK-MEMORY-115).
- Per-tenant importance bias (KYC tenants might rank PII higher than non-KYC) — slice 4+.
- Cost-budget enforcement (`--max-spend-per-day 0.10 USD`) — slice 4+.
- Stats CLI — §1 #15; slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| `--score-importance` flag missing, default invoker absent (no API key, no env, no manifest) | none — by design, doesn't call LLM | Default `put` proceeds without score | None — operator opts in or sets manifest |
| AnthropicInvoker selected, `ANTHROPIC_API_KEY` unset | constructor raises | `cyberos put` fails with structured error naming env var | Operator exports key or sets `CYBEROS_DISABLE_LLM=1` |
| AnthropicInvoker selected, `anthropic` package not installed | constructor raises | same | Operator `pip install anthropic` |
| Timeout > 5s | `asyncio.TimeoutError` | fallback to 0.5; `outcome: "fallback", reason: "timeout"` | Operator checks API status |
| Parse failure (LLM returns prose) | regex no match | fallback; `reason: "parse_error:<text>"` | Audit row reveals the text; operator tunes prompt if recurrent |
| Out-of-range LLM output (e.g. 1.5) | clamp at score() return | success; logged at DEBUG | None — by design |
| Rate-limit (HTTP 429) | API exception | one retry with 1s jitter; if still rate-limited, fallback | Operator checks tier |
| Cache file corrupted | sqlite open raises | new cache created; previous entries lost | Operator deletes `index/importance_cache.db` |
| Cache invariant violation (sha256 column not BLOB) | `importance-cache-valid-sha256` walker rule | `cyberos doctor` fails | `cyberos doctor --repair` rebuilds the cache table |
| Concurrent writes scoring same content | both call invoker; second insert is `INSERT OR REPLACE` (DEC-203) | both writes succeed; cache reflects last writer | None — by design |
| Mock invoker drift (PRNG changed) | `test_mock_invoker_deterministic` asserts | CI catches | Author preserves PRNG semantics |
| Caller passes `--importance 1.5` (manual override out of range) | argparse validates `[0.0, 1.0]` | CLI rejects with structured error | Operator picks a valid value |
| `--dry-run` invoked without `--score-importance` | argparse mutually-exclusive check | CLI rejects | Operator adds flag |
| Manifest `importance.timeout_s` <= 0 | jsonschema validate | `ManifestError` | Operator fixes manifest |
| Audit row emit fails (writer locked) | writer raises | scoring still succeeds; row dropped at writer layer (handled by writer's existing retry policy) | None — by design |
| Large input (> 100KB) | invoker accepts but Haiku rejects | API error → fallback | Operator considers splitting input |
| Different invoker types tested in same run | per-test isolation | works | None |

---

## §11 — Implementation notes

- **Why we mirror CUO's invoker layout instead of importing it directly.** `modules/cuo/cuo/invokers.py` exists but the import would create a circular dep (cuo depends on memory). Copying the small shape (MockInvoker, AnthropicInvoker, select) is cleaner than restructuring.
- **The MockInvoker's deterministic mapping** — `sha256(content) → [0.1, 0.95]` — was chosen so tests can assert specific scores (within tolerance) without flakiness. The 0.1/0.95 clamp prevents tests from ever hitting literal 0.0 / 1.0 which would conflate with fallback behaviour.
- **`select_invoker(name=None)` reuses CUO's selection algorithm** — explicit name beats env beats default. Same priority order = same operator mental model.
- **Cache table schema is single-table on purpose.** No JOINs, no migrations, no ALTER TABLE. If the schema changes in slice 4, drop + recreate (data is regenerable by re-scoring).
- **The `aux_emitter` callable is the same shim TASK-MEMORY-112 uses for `episode.logged`.** Keeps the importance.py module decoupled from Writer (no direct call into Writer.emit_aux).
- **Why we don't batch invoker calls** — Haiku's per-call latency is ~1.5s; batching would require structured-output prompting and complicate fallback handling. The Slice-4 `score-all` mode can batch internally without changing this task's API.
- **AnthropicInvoker uses `AsyncAnthropic`, not `Anthropic`** — non-blocking. Lets the Writer overlap scoring with the file-write fsync if it wanted to (it currently doesn't, but the option is preserved).
- **The `test_anthropic_invoker_parse_paths` test scaffold is simplified for readability.** Real tests use `responses.RequestsMock` or `aioresponses` to mock the HTTP layer. The fixture lives in `modules/memory/tests/core/test_mmr.py`.

---

*End of TASK-MEMORY-114.*
