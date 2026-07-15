---
id: TASK-MEMORY-116
title: "memory consolidate — semantic-dedup phase (Walk → Compact → Sign → Publish → SemanticDedup); shares duplicates detector with TASK-MEMORY-115; opt-in via --semantic-dedup; dry-run by default"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
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
related_tasks: [TASK-MEMORY-115, TASK-MEMORY-108]
depends_on: [TASK-MEMORY-115]
blocks: []

source_pages:
  - playground/extracts/agentic-memory.article.txt  # "Periodic consolidation" / `consolidate_memories()` reference
source_decisions:
  - DEC-220 (Reuse `cyberos.core.dream.detectors.duplicates` verbatim; semantic-dedup is the duplicates-only subset of dreaming gated to run inside the existing consolidation pipeline)
  - DEC-221 (Default = `--dry-run`; operator must explicitly pass `--apply` to merge proposals. Mirrors TASK-MEMORY-115's operator-review gate)
  - DEC-222 (No new audit kinds — semantic-dedup phase emits the same `dream.start` / `dream.complete` / `dream.proposal_applied` rows with `extra.invocation = "consolidate"` to tag origin)

language: python 3.10
service: modules/memory/cyberos/
new_files:
  - modules/memory/tests/test_consolidate_semantic_dedup.py
modified_files:
  - modules/memory/cyberos/core/consolidate.py    # add SemanticDedup phase after Publish; gated on --semantic-dedup flag
  - modules/memory/cyberos/__main__.py            # wire `--semantic-dedup` + `--apply` flags on `cyberos consolidate`
allowed_tools:
  - file_read: modules/memory/**
  - file_write: modules/memory/cyberos/core/consolidate.py, modules/memory/cyberos/__main__.py, modules/memory/tests/**
  - bash: cd modules/memory && python -m pytest tests/test_consolidate_semantic_dedup.py -v
  - bash: cd modules/memory && python -m cyberos consolidate --semantic-dedup --dry-run
disallowed_tools:
  - apply semantic-dedup proposals without explicit `--apply` flag (per §1 #2, DEC-221)
  - bypass the TASK-MEMORY-115 protocol-amendment §7.7 anchor check when `--apply` is used (the same anchor check applies)
  - introduce a new audit kind for this phase (per DEC-222 — reuse `dream.*`)

effort_hours: 6
subtasks:
  - "0.5h: cyberos/core/consolidate.py — add `semantic_dedup_phase()` that wraps `cyberos.core.dream.detectors.duplicates.run` + (if `--apply`) `cyberos.core.dream.applier.apply` with `extra.invocation = 'consolidate'`"
  - "0.5h: __main__.py — `consolidate --semantic-dedup [--apply] [--threshold 0.92]` flags"
  - "1.0h: ensure the existing 4-phase pipeline (Walk → Compact → Sign → Publish) runs to completion BEFORE SemanticDedup starts; if any earlier phase fails, SemanticDedup is skipped"
  - "1.0h: --threshold flag plumbing to the duplicates detector (default 0.92 per TASK-MEMORY-115 §1 #3)"
  - "1.5h: tests/test_consolidate_semantic_dedup.py — 10 cases (default-off, --semantic-dedup --dry-run produces diff, --semantic-dedup --apply requires §7.7, --apply with valid §7.7 advances chain, --threshold override, phase ordering, earlier-phase failure skips SemanticDedup)"
  - "0.5h: CHANGELOG entry + README/usage note"
  - "1.0h: integration test against the seeded_memory_with_dupes fixture from TASK-MEMORY-115 (validates the cross-task composition works end-to-end)"
risk_if_skipped: "TASK-MEMORY-116 is a strict subset of TASK-MEMORY-115's capabilities — it ships nothing TASK-MEMORY-115 doesn't already provide. The reason it's a distinct task is operator ergonomics: many operators run `cyberos consolidate` on a nightly cron and want dedup to happen there, not as a separate `cyberos dream` invocation. Without TASK-MEMORY-116, operators have to either (a) run two cron jobs (consolidate + dream) and live with the duplication of audit-chain work, or (b) skip dedup during routine maintenance and accept the duplicate accumulation. The 6-hour effort is dominantly tests + CLI glue — the semantic logic is already in TASK-MEMORY-115. Skipping means we'd later have to re-explain to operators that dedup lives in a separate command, which is a documentation tax. Cheaper to ship the convenience surface."
---

## §1 — Description (BCP-14 normative)

The semantic-dedup phase **MUST** be an optional addition to the existing four-phase consolidation pipeline (`Walk → Compact → Sign → Publish`), making the full pipeline `Walk → Compact → Sign → Publish → SemanticDedup`. The contract:

1. **MUST** be opt-in via `cyberos consolidate --semantic-dedup`. Default `cyberos consolidate` runs the four legacy phases ONLY (back-compat preserved exactly).
2. **MUST** default to `--dry-run` when `--semantic-dedup` is passed. Operator must explicitly add `--apply` to merge dedup proposals. Mirrors TASK-MEMORY-115's operator-review gate (DEC-221).
3. **MUST** reuse the `cyberos.core.dream.detectors.duplicates.run(...)` function and `cyberos.core.dream.applier.apply(...)` verbatim. No fork; no separate implementation. (DEC-220.)
4. **MUST** propagate the same `--threshold <float>` (default 0.92) that the duplicates detector accepts. The threshold is the cosine-sim cut for marking two memories as duplicates.
5. **MUST** tag the resulting `dream.start` / `dream.complete` / `dream.proposal_applied` audit rows with `extra.invocation: "consolidate"` so TASK-MEMORY-120's history view can distinguish dedup-from-consolidate from dedup-from-explicit-dream-run.
6. **MUST** skip the SemanticDedup phase if any of the four prior phases (Walk / Compact / Sign / Publish) failed. The pipeline emits the standard failure rows for the failed phase and exits non-zero; SemanticDedup never starts in that state.
7. **MUST** honour the same AGENTS.md §7.7 anchor check before `--apply` proceeds. The check is enforced inside `cyberos.core.dream.applier.apply()`; this task adds no new path that bypasses it.
8. **MUST** emit a structured summary to stdout at the end of the consolidation run: phase-by-phase pass/fail, semantic-dedup proposal count (if run), apply count (if applied).
9. **MUST** be idempotent — re-running `cyberos consolidate --semantic-dedup --apply` with identical store state produces zero new chain rows (same `dream apply` idempotency from TASK-MEMORY-115 #10).
10. **SHOULD** support `--threshold 0.95` for tighter dedup or `0.88` for looser — both within the cosine-sim sane range. Outside `[0.5, 0.99]` rejected at CLI parse time.

---

## §2 — Why this design (rationale for humans)

**Why subset, not separate path (§1 #3, DEC-220).** Two independent code paths drift. One is shared. The dream pipeline (TASK-MEMORY-115) is the canonical surface; semantic-dedup-in-consolidate is the convenience surface. Both call the same detector + applier.

**Why dry-run by default (§1 #2, DEC-221).** `cyberos consolidate` runs nightly in cron environments. Auto-applying dedup proposals in cron without a human in the loop is the bad-night scenario: a buggy detector + 1000 silent merges = corrupted memory store. Dry-run-by-default means cron-cron-runs produce inspectable diffs that the operator reviews before the next morning's apply.

**Why `extra.invocation` instead of a new audit kind (DEC-222).** New audit kinds expand the schema surface; new code paths in walkers / readers / TASK-MEMORY-120 history. `extra.invocation` is one bag-key, leaves walkers untouched, and lets the history view filter by invocation source.

**Why threshold knob (§1 #4, §1 #10).** Different scopes need different thresholds. `memories/refinements/` benefits from aggressive dedup (0.88) — the goal is one canonical pattern per insight. `memories/decisions/` should be conservative (0.95) — decisions that look semantically similar may have crucial wording differences. Operator picks per invocation.

**Why phase ordering matters (§1 #6).** The Sign / Publish phases update the audit-chain tip. If SemanticDedup ran first and added rows, then Sign / Publish would have to incorporate those — invites circularity. Running SemanticDedup AFTER Publish means the chain tip is stable when dedup operates, and dedup's own writes form a new tail.

---

## §3 — API contract

### CLI

```
cyberos consolidate \
    [--semantic-dedup [--apply] [--threshold <float>] [--scope <path>]] \
    [--quiet]
```

### Pipeline integration

```python
# modules/memory/cyberos/core/consolidate.py — additive diff
async def consolidate(
    writer: Writer,
    *,
    semantic_dedup: bool = False,
    apply_dedup:    bool = False,
    dedup_threshold: float = 0.92,
    dedup_scope:    str   = "",
) -> ConsolidationResult:
    # ─── Existing four phases (unchanged) ───
    walk_result    = await _walk_phase(writer)
    compact_result = await _compact_phase(writer)
    sign_result    = await _sign_phase(writer)
    publish_result = await _publish_phase(writer)

    if not all([walk_result.ok, compact_result.ok, sign_result.ok, publish_result.ok]):
        return ConsolidationResult(
            walk=walk_result, compact=compact_result, sign=sign_result, publish=publish_result,
            semantic_dedup=None,
        )

    # ─── Optional SemanticDedup phase ───
    dedup_result = None
    if semantic_dedup:
        from cyberos.core.dream.detectors import duplicates
        from cyberos.core.dream.applier  import apply as dream_apply
        from datetime import timedelta

        proposals = await duplicates.run(
            writer, since=timedelta(days=365), scope=dedup_scope,
            invoker_name=None, threshold=dedup_threshold,
        )
        if apply_dedup and proposals:
            diff = _proposals_to_diff(proposals, scope=dedup_scope, invocation="consolidate")
            apply_result = dream_apply(writer, diff)
            dedup_result = DedupPhaseResult(
                proposals_count=len(proposals),
                applied_count=apply_result["applied_count"],
                dry_run=False,
            )
        else:
            dedup_result = DedupPhaseResult(
                proposals_count=len(proposals),
                applied_count=0,
                dry_run=True,
            )

    return ConsolidationResult(
        walk=walk_result, compact=compact_result, sign=sign_result, publish=publish_result,
        semantic_dedup=dedup_result,
    )
```

---

## §4 — Acceptance criteria

1. **Default consolidate unchanged** — `cyberos consolidate` (no flag) runs only the 4 legacy phases; `dream.start` not emitted. *(traces_to: §1 #1)*
2. **--semantic-dedup --dry-run produces diff only** — `cyberos consolidate --semantic-dedup` (no --apply) on seeded fixture → diff file at `dreams/<ts>/diff.json` AND zero applied rows. *(traces_to: §1 #2)*
3. **--apply requires --semantic-dedup** — `cyberos consolidate --apply` (no --semantic-dedup) → CLI parse error. *(traces_to: §1 #2)*
4. **--semantic-dedup --apply applies proposals** — same fixture + §7.7 present → applied count > 0; chain advances. *(traces_to: §1 #2, §1 #7)*
5. **--apply requires §7.7 anchor** — fixture without §7.7 in AGENTS.md → `--apply` raises with structured message. *(traces_to: §1 #7)*
6. **Audit rows tagged with invocation=consolidate** — every `dream.*` row's `payload.extra` (or top-level field, per writer convention) has `invocation: "consolidate"`. *(traces_to: §1 #5)*
7. **Threshold override** — `--semantic-dedup --threshold 0.95` produces fewer proposals than default 0.92 on same fixture. *(traces_to: §1 #4, §1 #10)*
8. **Threshold out-of-range rejected** — `--threshold 0.4` → CLI parse error; `--threshold 1.1` → CLI parse error. *(traces_to: §1 #10)*
9. **Phase ordering — failure aborts SemanticDedup** — inject a Walk-phase failure → consolidation returns ConsolidationResult with `walk.ok=False` and `semantic_dedup=None`. *(traces_to: §1 #6)*
10. **Phase ordering — Sign before Dedup** — instrument both phases; SemanticDedup observes the post-Publish chain tip (not pre-Sign). *(traces_to: §1 #6)*
11. **Re-apply idempotent** — apply once, then apply again on unchanged state → second apply emits zero new rows. *(traces_to: §1 #9)*
12. **Summary stdout** — final stdout line is JSON-parseable summary with `walk_ok`, `compact_ok`, `sign_ok`, `publish_ok`, `dedup_proposals_count`, `dedup_applied_count`. *(traces_to: §1 #8)*
13. **Reuses TASK-MEMORY-115 detector** — code-level test asserts the import path `cyberos.core.dream.detectors.duplicates.run` is invoked from consolidate.py; no duplicate detector code exists in consolidate.py. *(traces_to: §1 #3, DEC-220)*
14. **Cross-task integration smoke** — `seeded_memory_with_dupes` fixture (TASK-MEMORY-115) → `cyberos consolidate --semantic-dedup --dry-run` produces the SAME diff (same proposal_ids) as `cyberos dream --detectors duplicates --dry-run`. *(traces_to: §1 #3)*

---

## §5 — Verification

```python
# modules/memory/tests/test_consolidate_semantic_dedup.py
import asyncio, json, pytest
from cyberos.core.consolidate import consolidate
from cyberos.core.writer      import Writer


@pytest.mark.asyncio
async def test_default_consolidate_unchanged(seeded_memory):
    """AC #1"""
    res = await consolidate(seeded_memory)
    assert res.semantic_dedup is None


@pytest.mark.asyncio
async def test_semantic_dedup_dry_run_produces_diff(seeded_memory_with_dupes):
    """AC #2"""
    head_before = seeded_memory_with_dupes.head_seq()
    res = await consolidate(seeded_memory_with_dupes, semantic_dedup=True, apply_dedup=False)
    assert res.semantic_dedup is not None
    assert res.semantic_dedup.proposals_count > 0
    assert res.semantic_dedup.applied_count == 0
    assert res.semantic_dedup.dry_run is True
    # No additional `put`/`delete` rows from the dedup pass
    # (only `dream.start` + `dream.complete` from TASK-MEMORY-115's runner)
    diff_files = list((seeded_memory_with_dupes.store_path / "dreams").rglob("diff.json"))
    assert len(diff_files) >= 1


@pytest.mark.asyncio
async def test_apply_requires_section_7_7(seeded_memory_with_dupes):
    """AC #5"""
    (seeded_memory_with_dupes.store_path / "AGENTS.md").write_text("# no section 7.7")
    with pytest.raises(RuntimeError, match=r"APPROVE protocol change P19 §7\.7"):
        await consolidate(seeded_memory_with_dupes, semantic_dedup=True, apply_dedup=True)


@pytest.mark.asyncio
async def test_apply_with_section_7_7(seeded_memory_with_dupes, ensure_section_7_7):
    """AC #4"""
    head_before = seeded_memory_with_dupes.head_seq()
    res = await consolidate(seeded_memory_with_dupes, semantic_dedup=True, apply_dedup=True)
    assert res.semantic_dedup.applied_count > 0
    assert seeded_memory_with_dupes.head_seq() > head_before


@pytest.mark.asyncio
async def test_audit_rows_tagged_invocation_consolidate(seeded_memory_with_dupes,
                                                        ensure_section_7_7):
    """AC #6"""
    head_before = seeded_memory_with_dupes.head_seq()
    await consolidate(seeded_memory_with_dupes, semantic_dedup=True, apply_dedup=True)
    for seq in range(head_before + 1, seeded_memory_with_dupes.head_seq() + 1):
        row = seeded_memory_with_dupes.read_audit_row(seq)
        if row["kind"].startswith("dream."):
            assert row.get("extra", {}).get("invocation") == "consolidate"


@pytest.mark.asyncio
async def test_threshold_override(seeded_memory_with_dupes):
    """AC #7"""
    res_default = await consolidate(seeded_memory_with_dupes, semantic_dedup=True)
    res_tight   = await consolidate(seeded_memory_with_dupes, semantic_dedup=True,
                                     dedup_threshold=0.95)
    assert res_tight.semantic_dedup.proposals_count <= res_default.semantic_dedup.proposals_count


@pytest.mark.asyncio
async def test_phase_failure_aborts_dedup(seeded_memory_with_dupes, monkeypatch):
    """AC #9"""
    from cyberos.core import consolidate as mod
    async def failing_walk(writer): return type("R", (), {"ok": False})()
    monkeypatch.setattr(mod, "_walk_phase", failing_walk)
    res = await consolidate(seeded_memory_with_dupes, semantic_dedup=True)
    assert res.semantic_dedup is None


@pytest.mark.asyncio
async def test_reapply_idempotent(seeded_memory_with_dupes, ensure_section_7_7):
    """AC #11"""
    await consolidate(seeded_memory_with_dupes, semantic_dedup=True, apply_dedup=True)
    head_after_first = seeded_memory_with_dupes.head_seq()
    await consolidate(seeded_memory_with_dupes, semantic_dedup=True, apply_dedup=True)
    # No new rows from re-apply because preconditions already match the new state
    # The 4-phase pipeline does run, but no new dream apply rows
    # (assertion checks only the dream.* rows, not Walk/Compact/Sign/Publish noise)
    new_dream_rows = [
        seq for seq in range(head_after_first + 1, seeded_memory_with_dupes.head_seq() + 1)
        if seeded_memory_with_dupes.read_audit_row(seq)["kind"].startswith("dream.proposal_applied")
    ]
    assert new_dream_rows == []


@pytest.mark.asyncio
async def test_summary_stdout_json(seeded_memory_with_dupes, capsys):
    """AC #12"""
    res = await consolidate(seeded_memory_with_dupes, semantic_dedup=True)
    captured = capsys.readouterr().out
    # The CLI wrapper prints a JSON line; consolidate() returns the struct
    payload = res.to_summary_dict()
    for k in ("walk_ok", "compact_ok", "sign_ok", "publish_ok",
              "dedup_proposals_count", "dedup_applied_count"):
        assert k in payload


@pytest.mark.asyncio
async def test_diff_matches_dream_run(seeded_memory_with_dupes):
    """AC #14 — same proposals from consolidate as from direct dream run"""
    from cyberos.core.dream.runner import run as dream_run
    from datetime import timedelta
    dream_diff = await dream_run(seeded_memory_with_dupes, since=timedelta(days=365),
                                  detector_names=("duplicates",), invoker_name="mock")
    consol = await consolidate(seeded_memory_with_dupes, semantic_dedup=True)
    # IDs are seeded; both runs produce identical proposal sets given identical inputs
    consol_props = consol.semantic_dedup_proposals  # accessor on ConsolidationResult
    assert {p.proposal_id for p in dream_diff.proposals} == {p.proposal_id for p in consol_props}
```

---

## §6 — Implementation skeleton

Skeleton above is the implementation. Order:

1. `cyberos/core/consolidate.py` — extend with `semantic_dedup_phase`.
2. `__main__.py` — CLI flags + parse validation.
3. Tests.
4. CHANGELOG.

---

## §7 — Dependencies

- **TASK-MEMORY-115 (depends on)** — `duplicates.run` + `dream_apply` + AGENTS.md §7.7. This task is a CLI wrapper around them.
- **TASK-MEMORY-108 (transitively)** — semantic backend (sentence-transformers) is the substrate for cosine sim.

---

## §8 — Example payloads

### Summary line on success

```text
$ cyberos consolidate --semantic-dedup --apply
[walk] ok (142 memories, 311 rows)
[compact] ok (0 segments archived)
[sign] ok (sth d4127a3b... → e5238c4d...)
[publish] ok
[semantic_dedup] 4 proposals, 4 applied (threshold 0.92, scope /)
{"walk_ok":true,"compact_ok":true,"sign_ok":true,"publish_ok":true,"dedup_proposals_count":4,"dedup_applied_count":4}
```

### Summary on dry-run

```text
$ cyberos consolidate --semantic-dedup
[walk] ok
[compact] ok
[sign] ok
[publish] ok
[semantic_dedup] 4 proposals, 0 applied (dry-run; pass --apply to merge)
Diff: dreams/20260519T123010Z/diff.json
{"walk_ok":true,"compact_ok":true,"sign_ok":true,"publish_ok":true,"dedup_proposals_count":4,"dedup_applied_count":0,"dry_run":true}
```

---

## §9 — Open questions

All resolved. Deferred:
- Per-tenant threshold override (KYC tenants want tighter dedup) — slice 4+ once TASK-MEMORY-117 lands.
- Daily scheduled apply (auto-apply for low-risk verify proposals only) — slice 4+ design discussion.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| `--apply` without `--semantic-dedup` | argparse mutually-required | CLI rejects | Operator adds `--semantic-dedup` |
| §7.7 missing | applier check | run aborts before any writes | Operator runs APPROVE chat-turn |
| Earlier phase fails | pipeline aborts dedup | dedup_result is None | Operator fixes underlying issue and re-runs |
| Threshold out of range | argparse `0.5 ≤ x ≤ 0.99` | CLI rejects | Operator picks valid value |
| Duplicates detector errors | TASK-MEMORY-115's error path | dream.detector_failed row + partial diff | Operator inspects + re-runs |
| Apply transaction fails mid-way | TASK-MEMORY-115's transactional rollback | head unchanged | Re-run after fixing |
| Re-apply on changed state | TASK-MEMORY-115 PreconditionFailed | exception, no writes | Re-run dedup detector + re-apply |
| Concurrent consolidate runs | `.lock` serialises | second blocks until first completes | None — by design |
| Mock invoker drift | unit tests assert proposal id stability | CI catches | Author preserves PRNG |

---

## §11 — Implementation notes

- **`--apply` is the operator's "I trust the proposals" signal.** Resist the temptation to add an `--auto-apply-merges` shortcut; that drifts back into the auto-apply anti-pattern.
- **`extra.invocation` tagging** — happens via `txn.set_origin(dream_id=..., invocation="consolidate")` from `cyberos.core.dream.applier`. This task adds the `invocation` kwarg to `set_origin`.
- **Phase ordering matters** for the SemanticDedup audit-row chain.
- **`to_summary_dict()` is a thin accessor on `ConsolidationResult`** — keeps the CLI's stdout-JSON contract decoupled from the dataclass field names.

---

*End of TASK-MEMORY-116.*
