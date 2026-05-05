# HITL_BATCH_REQUEST format + RESUME protocol

> Sourced verbatim from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0
> §7 + §6.

## HITL_BATCH_REQUEST format

Emitted as the LAST thing in the response when at least one FR has
`status = HITL_PAUSE`. Maps to the CyberOS Question primitive
(SRS §6.6.2).

```
HITL_BATCH_REQUEST
====================
manifest_path:                 <output_dir>/manifest.json
requirements_hash:             <hex>
plan_approval_hash:            <hex>
paused_frs:                    <int>
total_blocking_issues:         <int>
hitl_categories_present:       [<one or more from CONTRACT_ECHO hitl_categories>]

[FR-NNN-<slug>]  status: HITL_PAUSE  iteration: <int>/<max>

  ISS-NNN (<category>) [rule_id: <FM/SEC/COND/QA-…>]
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

When all <int> issues are answered, re-invoke fr-create.
The skill will resume from manifest state. Issue IDs and option letters
WILL NOT be re-issued.
END_HITL_BATCH_REQUEST
```

Each emission appends one row to `genie.action_log` with `row_kind:
question` (per SRS §6.6.2 + §6.7). The row's `payload_hash_field` is the
SHA-256 of the canonical-JSON serialisation of the issue list.

## RESUME protocol

### 6.1 Detect

Phase is `RESUME` when at least one FR has `status = HITL_PAUSE` and all
of its `blocking_issues[].resolution` are non-null AFTER parsing the
human's reply.

### 6.2 Answer parsing

Reply lines are of the form `FR-NNN/ISS-NNN: <letter>` or
`AMD-NNN: <letter>` with optional indented payload. One line per
issue/amendment. Strict matching against issue/amendment IDs in the
manifest.

### 6.3 Application

For each resolved issue:

- `ISS-NNN`: update `frs[FR].blocking_issues[i].resolution` and
  `resolved_at`. The FR's chained `fr-audit` re-runs with the answer
  injected.
- `AMD-NNN`: branch per `AMENDMENT_PROTOCOL.md` §6.7.

### 6.4 Aggregation on resume

If multiple FRs were paused, the worker resumes each one's audit before
claiming new FRs from the backlog.

### 6.5 Malformed answers

If a reply line doesn't match the expected format, halt with a
clarification request — do NOT guess.

### 6.6 Never-re-ask invariant

The skill MUST NEVER re-ask a HITL question whose `resolution` is
non-null. Re-asking is a contract violation surfaced as a drift event in
OBS (SRS §6.12).
