# CHANGELOG — `cuo/cpo/fr-audit`

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the skill level: MAJOR breaks the rubric (changes a rule_id, adds an error-severity rule, removes a rule) or breaks the audit-report file format. MINOR adds new warning-severity rules, new optional reference docs, or extends the output envelope additively. PATCH is editorial.

---

## v0.2.0 — 2026-05-06 (registry v0.2.0 contract expansion)

### Added

- Frontmatter blocks per registry README v0.2.0 Part 2:
  - `invocation_modes: [standalone, chained]`.
  - `expects.optional_fields` (`rubric_version`, `upstream_context`, `trace_id`) + `expects.standalone_interview_ref`.
  - `produces.human_summary_ref` for chat-rendered audit summaries.
  - `exposable_as` — the rubric makes a strong MCP-tool surface; flag set `mcp_tool: true` (others gated).
  - `self_audit:` block with `deterministic_drift` as a sev-0 anomaly signal — fr-audit's reproducibility contract is the highest-value invariant.
  - `human_fine_tune:` block with rubric-specific review gates (`on_rubric_rule_added: true`, `on_rubric_rule_removed: true`).
  - `depends_on_contracts:` pinning `feature-request@v1` at the new contract location.
- `STANDALONE_INTERVIEW.md` — entry script for chat-mode invocations.
- `HUMAN_SUMMARY.md` — chat output template after each audit batch.
- `INVARIANTS.md` — INV-001 through INV-008 self-audit invariants (verdict determinism, rubric coverage, precise needs_human, citation completeness, no-false-pass-on-STALE, confidence reporting, no-rubric-drift-mid-batch, scope discipline).

### Changed

- All references to `cuo/_shared/feature-request-template/` updated to `cyberos/docs/contracts/feature-request/v1/template.md` (declared via `depends_on_contracts:`). Affected: §"CONTRACT_ECHO" line, the Citations §"Template under audit" entry. Body of the audit rubric itself (`RUBRIC.md`) gets a one-line path update per its own CHANGELOG entry below.
- `audit.row_kind` and `produces.output_kind` extended: `self_refinement_proposal` and `refinement_proposal` respectively.

### Driver

Same as `fr-create` v0.2.0 — registry v0.2.0 (DEC-090..093). fr-audit gets a slightly tighter `self_audit` block because its reproducibility contract is harder than fr-create's (auditor verdicts are deterministic by design; drift = critical).

### Backwards compatibility

- `audit_rubric@2.0` rule IDs and severities are **unchanged**. Existing `*.audit.md` reports remain valid.
- Output envelope shape is **unchanged** — every v0.1.0 field remains; v0.2.0 additions are under new top-level keys.
- The contract path move is read-side only; no FR's audit verdict changes as a result of this version bump.

---

## v0.1.0 — 2026-05-05 (port from FR_CREATE_AND_AUDIT.md v2.0.0 — audit half)

### Added

- `SKILL.md` — entry. Owns the audit-half lifecycle: CONTRACT_ECHO → per-FR audit loop → AUDIT_BATCH_SUMMARY, halting on needs_human via HITL_BATCH_REQUEST.
- `RUBRIC.md` — `audit_rubric@2.0` — full rule catalog: FM-001..111, SEC-001..009, COND-001..004, QA-001..009 + QA-TODO, SAFE-001..004, STALE-001 (cross-skill).
- `AUDIT_LOOP.md` — the 8-step audit algorithm (§16): locate → hash → load-or-init → run-rubric → attempt-fix → re-audit → terminate → write.
- `REPORT_FORMAT.md` — audit report frontmatter + per-issue block format (§17). Determinism contract: byte-stable for a given `audited_file_sha256` modulo `last_audit_at`.
- `PIPELINE.md` — input/output flows for chained-from-`fr-create`, standalone, and supervisor-classify-act-node interaction. Three worked chain examples.
- `references/UNTRUSTED_CONTENT.md` — copy of `fr-create`'s; shared contract enforced at both ends of the pipeline.
- `references/ANTI_FABRICATION.md` — copy of `fr-create`'s; QA-007 and QA-008 enforce.
- `references/HITL_PROTOCOL.md` — HITL_BATCH_REQUEST format + RESUME protocol; same as `fr-create`'s but rule_id values originate here.
- `references/EU_AI_ACT_DECISION_TREE.md` — copy of `fr-create`'s; QA-001/002/003 enforce.
- `references/FAILURE_MODES.md` — BOOT codes specific to audit-side failures (BOOT-001/002/003/004/006/007).

### Changed (vs FR_CREATE_AND_AUDIT.md v2.0.0)

- `prompt_revision: fr_create_and_audit@2.0.0` → `fr_audit@2.0.0`.
- BOOT-006 retasked: was "the runtime cannot execute the §15–§17 audit loop"; is now "the runtime cannot execute the rubric (e.g., YAML parser missing)".
- BOOT-007 inverted: was "both `requirements_files` and `fr_paths` set"; is now "this skill invoked with `requirements_files` set" (only `fr-create` accepts requirements files).
- STALE-001 (was §15.7 mode-A-only) is now `§15.7 cross-skill`. Active when input envelope carries `upstream_context.from_skill == cuo/cpo/fr-create`; skipped otherwise.
- Audit-report frontmatter adds three optional fields: `upstream_skill`, `upstream_manifest`, `genie_action_log_row_id`. Audit reports from v2.0.0 load cleanly (the new fields are absent; audit considers them `null`).

### Removed

- The §0–§14 + §18 create-half (moved to `cuo/cpo/fr-create/`).
- The §10.6/§10.7 amendment protocol (moved to `cuo/cpo/fr-create/references/AMENDMENT_PROTOCOL.md`).
- The `fr-manifest@2` schema (moved to `cuo/cpo/fr-create/references/MANIFEST_SCHEMA.md`).
- Mode dispatch §0.1 (the unified prompt's two modes are now two separate skills; dispatch happens at the supervisor level via `next_skill_recommendation`).

### Backwards compatibility

Audit reports produced under the unified v2.0.0 prompt load cleanly under this skill — `audit_template_version: 2.0` is unchanged, and the unified prompt's reports never set the new optional fields. A first re-audit under v0.1.0 fills `genie_action_log_row_id` from the new audit run; older reports may have `null` for that field forever (audit-row continuity from before the split is reconstructible from prior audit-runs in `genie.action_log`).

### Acceptance evidence

- Source coverage 1157/1157 lines of FR_CREATE_AND_AUDIT.md v2.0.0 read (per AGENTS.md §4.10) during the joint port with `fr-create`.
- Round-trip pipeline test: `fr-create` PASS → `fr-audit` PASS → AUDIT_BATCH_SUMMARY emitted, recorded in `PIPELINE.md` Example A.
- HITL round-trip test: `fr-audit` needs_human → human answer → re-audit PASS, recorded in `PIPELINE.md` Example B.
- All `RUBRIC.md` rule IDs are byte-identical to source v2.0.0 §15; rubric version `audit_rubric@2.0` preserved.

## How to add a future entry

Standard sub-sections:

- **Added** — new rules (warning-severity additions are MINOR; new error rules are MAJOR), new BOOT codes, new reference docs.
- **Changed** — rule semantics changes that don't change the rule_id.
- **Removed** — rule deprecations (always MAJOR).
- **Backwards compatibility** — what audit reports from prior versions still load, what migrates automatically.
