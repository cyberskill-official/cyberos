---
id: NFR-CUO-005
title: "CUO persona-version stamping — every chain MUST record persona+workflow version"
module: CUO
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of audit rows carry persona_slug, persona_version, workflow_slug, workflow_version"
owner: CTO
created: 2026-05-18
related_frs: [FR-CUO-101, FR-CUO-103]
---

## §1 — Statement (BCP-14 normative)

1. Every CUO chain audit row **MUST** carry `{persona_slug, persona_version, workflow_slug, workflow_version}` — both slug and version, both for persona and workflow.
2. Slug values **MUST** be the full `chief-<role>-officer` (or canonical short form like `chief-of-staff`, `chief-architect`) — never short acronyms.
3. Version values **MUST** come from the workflow's frontmatter `version:` field (SemVer). If missing, the chain refuses to execute.
4. A workflow whose `version:` field changes **MUST NOT** retroactively change historical audit rows — they preserve the version at execution time.
5. The combination `(persona_slug, workflow_slug, workflow_version)` **MUST** be sufficient to reproduce the exact chain ran — replay relies on this.

## §2 — Why this constraint

Persona+workflow are versioned artifacts that evolve. Without stamping the version onto each audit row, post-hoc questions like "what did the CTO's architect-new-system workflow look like in March?" are unanswerable. The full-slug rule prevents the slug-mapping drift after the persona normalisation rename. The "refuses without version" gate forces workflow authors to keep versioning hygiene. Replay reproducibility is the binding correctness guarantee — a missing version means audit history is decoupled from artifact history.

## §3 — Measurement

- Counter `cuo_audit_missing_version_total` — must always be 0.
- Audit-row schema validator counts rows lacking any of the 4 keys.
- Quarterly replay test — pick 10 historical chains; assert each replays to the same skill_chain (proves version stamping works).

## §4 — Verification

- CI gate (T) — every workflow markdown has `version:` field; missing field fails the build.
- Unit test (T) — chain execution emits row containing all 4 fields.
- Replay test (T) — pick row; reconstruct + assert.

## §5 — Failure handling

- Workflow without `version:` → catalog scan fails → CI blocks.
- Audit row missing any of 4 fields → sev-2; chain row schema broken; halt CUO writes until fixed.
- Replay returns different chain than recorded → sev-1; version stamping has a bug; investigate immediately.

---

*End of NFR-CUO-005.*
