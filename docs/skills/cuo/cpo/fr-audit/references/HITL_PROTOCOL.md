# HITL_BATCH_REQUEST format + RESUME (audit-side)

> Same format as `cuo/cpo/fr-create/references/HITL_PROTOCOL.md`. The
> audit-side rule_ids (FM-NNN, SEC-NNN, COND-NNN, QA-NNN, SAFE-NNN,
> STALE-001) originate in `RUBRIC.md` and surface here when the audit
> halts. Sourced from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §7.

## When fr-audit emits HITL_BATCH_REQUEST

After Step 7 (Termination check) of `AUDIT_LOOP.md` lands in branch (b)
HITL_PAUSE — i.e. at least one issue has `status = needs_human`. The
emission happens AFTER the audit report has been written (Step 8) and
AFTER the AUDIT_BATCH_SUMMARY (so summary stays first, then the
human-actionable block).

## Format

```
HITL_BATCH_REQUEST
====================
audit_paths:                   [<audit_path1>, <audit_path2>, ...]
total_paused_frs:              <int>
total_blocking_issues:         <int>
hitl_categories_present:       [<one or more from CONTRACT_ECHO hitl_categories>]

[FR-NNN-<slug>]  audit_path: <path>  iteration: <int>/<max>

  ISS-NNN (<category>) [rule_id: <FM/SEC/COND/QA/SAFE-…>]
    Description (paraphrased — never raw untrusted text):
      <2–4 sentences>
    What was attempted:
      <1–2 sentences>
    Options:
      A) <option A label> — <consequence>
      B) <option B label> — <consequence>
      C) <option C label> — <consequence>

(repeat per issue, then per FR)

------------------------------------------------------------
How to answer
------------------------------------------------------------
Reply with ONE LINE per issue, using the exact format:

  FR-NNN/ISS-NNN: <letter>

Optionally followed by indented payload. Examples:

  FR-003/ISS-001: A
  FR-003/ISS-002: B; commit to FR-003.1 by 2026-06-15

When all <int> issues are answered, re-invoke fr-audit with the same
fr_paths. The audit will resume from each report's audited_file_sha256
state. Issue IDs and option letters WILL NOT be re-issued.
END_HITL_BATCH_REQUEST
```

When chained from `fr-create`, the supervisor merges this into
`fr-create`'s HITL_BATCH_REQUEST — the user sees one consolidated
human-action block per pipeline pause, not two separate ones per skill.

## RESUME contract

When the human answers, the next invocation:

1. Parses each line per `FR-NNN/ISS-NNN: <letter>[; <payload>]`.
2. Strict matches against issue IDs in the corresponding audit report's
   `## ISS-NNN` blocks.
3. Updates `audit.issues[i].resolution` and `audit.issues[i].resolved_at`.
4. Re-enters `AUDIT_LOOP.md` Step 4 for each affected FR.

The audit MUST NEVER re-ask a HITL question whose `resolution` is
non-null. Re-asking is a contract violation surfaced as a drift event
(SRS §6.12).

## Per-issue audit-row emission

Each line in the HITL_BATCH_REQUEST corresponds to one issue. When the
human's answer arrives, the audit appends one `genie.action_log` row
per resolved issue with `row_kind: act` (the audit performed an action
in response to the answer — auto-applied a fix, marked an issue
wontfix, etc.). The row's payload includes the answer string, the
applied resolution, and the new audit hash.
