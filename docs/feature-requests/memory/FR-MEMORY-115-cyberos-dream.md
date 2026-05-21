---
id: FR-MEMORY-115
title: "memory dreaming — `cyberos dream` out-of-band batch reflection that mines transcripts + episode rows for cross-session patterns; produces reviewable diffs that consolidate / mark-stale / verify / propose-new memories under explicit operator gate"
module: memory
priority: SHOULD
status: draft
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_frs: [FR-MEMORY-112, FR-MEMORY-113, FR-MEMORY-114, FR-MEMORY-116, FR-MEMORY-119, FR-MEMORY-120, FR-CUO-105]
depends_on: [FR-MEMORY-112, FR-MEMORY-113, FR-MEMORY-114]
blocks: [FR-MEMORY-116]
protocol_amendment_required: "AGENTS.md §7.7 (new) — dreaming-applied audit rows MUST carry extra.dream_id + extra.proposal_id provenance; approval phrase: APPROVE protocol change P19 §7.7"

source_pages:
  - playground/extracts/memory-and-dreaming.transcript.txt  # see talk segments at [681:39 - 1419:04]
source_decisions:
  - DEC-210 (Dreaming is OUT-OF-BAND — runs separately from any agent session; never adds latency to the hot path; per Anthropic talk "design perspective" segment)
  - DEC-211 (Dreaming produces a DIFF that requires explicit `cyberos dream apply` to merge into the chain; mirrors the talk's "operator review gate" design; no auto-apply by default)
  - DEC-212 (Dream-applied audit rows MUST carry `extra.dream_id` + `extra.proposal_id` provenance — requires AGENTS.md §7.7 amendment per chat-turn rule §0.2)
  - DEC-213 (Dream inputs are EITHER recent audit rows OR the session-transcript ledger from FR-MEMORY-119; FR-MEMORY-115 ships with audit-row input as default; transcript input is a stretch via FR-MEMORY-119 integration)
  - DEC-214 (Proposal kinds are CLOSED enum `merge | stale | new | verify`; "ambiguous" proposals are rejected at generation time so the operator review surface stays clean)
  - DEC-215 (Dream-quality is itself a tracked objective per the Anthropic talk; `memory.dream_completed` audit row carries `quality_metrics: {proposals_count, applied_count, …}` so successive dream runs can be A/B tested)

language: python 3.10
service: modules/memory/cyberos/
new_files:
  - modules/memory/cyberos/core/dream/__init__.py
  - modules/memory/cyberos/core/dream/runner.py
  - modules/memory/cyberos/core/dream/proposals.py
  - modules/memory/cyberos/core/dream/detectors/__init__.py
  - modules/memory/cyberos/core/dream/detectors/duplicates.py
  - modules/memory/cyberos/core/dream/detectors/stale.py
  - modules/memory/cyberos/core/dream/detectors/patterns.py
  - modules/memory/cyberos/core/dream/detectors/verify.py
  - modules/memory/cyberos/cli/dream.py
  - modules/memory/tests/test_dream_runner.py
  - modules/memory/tests/test_dream_detectors.py
  - modules/memory/tests/test_dream_apply.py
  - modules/memory/tests/fixtures/dream_inputs/
modified_files:
  - modules/memory/cyberos/__main__.py                  # wire `cyberos dream` + `cyberos dream apply` subcommands
  - modules/memory/cyberos/core/writer.py               # accept `extra` fields on `put`/`delete` rows; validate `extra.dream_id`/`extra.proposal_id` form
  - modules/memory/cyberos/core/walker.py               # invariant: every dream-applied row has well-formed extras
  - modules/memory/memory.schema.json                   # `DreamDiff`, `DreamProposal`, `DreamProposalKind` definitions
  - modules/memory/memory.invariants.yaml               # `dream-applied-row-has-provenance` + `dream-diff-schema-valid`
  - AGENTS.md                                            # add §7.7 Dreaming (REQUIRES amendment via APPROVE chat-turn P19 §7.7)
allowed_tools:
  - file_read: modules/memory/**, playground/extracts/**
  - file_write: modules/memory/cyberos/**, modules/memory/tests/**, modules/memory/memory.schema.json, modules/memory/memory.invariants.yaml, AGENTS.md
  - bash: cd modules/memory && python -m pytest tests/test_dream_*.py -v
  - bash: cd modules/memory && python -m cyberos dream --since 24h --scope memories/facts --dry-run
disallowed_tools:
  - auto-apply dream proposals without explicit `dream apply` invocation (per §1 #4, DEC-211)
  - mutate AGENTS.md §7.7 without an APPROVE protocol change P19 §7.7 chat-turn (per §0.2 of the protocol itself)
  - emit `put`/`delete` rows without `extra.dream_id` if the call originated from `cyberos.core.dream.runner` (per §1 #11)
  - hit the production Anthropic API in unit tests (use MockInvoker — same DEC-201 pattern as FR-MEMORY-114)

effort_hours: 32
sub_tasks:
  - "1.0h: AGENTS.md §7.7 amendment text drafted (requires Stephen APPROVE chat-turn before merge)"
  - "1.5h: memory.schema.json — `DreamDiff` (dream_id, scope, input_sessions[], proposals[], metrics) + `DreamProposal` (op enum, paths, rationale, content_preview)"
  - "0.5h: memory.invariants.yaml — `dream-applied-row-has-provenance` (error) + `dream-diff-schema-valid` (error)"
  - "2.0h: cyberos/core/dream/proposals.py — `DreamProposal` + `DreamDiff` dataclasses with JSON round-trip; `apply()` method that emits `put`/`delete`/`move` ops with proper `extra` provenance"
  - "3.0h: cyberos/core/dream/detectors/duplicates.py — pairwise cosine-similarity over `meta.body_hash`-distinct memories within scope; ≥ 0.92 threshold; emits `merge` proposals"
  - "3.5h: cyberos/core/dream/detectors/stale.py — looks for memories whose claims contradict more-recent audit rows (using LLM judgement from cuo Phase-3 invoker pattern, fallback to TF-IDF text-similarity hint); emits `stale` proposals"
  - "5.0h: cyberos/core/dream/detectors/patterns.py — cross-session pattern detection over `episode.logged` rows (FR-MEMORY-112 dependency); identifies recurring task/outcome combos; emits `new` proposals under `memories/refinements/`"
  - "2.0h: cyberos/core/dream/detectors/verify.py — finds memories whose facts were used + observed-still-true in recent sessions; emits `verify` proposals (no body change, just `meta.last_verified_at` update)"
  - "3.0h: cyberos/core/dream/runner.py — orchestrator: spin sub-tasks for each detector, aggregate, produce DreamDiff at `dreams/<utc-timestamp>/diff.json`, emit `dream.start`/`dream.complete` audit rows"
  - "2.0h: cyberos/cli/dream.py — `cyberos dream [--since 24h] [--scope <path>] [--detectors ...] [--invoker ...] [--dry-run]` + `cyberos dream apply <dream_id> [--proposal-ids ...] [--interactive]`"
  - "1.5h: writer.py — accept `extra` dict on `put`/`delete`/`move`; validate `extra.dream_id` ULID form when origin is dream"
  - "3.5h: tests/test_dream_runner.py — 14 cases (end-to-end fixture-driven run, dry-run produces diff but no apply, apply replays into chain, idempotent re-apply rejected, dream_id provenance carried, detectors aggregated correctly)"
  - "2.0h: tests/test_dream_detectors.py — 18 cases (4 per detector + cross-detector dedup; deterministic outputs given fixed inputs)"
  - "1.5h: tests/test_dream_apply.py — 8 cases (apply with --proposal-ids filter, --interactive simulated stdin, idempotency, audit-row provenance valid, rollback semantics on partial failure)"
  - "1.0h: tests/fixtures/dream_inputs/ — JSONL fixtures simulating 3 dream scenarios (5 duplicates / 2 contradictions / 1 cross-session pattern; 1 verification candidate)"
risk_if_skipped: "Without dreaming, the memory's memory-quality objective is permanently coupled to the task-completion objective — every session sees only its own context and can't notice cross-session patterns. The Anthropic talk's two cited customer outcomes (Harvey 6× task completion; Rakerton 90% mistake-drop) come from precisely this separation. Skipping means: (a) the 0.5M-row cyberos memory keeps accumulating duplicates because nothing dedupes them; (b) stale entries (e.g. 'Linear project INGEST' after we moved to Jira) never get caught until manual cleanup; (c) cross-FR / cross-session learnings ('5 different sessions all had to re-discover the §1↔§4↔§5 traceability rule') never crystallise into a `memories/refinements/` entry. The talk's framing — `memory is going to be increasingly important and load bearing` — is the precise reason this FR is the headline of the 2026-Q3 wave. Worse: FR-MEMORY-116 (`semantic-dedup consolidate`) is a strict subset of FR-MEMORY-115's `duplicates` detector — without FR-MEMORY-115, FR-MEMORY-116 has nowhere to live."
---

## §1 — Description (BCP-14 normative)

The dreaming subsystem is a **batch asynchronous process** that mines the memory's recent state for memory-quality improvements and produces a reviewable diff. It runs **out of band** from any agent session and has its own audit identity. The contract:

1. **MUST** be triggerable via `cyberos dream` CLI invocation, with a per-run `dream_id` (ULID, 26-char base32) generated at start. Triggers fall into three classes:
    - **Manual**: `cyberos dream --since 24h` (operator on-demand)
    - **Cron**: identical CLI invocation via OS-level scheduler (covered by FR-MEMORY-110-style automation runbook; not part of this FR's scope)
    - **API**: `POST /api/v2/dream` on `cyberos serve` (deferred to slice 4; CLI is the slice-3 surface)
2. **MUST NOT** run in-band with any normal `cyberos put` / `cyberos read` / `cyberos search` operation. The dream runner takes a snapshot (HEAD seq + chain tip) at start and operates against that snapshot; concurrent writes from other processes proceed normally and are integrated on the next dream run, not preempted.
3. **MUST** support the following four detector types (closed enum DEC-214) that produce a `DreamProposal`:

    | Detector | Output proposal `op` | What it does |
    |---|---|---|
    | `duplicates` | `merge` | finds memories with body cosine-sim ≥ 0.92; proposes consolidation into one canonical with the others becoming `extra.merged_into` provenance |
    | `stale` | `stale` | finds memories contradicted by more-recent audit rows OR explicit user `correction_to:` rows; proposes tombstone with rationale |
    | `patterns` | `new` | finds recurring task/outcome combos across `episode.logged` rows (FR-MEMORY-112 dep); proposes a NEW memory under `memories/refinements/` summarising the pattern |
    | `verify` | `verify` | finds memories whose facts were actually used in recent sessions and didn't trigger correction; proposes annotating `meta.last_verified_at` (no body change) |
4. **MUST NOT** auto-apply proposals. The runner produces a `DreamDiff` JSON artefact at `dreams/<utc-timestamp>/diff.json` (with `manifest.json:dreams.retention_days` default = 90 governing cleanup). A separate `cyberos dream apply <dream_id>` call is REQUIRED to merge proposals into the chain. Subset application via `--proposal-ids` is supported.
5. **MUST** emit two audit rows per dream run (regardless of apply state):
    - `dream.start`: payload `{dream_id, scope, since, detectors, invoker, started_at}`
    - `dream.complete`: payload `{dream_id, proposals_count, applied_count: 0, duration_ms, quality_metrics: {…}}`
6. **MUST** validate that every `cyberos.core.writer.Writer` operation originating from `dream apply` carries `extra.dream_id` + `extra.proposal_id`. The writer enforces this via constructor flag `dream_origin: bool = False`; the dream applier sets it True. Walker invariant `dream-applied-row-has-provenance` catches drift.
7. **MUST** support `--dry-run` on `cyberos dream` — runs the full detection pipeline + writes the diff to disk, but emits a `dream.complete` row with `applied_count: 0` and never advances any memory file. The diff file IS written so operators can inspect it.
8. **MUST** support `--detectors duplicates,stale,patterns,verify` (default: all) and `--scope <path>` (default: full memory) for narrow runs. A scope like `memories/facts/` only walks that subtree.
9. **MUST** select the active LLM invoker via the same chain as FR-MEMORY-114 (`--invoker {mock|anthropic}` → `CYBEROS_DREAM_INVOKER` → `manifest.dream.invoker` → default `mock` if no API key). Same `CYBEROS_DISABLE_LLM=1` escape hatch.
10. **MUST** be idempotent in the strict sense: re-running `cyberos dream apply <dream_id>` with the same proposal-id set is a no-op IF nothing has changed about the affected memories between the two applies. The applier records the body_hash of each affected memory at first-apply time; on re-apply, if the on-disk hash matches the recorded one AND the row already exists in the chain, no new row is emitted. If the on-disk hash differs (someone edited the memory meanwhile), the re-apply REFUSES with `proposal_preconditions_failed` rather than overwriting.
11. **MUST** require the AGENTS.md §7.7 amendment APPROVED via `APPROVE protocol change P19 §7.7` chat-turn before any apply rows are emitted. The CLI loads AGENTS.md and checks for the §7.7 anchor at runtime; missing → refuse with structured error pointing at the protocol section.
12. **MUST** complete a typical dream run (input ≤ 10K memories, ≤ 5K episode rows, ≤ 30-day audit-row window) in ≤ 5 minutes wall time with `--invoker anthropic`, ≤ 1 minute with `--invoker mock`. The runner emits incremental progress to stderr.
13. **MUST** support `--interactive` on `cyberos dream apply` — prompts per-proposal with the rationale + body preview; operator types `y` / `n` / `s` (skip remaining); ESC quits without partial-apply.
14. **MUST** preserve full memory integrity on dream apply — if any single proposal's write fails (e.g. ACL denied per FR-MEMORY-117), the entire apply transaction rolls back to the snapshot HEAD seq. No half-applied diffs.
15. **MUST** emit `dream.proposal_applied` aux audit rows per successful proposal application — payload `{dream_id, proposal_id, op, affected_paths[], applied_at}`.
16. **MUST** track quality metrics over time. Each `dream.complete` row's `quality_metrics` MUST include: `{proposals_count_by_kind, applied_count_by_kind, fallback_count, avg_invoker_latency_ms, scope_size_memories, scope_size_audit_rows}`. These feed FR-MEMORY-120's `cyberos history` retrospective view.
17. **SHOULD** support `cyberos dream review <dream_id>` — renders the diff in a human-friendly table view (path / op / rationale / preview). Slice-4 polish, not blocker.
18. **SHOULD** support detector plug-ins via `entry_points["cyberos.dream.detectors"]` — third-party detectors can register without forking. Slice-4+ stretch.

---

## §2 — Why this design (rationale for humans)

**Why out-of-band, not in-session (§1 #2, DEC-210).** The Anthropic talk's most-emphasised design property. In-session reflection means the agent's task latency tax goes up for memory-quality work that benefits *future* sessions, not the current one. Out-of-band means the operator chooses when to pay the latency (overnight cron, post-task batch, on-demand investigation). Concurrent writes during a dream run integrate on the next dream — no need to fight for the snapshot.

**Why operator-gated apply, not auto-apply (§1 #4, DEC-211).** The talk repeatedly returns to this. Apply has memory-mutating consequences (tombstone, merge, new refinement). Apply must be auditable + reviewable, not surprising. Three patterns the talk endorses: (a) human review then apply; (b) review via API + auto-apply with rule-based filter; (c) auto-apply for low-risk proposals (verify-only). FR-MEMORY-115 ships pattern (a); future work can opt-in (b) and (c) by extending the CLI.

**Why ULID dream_id (§1 #1).** ULIDs are time-sortable + URL-safe + 128-bit. Beats UUID v4 for chronological scanning ("all dreams since X"). Beats sequential integers for distribution (multiple operators can run dreams without coordination). Matches the existing audit-row ID convention.

**Why 4 closed proposal kinds (§1 #3, DEC-214).** Open ops space invites detector drift. Each new "kind" expands the apply-side validation surface. The 4 ops cover the talk's enumerated scenarios verbatim: dedup, stale-removal, new-pattern, verification. Slice-4 can add `relocate` (move to better-scoped store, gated on FR-MEMORY-117 ACLs) and `rescore` (adjust importance after observation).

**Why `dreams/<utc-timestamp>/` directory (§1 #4).** Two reasons: (a) timestamp prefix sorts chronologically; (b) directory-per-dream lets the diff coexist with `notes.md`, intermediate logs, applied-rows-summary file — future expansion lives in the same dir. The retention policy lives on `manifest.json` so operators can tune per project.

**Why preconditioned re-apply (§1 #10).** Idempotency without preconditions is dangerous: imagine a stale-tombstone applied today, then someone resurrects the content tomorrow, then `dream apply --proposal-ids tombstone-prop` is re-run — would clobber the resurrection. Preconditions ensure the apply only proceeds if the memory's state matches what the proposal was generated against. The body_hash comparison is the natural primitive (already in `meta.body_hash`).

**Why AGENTS.md §7.7 amendment required (§1 #11, DEC-212).** The protocol's §0.2 immutability gate. Adding `extra.dream_id` provenance is a new normative requirement on the writer ("rows from this origin MUST carry this metadata"). That's a protocol change, not a config change. The CLI's runtime check for the §7.7 anchor enforces the gate: an operator who installs the FR's code but hasn't APPROVED the protocol change can't accidentally emit non-compliant rows.

**Why 5-minute wall-time budget (§1 #12).** Dream is meant to run nightly. 5 minutes fits within a `0 2 * * *` cron slot without contending with other automation. The mock-invoker fast path (≤ 1 min) keeps tests + CI green.

**Why transactional apply (§1 #14).** Half-applied diffs are worse than not-applied. The talk's design notes call out "produces an updated memory state that you can then apply immediately"; the implication is atomicity. Practically: the applier opens a single Writer transaction (acquires `.lock`), validates every proposal's precondition, then emits each proposal's rows under the same lock. Any failure → release lock + rollback at HEAD.

**Why per-proposal aux row instead of one bulk row (§1 #15).** FR-MEMORY-120 (`cyberos history`) needs per-path provenance: "this memory was tombstoned by dream X, proposal Y." Bulk rows force history to re-parse the payload to find which path was affected. Per-proposal rows are cheap (audit chain is designed for high cardinality) and make history queries trivial.

**Why explicit detector flag (§1 #8).** Operators want to run individual detectors during investigation ("just look for duplicates in `memories/facts/`"). Default-all-on means the canonical full run is one command; opt-in subsets means the focused run is also one command. The flag is the affordance.

---

## §3 — API contract

### Schema

```json
{
  "$defs": {
    "DreamProposalKind": {
      "type": "string",
      "enum": ["merge", "stale", "new", "verify"]
    },
    "DreamProposal": {
      "type": "object",
      "required": ["proposal_id", "op", "rationale"],
      "properties": {
        "proposal_id":  {"type": "string", "pattern": "^P[0-9A-Z]{8}$"},
        "op":           {"$ref": "#/$defs/DreamProposalKind"},
        "paths":        {"type": "array", "items": {"type": "string"}, "default": []},
        "into":         {"type": "string"},
        "content_preview": {"type": "string", "maxLength": 2048},
        "rationale":    {"type": "string", "minLength": 1},
        "input_session_ids": {"type": "array", "items": {"type": "string"}, "default": []},
        "input_audit_seqs":  {"type": "array", "items": {"type": "integer", "minimum": 1}, "default": []},
        "precondition_body_hashes": {"type": "object", "additionalProperties": {"type": "string"}, "default": {}}
      }
    },
    "DreamDiff": {
      "type": "object",
      "required": ["dream_id", "scope", "since", "proposals", "metrics"],
      "properties": {
        "dream_id": {"type": "string", "pattern": "^[0-9A-HJKMNP-TV-Z]{26}$"},
        "scope":    {"type": "string"},
        "since":    {"type": "string", "format": "date-time"},
        "input_sessions": {"type": "array", "items": {"type": "string"}, "default": []},
        "proposals": {"type": "array", "items": {"$ref": "#/$defs/DreamProposal"}},
        "metrics":  {"type": "object", "additionalProperties": true}
      }
    }
  }
}
```

### Core dataclasses

```python
# modules/memory/cyberos/core/dream/proposals.py
from __future__ import annotations
from dataclasses import dataclass, field
from typing import Literal, Optional

ProposalKind = Literal["merge", "stale", "new", "verify"]


@dataclass
class DreamProposal:
    proposal_id:  str          # "P" + 8 random base32 chars
    op:           ProposalKind
    paths:        list[str]    = field(default_factory=list)
    into:         Optional[str] = None
    content_preview: str       = ""
    rationale:    str          = ""
    input_session_ids:         list[str] = field(default_factory=list)
    input_audit_seqs:          list[int] = field(default_factory=list)
    precondition_body_hashes:  dict[str, str] = field(default_factory=dict)


@dataclass
class DreamDiff:
    dream_id: str    # ULID
    scope:    str
    since:    str    # ISO timestamp
    input_sessions: list[str]
    proposals: list[DreamProposal]
    metrics:  dict
```

### Runner skeleton

```python
# modules/memory/cyberos/core/dream/runner.py
from __future__ import annotations
import asyncio, json, time
from dataclasses import asdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Optional

from cyberos.core.writer import Writer
from cyberos.core.dream.proposals import DreamProposal, DreamDiff
from cyberos.core.dream.detectors import duplicates, stale, patterns, verify


async def run(
    writer:        Writer,
    since:         timedelta,
    scope:         str            = "",
    detector_names: list[str]     = ("duplicates", "stale", "patterns", "verify"),
    invoker_name:  Optional[str]  = None,
    dry_run:       bool           = False,
) -> DreamDiff:
    dream_id = _generate_ulid()
    started_at = datetime.now(timezone.utc)

    # snapshot guard
    snapshot_head = writer.head_seq()

    writer.emit_aux(
        kind="dream.start",
        payload={
            "dream_id":   dream_id,
            "scope":      scope or "*",
            "since":      (started_at - since).isoformat(),
            "detectors":  list(detector_names),
            "invoker":    invoker_name or "default",
            "started_at": started_at.isoformat(),
        },
        actor="dream-runner",
    )

    detectors = {
        "duplicates": duplicates.run,
        "stale":      stale.run,
        "patterns":   patterns.run,
        "verify":     verify.run,
    }

    all_proposals: list[DreamProposal] = []
    for name in detector_names:
        if name not in detectors:
            raise ValueError(f"unknown detector {name!r}; expected one of {sorted(detectors)}")
        proposals = await detectors[name](writer, since, scope, invoker_name)
        all_proposals.extend(proposals)

    # de-dup proposals that touch the same path (e.g. duplicates and stale both fire on one path)
    seen = set()
    dedup: list[DreamProposal] = []
    for p in all_proposals:
        key = (p.op, tuple(sorted(p.paths)))
        if key in seen: continue
        seen.add(key)
        dedup.append(p)

    diff = DreamDiff(
        dream_id=dream_id,
        scope=scope or "*",
        since=(started_at - since).isoformat(),
        input_sessions=[],  # populated when FR-MEMORY-119 lands
        proposals=dedup,
        metrics={
            "proposals_count_by_kind": _bucket_by_kind(dedup),
            "scope_size_memories":     writer.count_memories(scope),
            "scope_size_audit_rows":   writer.count_audit_rows_since(started_at - since),
            "snapshot_head":           snapshot_head,
        },
    )

    out_dir = writer.store_path / "dreams" / started_at.strftime("%Y%m%dT%H%M%SZ")
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "diff.json").write_text(json.dumps(_diff_to_dict(diff), indent=2, sort_keys=True))

    writer.emit_aux(
        kind="dream.complete",
        payload={
            "dream_id":        dream_id,
            "proposals_count": len(dedup),
            "applied_count":   0,                            # always 0 here — apply is a separate command
            "duration_ms":     int((time.time() - started_at.timestamp()) * 1000),
            "quality_metrics": diff.metrics,
            "dry_run":         dry_run,
        },
        actor="dream-runner",
    )
    return diff


def _generate_ulid() -> str:
    """26-char Crockford base32 ULID. Real impl uses `python-ulid` package."""
    import secrets, time
    alpha = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
    ms = int(time.time() * 1000)
    return "".join(alpha[(ms >> (5 * i)) & 0x1F] for i in range(10))[::-1] + "".join(
        secrets.choice(alpha) for _ in range(16)
    )


def _bucket_by_kind(proposals: list[DreamProposal]) -> dict:
    out = {"merge": 0, "stale": 0, "new": 0, "verify": 0}
    for p in proposals:
        out[p.op] += 1
    return out


def _diff_to_dict(d: DreamDiff) -> dict:
    return {
        "dream_id": d.dream_id, "scope": d.scope, "since": d.since,
        "input_sessions": d.input_sessions,
        "proposals": [asdict(p) for p in d.proposals],
        "metrics": d.metrics,
    }
```

### Applier

```python
# modules/memory/cyberos/core/dream/applier.py
from __future__ import annotations
import hashlib
from pathlib import Path
from typing import Optional

from cyberos.core.writer import Writer
from cyberos.core.dream.proposals import DreamDiff, DreamProposal


class PreconditionFailed(Exception):
    pass


def apply(writer: Writer, diff: DreamDiff, proposal_ids: Optional[set[str]] = None,
          dream_id: Optional[str] = None) -> dict:
    """Apply selected proposals from a DreamDiff. Returns metrics."""
    target_dream_id = dream_id or diff.dream_id
    targets = [p for p in diff.proposals if proposal_ids is None or p.proposal_id in proposal_ids]

    # Verify §7.7 anchor exists before any writes
    if not _agents_md_has_section_7_7(writer.store_path):
        raise RuntimeError(
            "AGENTS.md does not contain §7.7 Dreaming. Approve via:\n"
            "APPROVE protocol change P19 §7.7"
        )

    # Precondition pass
    for p in targets:
        for path, expected_hash in p.precondition_body_hashes.items():
            actual = _sha256_body(writer.store_path / path)
            if actual != expected_hash:
                raise PreconditionFailed(
                    f"Proposal {p.proposal_id}: path {path} body_hash drift "
                    f"(expected {expected_hash[:12]}…, got {actual[:12]}…)"
                )

    # Apply pass under exclusive lock
    applied_count = 0
    with writer.exclusive_lock(reason=f"dream-apply:{target_dream_id}") as txn:
        txn.set_origin(dream_id=target_dream_id)
        for p in targets:
            _apply_one(txn, p)
            applied_count += 1
        writer.emit_aux(
            kind="dream.proposal_applied",
            payload={
                "dream_id":      target_dream_id,
                "proposal_id":   p.proposal_id,
                "op":            p.op,
                "affected_paths": p.paths,
                "applied_at":    txn.now_iso(),
            },
            actor="dream-applier",
        )
    return {"applied_count": applied_count, "rejected": len(diff.proposals) - applied_count}


def _apply_one(txn, p: DreamProposal):
    if p.op == "merge":
        # body of first path becomes the canonical; others tombstoned with extra.merged_into
        canonical = p.into or p.paths[0]
        for src in p.paths[1:]:
            txn.delete(src, mode="tombstone",
                       extra={"merged_into": canonical, "proposal_id": p.proposal_id})
    elif p.op == "stale":
        for path in p.paths:
            txn.delete(path, mode="tombstone",
                       extra={"reason": "stale", "proposal_id": p.proposal_id,
                              "rationale": p.rationale})
    elif p.op == "new":
        txn.put(p.paths[0], p.content_preview,
                extra={"proposal_id": p.proposal_id, "source": "dream"})
    elif p.op == "verify":
        for path in p.paths:
            # update meta.last_verified_at; body unchanged
            txn.touch_meta(path, {"last_verified_at": txn.now_iso()},
                           extra={"proposal_id": p.proposal_id})
    else:
        raise ValueError(f"unknown proposal op {p.op!r}")


def _agents_md_has_section_7_7(store_path: Path) -> bool:
    agents_md = store_path / "AGENTS.md"
    if not agents_md.exists():
        return False
    body = agents_md.read_text()
    return "## §7.7" in body or "§7.7  Dreaming" in body or "§7.7 Dreaming" in body


def _sha256_body(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
```

### AGENTS.md §7.7 amendment text (proposed)

```text
## §7.7  Dreaming (added by P19 — requires APPROVE chat-turn per §0.2)

§7.7.1  Dreaming is the out-of-band batch process specified in
FR-MEMORY-115. Dream-runner and dream-applier are distinct identities;
they MUST NOT execute in-band with any agent session that mutates memory.

§7.7.2  Every audit row emitted by dream-applier MUST carry both
`extra.dream_id` (ULID matching the active `dream.start` row) and
`extra.proposal_id` (matching the source `DreamProposal.proposal_id`).
Walker invariant `dream-applied-row-has-provenance` enforces this.

§7.7.3  Dream apply MUST validate body-hash preconditions before any
writes. A precondition failure on any proposal in a batch aborts the
batch; no half-applied state.

§7.7.4  Dream proposals are EITHER applied via `cyberos dream apply`
(operator-gated) OR remain on disk as `dreams/<ts>/diff.json` artefacts
until apply or retention-expiry. The protocol provides no auto-apply
mechanism.
```

---

## §4 — Acceptance criteria

1. **Dream emits both start + complete rows** — `cyberos dream --since 24h` advances HEAD by exactly 2 aux rows when there are zero proposals (just `dream.start` + `dream.complete`). *(traces_to: §1 #5)*
2. **Dream emits a diff file** — same invocation writes `dreams/<ts>/diff.json` to disk; file is valid against `DreamDiff` schema. *(traces_to: §1 #4)*
3. **Dry-run never advances memory state** — `cyberos dream --dry-run` produces the diff + 2 aux rows but no `put`/`delete` rows. *(traces_to: §1 #7)*
4. **Apply requires §7.7 anchor** — `AGENTS.md` lacking the §7.7 section → `dream apply` raises with message naming `APPROVE protocol change P19 §7.7`. *(traces_to: §1 #11)*
5. **Apply advances HEAD per proposal** — diff with 3 merge proposals + §7.7 present → `dream apply <dream_id>` advances HEAD by 3 (delete rows) + 3 (aux `dream.proposal_applied`) = 6. *(traces_to: §1 #15)*
6. **Applied rows carry dream_id + proposal_id** — inspect each delete row's `extra` — both keys present and well-formed. *(traces_to: §1 #6)*
7. **Walker rejects dream-origin row without provenance** — manually emit a `put` row with `extra: {}` while `dream_origin: True` → `cyberos doctor` fails with `dream-applied-row-has-provenance`. *(traces_to: §1 #6)*
8. **Detector — duplicates threshold** — fixture with 3 memories at cosine 0.95 / 0.90 / 0.80 → only the 0.95 pair generates a merge proposal. *(traces_to: §1 #3)*
9. **Detector — stale catches contradicted memory** — fixture with `facts/x.md` saying "Linear project X" followed by `correction_to:` row stating "moved to Jira" → stale proposal for `facts/x.md` with rationale citing the correction row's seq. *(traces_to: §1 #3)*
10. **Detector — patterns identifies recurring task/outcome** — 5 `episode.logged` rows with same task fingerprint, 4 success + 1 failure → one new proposal under `memories/refinements/<task-slug>.md` with `content_preview` mentioning the 4:1 ratio. *(traces_to: §1 #3)*
11. **Detector — verify finds used-and-not-corrected** — fact memory referenced in 3 sessions over the window, never followed by a correction → verify proposal annotates `meta.last_verified_at`. *(traces_to: §1 #3)*
12. **Snake-case proposal_id** — `^P[0-9A-Z]{8}$` regex matches every generated id. *(traces_to: §1 #1 indirectly via DreamProposal schema)*
13. **ULID format** — `dream_id` matches `^[0-9A-HJKMNP-TV-Z]{26}$` (Crockford base32). *(traces_to: §1 #1)*
14. **Apply idempotent on unchanged state** — apply diff twice → second apply emits zero new rows (precondition match = no-op) and reports `applied_count: 0` with reason `idempotent`. *(traces_to: §1 #10)*
15. **Apply refuses on body-hash drift** — modify the target memory between dream + apply → second apply raises `PreconditionFailed` naming the drifted path. *(traces_to: §1 #10)*
16. **Apply transactional** — diff with 3 proposals where #2 will fail (e.g. nonexistent path) → entire apply rolls back; HEAD unchanged. *(traces_to: §1 #14)*
17. **Detectors flag** — `cyberos dream --detectors duplicates,stale` → patterns and verify detectors not invoked. *(traces_to: §1 #8)*
18. **Scope flag** — `cyberos dream --scope memories/facts` → no proposals reference paths outside `memories/facts/`. *(traces_to: §1 #8)*
19. **Invoker selection chain** — `--invoker anthropic` + missing key → constructor raises; `--invoker mock` always works; default with `ANTHROPIC_API_KEY` set → AnthropicInvoker. *(traces_to: §1 #9)*
20. **`CYBEROS_DISABLE_LLM=1` forces mock** — env set, manifest says anthropic → MockInvoker. *(traces_to: §1 #9)*
21. **Wall-time budget (mock)** — 10K-memory + 5K-episode fixture, mock invoker → run completes in ≤ 60 s on CI hardware. *(traces_to: §1 #12)*
22. **Interactive apply** — `dream apply --interactive` with mocked stdin "y\nn\ns\n" → first proposal applied, second skipped, remaining skipped after `s`. *(traces_to: §1 #13)*
23. **`dream.complete` quality_metrics shape** — payload contains `proposals_count_by_kind: {merge, stale, new, verify}`, `scope_size_memories`, `scope_size_audit_rows`, `snapshot_head`. *(traces_to: §1 #16)*
24. **Snapshot isolation** — dream-runner takes a snapshot; concurrent `cyberos put` during run does not change which memories the detectors see (verified by adding a memory mid-run via second process; diff doesn't include it). *(traces_to: §1 #2)*
25. **Diff retention default** — `manifest.json:dreams.retention_days` defaults to 90 if absent. *(traces_to: §1 #4)*
26. **Closed proposal kind enum** — handcraft a `DreamProposal(op="weird")` → constructor raises; walker rejects raw-written rows. *(traces_to: DEC-214)*

---

## §5 — Verification

```python
# modules/memory/tests/test_dream_runner.py
import asyncio, json, re
from datetime import timedelta
from pathlib import Path

import pytest
from cyberos.core.dream.runner import run as dream_run


@pytest.mark.asyncio
async def test_dream_emits_start_and_complete_rows(empty_memory):
    """AC #1"""
    head_before = empty_memory.head_seq()
    diff = await dream_run(empty_memory, since=timedelta(hours=24), invoker_name="mock")
    assert empty_memory.head_seq() == head_before + 2     # exactly start + complete
    assert diff.proposals == []


@pytest.mark.asyncio
async def test_dream_writes_diff_file(empty_memory):
    """AC #2"""
    diff = await dream_run(empty_memory, since=timedelta(hours=24), invoker_name="mock")
    dreams_dir = empty_memory.store_path / "dreams"
    diff_files = list(dreams_dir.rglob("diff.json"))
    assert len(diff_files) == 1
    content = json.loads(diff_files[0].read_text())
    assert content["dream_id"] == diff.dream_id


@pytest.mark.asyncio
async def test_dry_run_no_proposal_rows(seeded_memory_with_dupes):
    """AC #3"""
    head_before = seeded_memory_with_dupes.head_seq()
    diff = await dream_run(seeded_memory_with_dupes, since=timedelta(hours=24),
                            invoker_name="mock", dry_run=True)
    # Even though proposals were found, no apply rows emitted
    assert seeded_memory_with_dupes.head_seq() == head_before + 2
    assert len(diff.proposals) > 0


@pytest.mark.asyncio
async def test_dream_id_is_ulid_format(empty_memory):
    """AC #13"""
    diff = await dream_run(empty_memory, since=timedelta(hours=24), invoker_name="mock")
    assert re.fullmatch(r"[0-9A-HJKMNP-TV-Z]{26}", diff.dream_id)


@pytest.mark.asyncio
async def test_proposal_id_format(seeded_memory_with_dupes):
    """AC #12"""
    diff = await dream_run(seeded_memory_with_dupes, since=timedelta(hours=24),
                            invoker_name="mock")
    for p in diff.proposals:
        assert re.fullmatch(r"P[0-9A-Z]{8}", p.proposal_id), p.proposal_id


@pytest.mark.asyncio
async def test_dream_detectors_filter(seeded_memory_with_dupes):
    """AC #17"""
    diff = await dream_run(seeded_memory_with_dupes, since=timedelta(hours=24),
                            detector_names=("duplicates",), invoker_name="mock")
    # No "new"/"verify"/"stale" proposals expected
    kinds = {p.op for p in diff.proposals}
    assert kinds.issubset({"merge"})


@pytest.mark.asyncio
async def test_dream_scope_filter(seeded_memory_with_dupes):
    """AC #18"""
    diff = await dream_run(seeded_memory_with_dupes, since=timedelta(hours=24),
                            scope="memories/facts", invoker_name="mock")
    for p in diff.proposals:
        for path in p.paths:
            assert path.startswith("memories/facts"), path


@pytest.mark.asyncio
async def test_dream_quality_metrics_shape(seeded_memory_with_dupes):
    """AC #23"""
    diff = await dream_run(seeded_memory_with_dupes, since=timedelta(hours=24),
                            invoker_name="mock")
    qm = diff.metrics
    for key in ("proposals_count_by_kind", "scope_size_memories",
                "scope_size_audit_rows", "snapshot_head"):
        assert key in qm
    for kind in ("merge", "stale", "new", "verify"):
        assert kind in qm["proposals_count_by_kind"]


@pytest.mark.asyncio
async def test_unknown_detector_raises(empty_memory):
    with pytest.raises(ValueError, match="unknown detector"):
        await dream_run(empty_memory, since=timedelta(hours=24),
                         detector_names=("made_up",), invoker_name="mock")


@pytest.mark.asyncio
async def test_disable_llm_forces_mock(empty_memory, monkeypatch):
    """AC #20"""
    monkeypatch.setenv("CYBEROS_DISABLE_LLM", "1")
    diff = await dream_run(empty_memory, since=timedelta(hours=24), invoker_name="anthropic")
    # No error raised — fell back to mock
    assert diff.dream_id


@pytest.mark.asyncio
async def test_snapshot_isolation(seeded_memory, second_process_writer):
    """AC #24 — concurrent write during run doesn't change the dream's view."""
    snapshot_head = seeded_memory.head_seq()
    # Trigger a dream + simulate a concurrent write halfway through (via fixture)
    diff = await dream_run(seeded_memory, since=timedelta(hours=24), invoker_name="mock")
    # The concurrent write happened but the diff's snapshot_head should match what we started with
    assert diff.metrics["snapshot_head"] == snapshot_head
```

```python
# modules/memory/tests/test_dream_detectors.py
import asyncio
from datetime import timedelta
from cyberos.core.dream.detectors import duplicates, stale, patterns, verify


@pytest.mark.asyncio
async def test_duplicates_threshold(seeded_memory_with_dupes):
    """AC #8"""
    proposals = await duplicates.run(seeded_memory_with_dupes, timedelta(hours=24), "", "mock")
    # The 0.95-similar pair generates a proposal; 0.90 and 0.80 do not
    assert len(proposals) == 1
    assert proposals[0].op == "merge"


@pytest.mark.asyncio
async def test_stale_finds_contradiction(seeded_memory_with_correction):
    """AC #9"""
    proposals = await stale.run(seeded_memory_with_correction, timedelta(hours=24), "", "mock")
    assert any(p.op == "stale" and "facts/linear-project-x.md" in p.paths for p in proposals)


@pytest.mark.asyncio
async def test_patterns_identifies_recurring_episode(seeded_memory_with_pattern):
    """AC #10"""
    proposals = await patterns.run(seeded_memory_with_pattern, timedelta(hours=24), "", "mock")
    assert any(p.op == "new" and p.paths[0].startswith("memories/refinements/") for p in proposals)


@pytest.mark.asyncio
async def test_verify_finds_used_unconrected(seeded_memory_with_verified_fact):
    """AC #11"""
    proposals = await verify.run(seeded_memory_with_verified_fact, timedelta(hours=24), "", "mock")
    assert any(p.op == "verify" for p in proposals)


@pytest.mark.asyncio
async def test_detectors_deterministic(seeded_memory_with_dupes):
    """Re-running the same detector on the same fixture gives the same proposal set."""
    a = await duplicates.run(seeded_memory_with_dupes, timedelta(hours=24), "", "mock")
    b = await duplicates.run(seeded_memory_with_dupes, timedelta(hours=24), "", "mock")
    assert {p.proposal_id for p in a} == {p.proposal_id for p in b}     # mock seeds reproducible IDs
```

```python
# modules/memory/tests/test_dream_apply.py
import pytest
from cyberos.core.dream.applier import apply, PreconditionFailed


def test_apply_requires_section_7_7(seeded_memory_with_dupes, sample_diff):
    """AC #4"""
    (seeded_memory_with_dupes.store_path / "AGENTS.md").write_text("# no section 7.7")
    with pytest.raises(RuntimeError, match=r"APPROVE protocol change P19 §7\.7"):
        apply(seeded_memory_with_dupes, sample_diff)


def test_apply_advances_head_per_proposal(seeded_memory_with_dupes, sample_diff,
                                           ensure_section_7_7):
    """AC #5"""
    head_before = seeded_memory_with_dupes.head_seq()
    out = apply(seeded_memory_with_dupes, sample_diff)
    # 3 deletes + 3 aux rows = +6
    assert seeded_memory_with_dupes.head_seq() == head_before + 6
    assert out["applied_count"] == 3


def test_applied_rows_carry_provenance(seeded_memory_with_dupes, sample_diff,
                                       ensure_section_7_7):
    """AC #6"""
    head_before = seeded_memory_with_dupes.head_seq()
    apply(seeded_memory_with_dupes, sample_diff)
    # Walk new rows and confirm extra fields
    for seq in range(head_before + 1, seeded_memory_with_dupes.head_seq() + 1):
        row = seeded_memory_with_dupes.read_audit_row(seq)
        if row["kind"] in ("put", "delete"):
            assert "dream_id" in row.get("extra", {})
            assert "proposal_id" in row.get("extra", {})


def test_apply_idempotent(seeded_memory_with_dupes, sample_diff, ensure_section_7_7):
    """AC #14"""
    apply(seeded_memory_with_dupes, sample_diff)
    head_after_first = seeded_memory_with_dupes.head_seq()
    out = apply(seeded_memory_with_dupes, sample_diff)
    # Re-apply finds preconditions still satisfied (since first apply tombstoned),
    # but the tombstone row already exists → no new rows
    assert seeded_memory_with_dupes.head_seq() == head_after_first
    assert out["applied_count"] == 0


def test_apply_refuses_on_drift(seeded_memory_with_dupes, sample_diff_with_precondition,
                                 ensure_section_7_7):
    """AC #15"""
    # Modify one of the target paths between generation and apply
    p = seeded_memory_with_dupes.store_path / sample_diff_with_precondition.proposals[0].paths[0]
    p.write_text(p.read_text() + "\n# drift inserted")
    with pytest.raises(PreconditionFailed):
        apply(seeded_memory_with_dupes, sample_diff_with_precondition)


def test_apply_transactional_rollback(seeded_memory_with_dupes, bad_diff,
                                       ensure_section_7_7):
    """AC #16"""
    head_before = seeded_memory_with_dupes.head_seq()
    with pytest.raises(Exception):
        apply(seeded_memory_with_dupes, bad_diff)
    assert seeded_memory_with_dupes.head_seq() == head_before


def test_apply_proposal_id_filter(seeded_memory_with_dupes, sample_diff_3props,
                                   ensure_section_7_7):
    """Apply only one proposal id by filtering."""
    head_before = seeded_memory_with_dupes.head_seq()
    out = apply(seeded_memory_with_dupes, sample_diff_3props,
                 proposal_ids={sample_diff_3props.proposals[0].proposal_id})
    assert out["applied_count"] == 1
    assert seeded_memory_with_dupes.head_seq() == head_before + 2  # 1 delete + 1 aux


def test_apply_interactive_skip_remaining(seeded_memory_with_dupes, sample_diff_3props,
                                          ensure_section_7_7, mocked_stdin):
    """AC #22"""
    mocked_stdin.feed("y\nn\ns\n")
    out = apply(seeded_memory_with_dupes, sample_diff_3props, interactive=True)
    # First applied, second skipped, third skipped after 's'
    assert out["applied_count"] == 1
```

---

## §6 — Implementation skeleton

API contracts above are the skeleton. Implementation order:

1. AGENTS.md §7.7 amendment — author the section text (DO NOT commit until `APPROVE protocol change P19 §7.7` chat-turn).
2. Schema (memory.schema.json) — DreamDiff + DreamProposal + closed enums.
3. Walker invariants — `dream-applied-row-has-provenance`, `dream-diff-schema-valid`.
4. `cyberos/core/dream/proposals.py` — dataclasses + JSON I/O.
5. Four detector modules.
6. `cyberos/core/dream/runner.py` — orchestrator.
7. `cyberos/core/dream/applier.py` — apply + precondition + transactional.
8. `cyberos/core/writer.py` — `extra` dict support + `dream_origin` flag + `txn.set_origin`.
9. `cyberos/cli/dream.py` — `dream` + `dream apply` subcommands.
10. Tests + fixtures.
11. CHANGELOG.

---

## §7 — Dependencies

- **FR-MEMORY-112 (depends on)** — Episodes provide the `episode.logged` aux rows the patterns detector consumes.
- **FR-MEMORY-113 (depends on)** — recency decay informs detectors which memories are "old enough to be stale candidates".
- **FR-MEMORY-114 (depends on)** — importance signal feeds the stale-detector's "is this still important?" check; same Invoker pattern reused.
- **FR-MEMORY-116 (this FR enables)** — `cyberos consolidate --semantic-dedup` is a strict subset of the duplicates detector; FR-MEMORY-116 wraps it as a consolidation phase.
- **FR-MEMORY-119 (related)** — session transcript ledger provides higher-fidelity input to the patterns detector; first ship uses audit-row-only input.
- **FR-MEMORY-120 (this FR enables)** — `cyberos history <path>` surfaces "this memory was last modified by dream X proposal Y" via the per-proposal aux rows.
- **FR-CUO-105 (related)** — the LLM Invoker contract; both modules share the type signature.

---

## §8 — Example payloads

### `DreamDiff` (excerpt)

```json
{
  "dream_id": "01HJ8XVK9P0M7N5G4F3E2D1C0B",
  "scope":    "memories/sre",
  "since":    "2026-05-18T14:00:00Z",
  "input_sessions": [],
  "proposals": [
    {
      "proposal_id": "P3FQ8K2X",
      "op":          "merge",
      "paths":       ["memories/sre/dispatch-1.md", "memories/sre/dispatch-2.md"],
      "into":        "memories/sre/dispatch-1.md",
      "rationale":   "Both memories describe the 60-second retry pattern (cosine 0.94); merging preserves all observations.",
      "input_audit_seqs": [4291, 4317],
      "precondition_body_hashes": {
        "memories/sre/dispatch-1.md": "abc123…",
        "memories/sre/dispatch-2.md": "def456…"
      },
      "content_preview": "Observed across 5 sessions: every page-out alert produces a 60-second retry storm…"
    },
    {
      "proposal_id": "P7TX2N4F",
      "op":          "stale",
      "paths":       ["memories/facts/linear-project-x.md"],
      "rationale":   "Contradicted by audit row seq=4298: 'moved to Jira project PIPE'.",
      "input_audit_seqs": [4298],
      "precondition_body_hashes": {
        "memories/facts/linear-project-x.md": "xyz789…"
      }
    }
  ],
  "metrics": {
    "proposals_count_by_kind": {"merge": 1, "stale": 1, "new": 0, "verify": 0},
    "scope_size_memories":     142,
    "scope_size_audit_rows":   311,
    "snapshot_head":           4318
  }
}
```

### `dream.complete` audit row

```json
{
  "kind": "dream.complete",
  "payload": {
    "dream_id": "01HJ8XVK9P0M7N5G4F3E2D1C0B",
    "proposals_count": 2,
    "applied_count":   0,
    "duration_ms":     87420,
    "quality_metrics": {
      "proposals_count_by_kind": {"merge": 1, "stale": 1, "new": 0, "verify": 0},
      "scope_size_memories":     142,
      "scope_size_audit_rows":   311,
      "snapshot_head":           4318,
      "fallback_count":          0,
      "avg_invoker_latency_ms":  340
    },
    "dry_run": false
  }
}
```

### Apply preconditions failure

```text
$ cyberos dream apply 01HJ8XVK9P0M7N5G4F3E2D1C0B --proposal-ids P3FQ8K2X
Error: PreconditionFailed: Proposal P3FQ8K2X: path memories/sre/dispatch-1.md
body_hash drift (expected abc123…, got def890…)

The memory was modified between dream generation and apply. Re-run `cyberos dream`
to capture the new state, then re-apply.
```

### AGENTS.md anchor missing

```text
$ cyberos dream apply 01HJ8XVK9P0M7N5G4F3E2D1C0B
Error: AGENTS.md does not contain §7.7 Dreaming. Approve via:
APPROVE protocol change P19 §7.7
```

---

## §9 — Open questions

All resolved. Deferred:
- API surface on `cyberos serve` (POST /api/v2/dream) — §1 #1; slice 4.
- Auto-apply for low-risk proposals (verify-only) — §1 #4; slice 4 design discussion.
- Detector plug-ins via entry_points — §1 #18; slice 4+.
- `cyberos dream review <id>` rich table view — §1 #17; slice 4 polish.
- A/B comparison harness across dream runs — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| §7.7 not yet APPROVED | `_agents_md_has_section_7_7` check | `dream apply` raises | Operator runs the APPROVE chat-turn + commits §7.7 to AGENTS.md |
| Body-hash drift mid-window | precondition check pre-apply | `PreconditionFailed` with named path | Re-run dream, re-apply |
| Single proposal write fails (ACL denied per FR-MEMORY-117) | writer exception in transaction | entire batch rolls back | Adjust ACL or skip that proposal id |
| Detector raises unhandled exception | runner catches + emits `dream.detector_failed` aux row | partial diff (with errors); other detectors complete | Operator inspects log; re-run dream with `--detectors` filter |
| Invoker timeout (Anthropic API slow) | per-call timeout in detector | detector falls back to TF-IDF heuristic; proposal noted with `quality: degraded` | Operator checks API status |
| Mock-LLM regression (different scores between versions) | reproducible-id tests | CI catches | Preserve PRNG semantics |
| Diff retention exceeded | nightly cleanup script | old `dreams/<ts>/` directories pruned per `manifest.dreams.retention_days` | Adjust retention |
| Two dream runs racing | both acquire snapshot, both emit independent diffs | both diffs coexist; apply of either is independent | None — by design |
| ULID collision | 128-bit space; statistically impossible | n/a | n/a |
| Proposal_id collision within one dream | secrets-driven; collision in 8-char base32 is rare | runner re-generates on collision | None |
| Apply attempts to write to a no-longer-existent path | writer raises FileNotFoundError | transactional rollback | Re-run dream |
| AGENTS.md §7.7 amendment present but malformed | invariants walker | `cyberos doctor` warns | Operator fixes wording |
| Dream produces zero proposals on clean store | runs to completion; diff has empty proposals[] | `dream.complete` row's `proposals_count: 0`; success | None — by design |
| Patterns detector with < 5 episodes per task fingerprint | threshold not met; no proposal | none emitted | None — by design |
| Verify detector fires on every memory if no corrections seen | scope filter + age filter | bounded set; verify proposals add only `last_verified_at` | None — semantically cheap |
| Operator runs dream during cron run of another dream | snapshot isolation lets both run; lock contention only at apply | both complete normally; apply order serialised | None |
| `dream.proposal_applied` row written before transaction commit | writer's aux-emit is inside transaction | transactional rollback unwinds it | None |
| `manifest.dreams.retention_days = 0` | jsonschema rejects (minimum=1) | `ManifestError` | Operator picks valid value |
| Dream interrupted (Ctrl-C) | partial diff file written to disk | `dream.complete` aux row missing | Operator deletes the partial dir; re-runs |

---

## §11 — Implementation notes

- **Why per-detector module under `cyberos/core/dream/detectors/`** — each detector is independently testable and replaceable. Slice-4 `entry_points` plug-in mechanism slots in here.
- **ULID implementation** — minimal in-file generator for portability; `python-ulid` package can be a soft dependency for production. CI uses the inline generator.
- **Proposal id generator uses `secrets`** — cryptographic randomness avoids `random` seeding issues across multi-process dream runs. 8 base32 chars = 40 bits; collision probability in a single dream is < 1 in 10^10 even at 1000 proposals.
- **The `extra` dict on `put`/`delete` rows is open-ended** — `dream_id`, `proposal_id`, `reason`, `merged_into`, etc. all live in one bag. Schema-wise the AuditRecord's `extra` is `object` with no required keys; specific origins (dream) impose their own required-key invariant.
- **The applier's `txn.set_origin(dream_id=...)` is the load-bearing primitive** — it tells the Writer that until the transaction commits, every emitted row gets `extra.dream_id` automatically. No risk of an applier forgetting to set it on one path.
- **The §7.7 anchor-check is anti-footgun** — a developer who installs this FR's code but hasn't APPROVED the protocol amendment shouldn't be able to silently emit non-compliant rows. The check turns "subtle protocol violation" into "explicit error at apply time".
- **Transcript-input deferred** — FR-MEMORY-119 ships the session transcript ledger. For FR-MEMORY-115 slice-3, dream consumes audit rows + memory bodies. When FR-MEMORY-119 lands, the patterns detector gains conversation context as a stretch source.
- **The bench / load test for `≤ 5 min wall-time anthropic` is operator-side** — needs a real API key + a representative fixture. CI runs the ≤ 60 s mock variant.
- **`txn.touch_meta(...)` is a writer convenience for the verify op** — updates frontmatter `meta.last_verified_at` without rewriting the body. Implementation: read body, parse frontmatter, replace `last_verified_at` field, write back as a new `put` row whose body_hash differs only in frontmatter.

---

*End of FR-MEMORY-115.*
