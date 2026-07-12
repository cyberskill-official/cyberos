---
# ── Identity ─────────────────────────────────────────────────────────
name: architectural-spike-audit
description: >-
  Audit an architectural-spike@1 against architectural_spike_rubric@1.0: enforces structural completeness (SPK-STRUCT - frontmatter, five sections, recommendation names exactly one probed option), checkable evidence on every option (SPK-EVID - repo path, command+output, or URL; confidence cross-checked against evidence depth), timebox discipline (SPK-BOX - plan and actual recorded, >1.5x overrun carries a recorded operator HALT verdict), and a non-empty discard log when options were rejected (SPK-DISC). Emits a `score / 10` verdict; refuses to pass on <10/10. Use when user asks to "audit this architectural spike" or "check the architectural spike". Do NOT use for "draft a new architectural spike" (use architectural-spike-author instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: d
  cyberos-template: architectural-spike@1
  cyberos-rubric-target: architectural_spike_rubric@1.0

# ── Scope contract (memory/AGENTS.md §15) ────────────────────────────
allowed_memory_scopes:
  read:
    - project:*
    - module:*
  write:
    - project:fr/{fr_id}/architectural-spike-audit
audit:
  row_kind: architectural_spike_audited
  required_fields: [spike_id, fr_id, verdict, score, findings]

# ── Inputs / outputs ─────────────────────────────────────────────────
inputs:
  - { name: architectural_spike, format: architectural-spike@1, required: true }
outputs:
  - { name: spike_audit, format: "verdict: pass|fail|needs_human + score /10 + findings[]" }

# ── Triggers / blockers ──────────────────────────────────────────────
triggers:
  - an architectural-spike@1 artefact exists and has not passed audit
blockers:
  - "artefact is not architectural-spike@1 (unknown version) - needs_human, never guess"
---

# architectural-spike-audit

## 1. Purpose

Make spike verdicts reproducible: an auditor cites SPK rule ids instead of
paraphrasing prose. Only 10/10 passes; evidence is checked by RESOLUTION (does the
citation actually check out at audit time), not by presence.

## 2. Verdict semantics

pass = every rubric rule green (10/10). fail = any rule red, findings name each rule
id + location + what resolves it. needs_human = ambiguity the rubric cannot decide
(unknown artefact version, contradictory frontmatter, an operator-verdict question).

See RUBRIC.md for the rule families, AUDIT_LOOP.md for the iteration protocol, and
REPORT_FORMAT.md for the report shape.
