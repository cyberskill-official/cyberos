# CHANGELOG — `cuo/cpo/fr-create`

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/).
> SemVer at the skill level: MAJOR breaks the input/output envelope
> (`expects:` / `produces:`) or the `fr-manifest@2` schema; MINOR adds
> backwards-compatible fields or new optional behaviour; PATCH is editorial
> or reference-doc clarification.

---

## v0.1.0 — 2026-05-05 (port from FR_CREATE_AND_AUDIT.md v2.0.0 — create half)

### Added

- `SKILL.md` — entry. Frontmatter per
  `cyberos/docs/skills/README.md` §3. Owns the create-half lifecycle:
  CONTRACT_ECHO → PLAN → WORKER → RESUME, halting at HITL gates and
  amendment batches.
- `PIPELINE.md` — three worked chain examples (`fr-create` → `fr-audit`,
  audit-only, and the future `fr-create` → `fr-audit` →
  `cuo/cto/fr-to-tech-spec`).
- `references/MANIFEST_SCHEMA.md` — `fr-manifest@2` schema, hashing rules
  (§3.1), re-entrancy invariants (§3.2), write discipline (§3.4),
  BATCH_COMPLETE format.
- `references/PLAN_RENDER.md` — the plan-approval block (§11).
- `references/HITL_PROTOCOL.md` — HITL_BATCH_REQUEST format and RESUME
  protocol (§7 + §6).
- `references/AMENDMENT_PROTOCOL.md` — amendment record schema, risk-class
  table, batch aggregation, inline-apply (§10.6, §10.7, §6.7).
- `references/EU_AI_ACT_DECISION_TREE.md` — Article 5 / Annex III /
  Article 50 decision tree (§8); shared with `fr-audit` (which reads it
  during QA-001/002/003 enforcement).
- `references/ANTI_FABRICATION.md` — what the skill MUST NEVER invent (§9).
- `references/UNTRUSTED_CONTENT.md` — `<untrusted_content>` wrapping rules
  + injection-marker scan (§12) + CaMeL convergence (DEC-050).
- `references/FAILURE_MODES.md` — BOOT-001..008, CONTRACT_DRIFT,
  INPUTS_CHANGED, EXHAUSTED, STALE (§14).

### Changed (vs FR_CREATE_AND_AUDIT.md v2.0.0)

- Audit loop **removed** from this skill. Audit now lives in
  `cuo/cpo/fr-audit` and is invoked via the LangGraph supervisor's edge
  (when chained) or by direct invocation (when standalone). The W4 step
  in the WORKER loop renamed from `INVOKE AUDIT` to `EMIT EVENT` — the
  skill emits a NATS subject `cuo.fr_create.fr_written` and lets the
  supervisor decide whether to chain.
- `prompt_revision: fr_create_and_audit@2.0.0` → `fr_create@2.0.0`.
- Manifest schema field `prompt_revisions.fr_create_and_audit` →
  `skill_revisions.fr_create`. v2.0.0 manifests load cleanly with a
  `MIGRATE_FORWARD` audit row written on first invocation.
- BOOT-006 retasked: was "audit-loop tool unavailable"; is now "the
  runtime cannot reach the chained `fr-audit` skill" — only matters when
  chaining is requested in the input envelope.
- Output envelope (`produces`) now declares
  `next_skill_recommendation: cuo/cpo/fr-audit` so the LangGraph
  conditional edge has a deterministic decision input.

### Removed

- The §15 audit rubric (moved to `cuo/cpo/fr-audit/RUBRIC.md`).
- The §16 audit loop algorithm (moved to `cuo/cpo/fr-audit/AUDIT_LOOP.md`).
- The §17 audit report format (moved to `cuo/cpo/fr-audit/REPORT_FORMAT.md`).
- The §18 embedded template (moved to
  `cuo/_shared/feature-request-template/template.md`; loaded at runtime
  rather than embedded inline).

### Backwards compatibility

A v2.0.0 manifest produced by the original FR_CREATE_AND_AUDIT.md prompt
loads cleanly under this skill. The first WORKER invocation against such
a manifest writes a `MIGRATE_FORWARD` audit row noting the skill split
and updates `skill_revisions` in place.

A v2.0.0 FR markdown is byte-identical in shape to a v0.1.0 FR markdown
(both reference `template: feature_request@1`), so already-PASS FRs do
NOT need regeneration.

### Acceptance evidence

- Source coverage 1157/1157 lines of FR_CREATE_AND_AUDIT.md v2.0.0 read
  in three sequential chunks during the port (per AGENTS.md §4.10).
- Round-trip pipeline test: PRD → fr-create → fr-audit → resume → pass,
  recorded in `cuo/cpo/fr-create/PIPELINE.md` §1.
- All references/*.md files trace 1:1 to the source §-headings, with the
  prefix renaming (s/fr_create_and_audit/fr_create/) the only systematic
  diff.

## How to add a future entry

Standard sub-sections:

- **Added** — new fields, new sections, new BOOT codes, new
  references/*.md docs.
- **Changed** — semantics changes that don't break the schema.
- **Removed** — fields/rules deprecated.
- **Backwards compatibility** — what manifests/FRs from prior versions
  still work, what migrates automatically.
- **Acceptance evidence** — pointer to the test artifact or run that
  validated the release.
