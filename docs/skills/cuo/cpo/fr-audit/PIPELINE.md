# Pipeline — `fr-audit` inputs and outputs

> `fr-audit` is the audit half of the original v2.0.0 monolith, lifted
> into its own atomic skill. This file shows how it slots into chains.

## Chain in: from `fr-create` (the default)

When the supervisor's LangGraph routes `fr-create`'s output through the
conditional edge `next_skill_recommendation == "cuo/cpo/fr-audit"`,
`fr-audit` receives an input envelope like:

```json
{
  "fr_paths": ["./feature-requests/FR-001-foo.md"],
  "caller_persona": "cuo-cpo",
  "trace_id": "<inherited from fr-create>",
  "upstream_context": {
    "from_skill": "cuo/cpo/fr-create",
    "manifest_path": "./feature-requests/manifest.json"
  }
}
```

The presence of `upstream_context` enables three behaviours:

1. **STALE-001 enforcement.** The audit can compare each FR's on-disk
   hash against `fr-create`'s manifest `frs[FR].fr_hash` and raise
   STALE-001 if they diverge. (Skipped when standalone.)
2. **Manifest write-back.** On termination, the audit writes
   `audit_hash` and `audit_iteration_count` back into
   `manifest.frs[FR]` so `fr-create`'s next WORKER cycle has accurate
   state.
3. **Trace-id continuity.** The same `trace_id` flows through both
   skills' `genie.action_log` rows, letting auditors reconstruct a
   single FR's lifecycle from one query.

## Chain in: standalone

```json
{
  "fr_paths": ["./team-a/FR-001-something.md"],
  "caller_persona": "cuo-cpo",
  "trace_id": "<fresh uuid>"
}
```

No `upstream_context`. STALE-001 is skipped. The audit emits its report
sibling to each input file. CI integrations call this form when they
want a "lint" pass over a directory of FRs.

## Chain out: into the supervisor's classify_act node

`fr-audit`'s output envelope:

```json
{
  "skill_id": "cuo/cpo/fr-audit",
  "skill_version": "0.1.0",
  "audit_rubric_version": "audit_rubric@2.0",
  "total_frs": 1,
  "overall_status_counts": {"pass": 0, "needs_human": 1, "fail": 0},
  "exit_code": 1,
  "per_fr": [
    {
      "fr_path": "./feature-requests/FR-001-foo.md",
      "audit_path": "./feature-requests/FR-001-foo.audit.md",
      "status": "needs_human",
      "iterations": 3,
      "audited_file_sha256": "<hex>"
    }
  ],
  "hitl_required": true,
  "requires_regen": false,
  "next_skill_recommendation": ""
}
```

The supervisor reads this envelope and:

- If `hitl_required: true`: parse the per-FR HITL_BATCH_REQUEST that
  follows, surface it via the Question primitive (SRS §6.6.2), pause
  the LangGraph at the `interrupt()` node.
- If `overall_status_counts.fail > 0` AND `iterations >= max`: emit a
  Notify (SRS §6.6.1) — the FR is EXHAUSTED, human inspection needed.
- If `overall_status_counts.pass == total_frs`: continue to the next
  skill in the chain (or terminate). When chained from `fr-create`,
  this typically means returning control to `fr-create`'s next WORKER
  iteration so the next FR can be claimed.

## Chain examples

### Example A — full pipeline, all-pass

```
fr-create (W1-W5, FR-001)
  → fr-audit (FR-001 → pass)
    → fr-create (W1-W5, FR-002)
      → fr-audit (FR-002 → pass)
        → fr-create (W1-W5, FR-003)
          → fr-audit (FR-003 → pass)
            → fr-create BATCH_COMPLETE
              → supervisor terminates chain
```

`genie.action_log`: 6 rows (3 × `artefact_write` for FRs, 3 × `artefact_write`
for audits), all sharing the same `trace_id`.

### Example B — full pipeline, one needs-human

```
fr-create (W1-W5, FR-001)
  → fr-audit (FR-001 → needs_human, ISS-003 = QA-007)
    → supervisor surfaces HITL_BATCH_REQUEST, pauses chain
... (human answers FR-001/ISS-003: A; provides source) ...
fr-create RESUME (FR-001 issue applied)
  → fr-audit (FR-001 re-run → pass)
    → fr-create (W1-W5, FR-002)
      ... continues normally ...
```

`genie.action_log`: 1 × `question` (the HITL_BATCH_REQUEST), then the
chain resumes producing further `artefact_write` rows.

### Example C — standalone audit-only (no fr-create)

```
external CI tool → fr-audit ([./team-a/*.md])
  → AUDIT_BATCH_SUMMARY back to CI tool
  → CI exits with the rubric's exit_code (0/1/2)
```

`genie.action_log`: N rows (one per audited FR's report write), no
`question` rows unless any FR went `needs_human`.

## Failure-mode interaction with the chain

If `fr-audit` emits `BOOT-006` (rubric runtime unavailable), the
supervisor:

- Marks the affected FRs `status = ERRORED` in `fr-create`'s manifest.
- Emits a Notify (`row_kind: notify`) explaining what failed.
- Does NOT auto-retry — `fr-audit` is the rubric authority; if it can't
  run, no other skill can substitute.
