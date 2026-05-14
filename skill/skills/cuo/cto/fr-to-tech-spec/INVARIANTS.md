# `fr-to-tech-spec` self-audit invariants (scaffold)

> Declarative truths the spec-author MUST enforce about its own behaviour at runtime. A breach emits a `refinement_proposal`, pauses the pipeline, and waits for human review. Distinct from the (future) `tech-spec-audit` skill's RUBRIC.md which validates SPECS this skill produces — this file is rules the skill applies to its own behaviour.
>
> **Scaffold-only at v0.1.0.** The runtime that enforces these doesn't exist yet (gated on registry v0.3.0 harness build). Each invariant is documented at the contract level so the future runtime has a precise specification.

## How invariants work

Same machinery as `cuo/cpo/fr-author/INVARIANTS.md` and `cuo/cpo/fr-audit/INVARIANTS.md`. ID + Statement + Check + Severity + Refinement template. Checked at every node boundary, every batch completion, and on demand.

## Invariants

### INV-001 — pass-verdict-only ingestion

**Statement.** This skill MUST refuse to author a tech spec from any FR whose sibling `*.audit.md` reports `overall_status != pass`. Specifically: `fail`, `needs_human`, `stale`, `exhausted` verdicts all halt the spec from being written for that FR.

**Check.** For each `fr_paths[i]`, locate the sibling audit report (either via `audit_paths[i]` if provided, or by computing `<fr_path with .md replaced by .audit.md>`). Parse the audit's frontmatter `overall_status`. If non-`pass`, refuse + emit `BOOT-007`.

**Severity.** `error`. This is the single most-important contract this skill carries: it is the seam between "audited FR" and "engineering work", and authoring specs from non-pass FRs would short-circuit the audit gate.

**Refinement template.**
```
trigger: INV-001 breach: spec authored from non-pass FR {fr_id}
observation: FR {fr_id} has overall_status={status}; spec was written anyway.
proposed_amendment_target: cyberos/docs/skills/cuo/cto/fr-to-tech-spec/SKILL.md
proposed_amendment_section: §"Failure modes" BOOT-007
proposed_diff: |
  +  Reject the FR earlier in the pipeline (PLAN phase, before WORKER).
  +  Surface the audit verdict to the user as the rejection reason.
minimum_viable: "Add the verdict-check to the PLAN phase; halt before any WORKER step runs."
```

### INV-002 — citation completeness

**Statement.** Every spec section that references an FR claim (a stated requirement, an acceptance criterion, an explicit constraint) MUST cite the FR's section name and line number. Vague "based on the FR" is rejected.

**Check.** Regex against the spec body — every `<!-- fr-cite -->` marker must be paired with `(FR-NNN, §"<section>" line N)`.

**Severity.** `error`.

### INV-003 — open-question discipline

**Statement.** Every spec MUST end with a `## Open questions` section. If the section is empty, the spec MUST explicitly state "No open questions — all requirements decompose deterministically from the FR." Empty section without the explicit statement = breach.

**Check.** Regex match.

**Severity.** `warning`. Soft signal — sometimes a spec really has no open questions. The explicit statement just forces the author to confirm.

### INV-004 — scope discipline

**Statement.** No `write_file` lands outside the `output_dir` declared in the input envelope. Tech specs are siblings of each other in `output_dir/`; no nested folders, no writes to the FR folder, no writes to BRAIN.

**Check.** Walk audit rows of `op:create` written by this skill; every `path` is under `output_dir`.

**Severity.** `error`.

### INV-005 — sizing-uncertainty escalation

**Statement.** If any work-package in the implementation plan has sizing `XL` (>3 weeks of one engineer's work) AND no `## Open questions` entry surfaces that as a risk, the spec is incomplete. Either down-scope (split into multiple specs), justify why XL is acceptable in the open-questions section, or escalate to human.

**Check.** Parse the implementation plan; count `XL` rows; cross-reference with open-questions section text.

**Severity.** `warning` — not all XL work is wrong, but it MUST be visible to reviewers.

### INV-006 — confidence-band reporting

**Statement.** The spec body's `## Architecture summary` and `## Implementation plan` sections each carry a `confidence:` field in the frontmatter. Architecture summary is typically high-confidence (the FR's components touched are usually clear); implementation plan is usually medium-confidence (sizing has more uncertainty).

**Check.** Frontmatter validation.

**Severity.** `info` — schema validation enforces presence + range; this invariant exists as documentation of the contract (mirrors fr-audit's INV-006 demotion in v0.2.2).

## Adding a new invariant

Same procedure as fr-author + fr-audit. The author + persona steward (cto) propose an invariant; the registry maintainer reviews; an acceptance test is added under `acceptance/`; the skill MINOR-bumps with the new invariant.

## Invariants vs. future tech-spec rubric

The (future) `cuo/cto/tech-spec-audit` skill will carry a `RUBRIC.md` that validates SPECS this skill produces. That rubric is for spec content; this file is for spec-author behaviour. The split mirrors fr-author + fr-audit's separation.

| `tech-spec-audit/RUBRIC.md` (future) | `fr-to-tech-spec/INVARIANTS.md` (this file) |
| --- | --- |
| What the future auditor checks **on each spec**. | What this skill checks **on its own behaviour while writing specs**. |
| Authored by `cto` + `cseco`. Bumps the spec rubric version. | Authored by `cto` + the registry maintainer. Bumps this skill's version. |
| User-visible — surfaced in spec-audit reports. | Operator-visible — surfaced as `refinement_proposal` to the supervisor. |
