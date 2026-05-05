# Audit report file format

> Sourced verbatim from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §17.
> The audit report is the artefact that other CyberOS modules (PROJ, OBS,
> CP) read to understand FR conformance.

Each audit report is markdown with YAML frontmatter and one issue per H2
block. Field names are `snake_case`. Issue IDs are stable: once `ISS-007`
is assigned, it is never reused for a different problem in this file.

## Frontmatter

```yaml
---
audit_template_version: 2.0
audited_file: ./<fr-path>
audited_file_sha256: <64-char lowercase hex>
template: feature_request@1
audit_iteration_count: <integer ≥ 1>
first_audit_at: <ISO 8601 UTC>
last_audit_at: <ISO 8601 UTC>
overall_status: pass | fail | needs_human
counts:
  by_severity: {error: <int>, warning: <int>, info: <int>}
  by_status:   {open: <int>, fixed: <int>, wontfix: <int>, needs_human: <int>}
prompt_revision: fr_audit@2.0.0   # MUST match the literal in cuo/cpo/fr-audit/SKILL.md CONTRACT_ECHO
audit_rubric_version: audit_rubric@2.0

# v2.0+ optional fields:
amendment_acknowledgement: <amendment_id | null>
upstream_skill: cuo/cpo/fr-create | <other> | null
upstream_manifest: <path | null>
genie_action_log_row_id: <evt_… | null>  # the audit-row UUID emitted for this report
---
```

## Issue block

```markdown
## ISS-<NNN> — <one-line title>

- severity: error | warning | info
- category: frontmatter | section | quality | safety | bootstrap | mode
- rule_id: <e.g. FM-109>
- location: <"frontmatter:eu_ai_act_risk_class" | "section:## Success Metrics" | "line:42" | "block:untrusted_content#1">
- status: open | fixed | wontfix | needs_human
- first_seen_iteration: <int>
- last_seen_iteration: <int>
- resolved_at: <ISO, only if status=fixed or wontfix>
- hitl_reason: <one of CONTRACT_ECHO hitl_categories, only if status=needs_human>

### Description
<2–6 sentences. Cite the rule_id.>

### Suggested fix
<Concrete diff-style instruction or copy-pasteable replacement. If status=needs_human, leave empty and fill the HITL question.>

### HITL question
<Only when status=needs_human. Single-paragraph statement of what the human must decide. See references/HITL_PROTOCOL.md.>

### Resolution note
<Only when status=fixed or wontfix. ≤2 sentences.>
```

## Determinism property

Two runs of `fr-audit` against the same `audited_file_sha256` (= same
input FR bytes) produce byte-identical reports modulo `last_audit_at`
(which is the only time-dependent field). This is the determinism
contract from `SKILL.md` frontmatter (`determinism.reproducible: true`).

The `last_audit_at` field is excluded from the
`payload_hash_field: audited_file_sha256` audit-row hash precisely so
that re-runs produce the same chain hash — only the inputs that affect
verdict participate in the hash.
