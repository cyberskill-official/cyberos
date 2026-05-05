# Failure modes — required handling (audit-side)

> Sourced from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §14, scoped to the audit-side codes only.

## Bootstrap-error format

```
BOOTSTRAP_FAILURE
code:        BOOT-NNN
phase:       AUDIT
skill:       cuo/cpo/fr-audit
inputs:      <whatever was attempted>
fr_paths:    [<paths attempted>]
reason:      <one sentence, no chain-of-thought>
remediation: <one sentence, copy-pasteable command>
```

| Code | Reason |
| --- | --- |
| BOOT-001 | An `fr_path` could not be read. Other paths in the batch still proceed; this code applies per FR. |
| BOOT-002 | An FR was not valid UTF-8 after extraction. |
| BOOT-003 | An existing audit report at `audit_path` was malformed; renamed to `<audit_path>.corrupt-<ts>` if runtime allows; ISS-000 record. |
| BOOT-004 | An existing audit report's `audit_template_version` is not `2.0`; CONTRACT_DRIFT — see below. |
| BOOT-006 | The runtime cannot execute the rubric (e.g., YAML parser missing, regex engine unavailable). The supervisor receives this and does NOT retry. |
| BOOT-007 | Mode dispatch ambiguous — `fr-audit` invoked with `requirements_files` set (those belong to `fr-create`). |

Do NOT write a partial audit report on bootstrap failure. Every BOOT failure appends one `genie.action_log` row with `row_kind: notify`.

## CONTRACT_DRIFT

```
CONTRACT_DRIFT
this_skill:                   cuo/cpo/fr-audit
this_skill_version:           0.1.0
this_prompt_revision:         fr_audit@2.0.0
this_audit_rubric_version:    audit_rubric@2.0
audit_report_template_version: <observed value from audit_path frontmatter>
audit_report_prompt_revision: <observed value>
remediation: Either re-run with a skill version matching the report, OR
             migrate the report forward by re-invoking under the current
             skill version (the audit writes a MIGRATE_FORWARD audit row
             before advancing). The template (loaded from
             cyberos/docs/contracts/feature-request/v1/ via
             depends_on_contracts:) and the rubric (RUBRIC.md) advance
             lockstep with the prompt_revision.
```

The skill body is the single source of truth for `prompt_revision` and `audit_rubric_version`. Reports from a future skill version cannot be loaded by an older skill — the older skill emits CONTRACT_DRIFT and refuses.

## EXHAUSTED termination block

Same as `fr-create`'s; reproduced here so audit-only invocations don't have to cross-reference the create-side.

```
LOOP_EXHAUSTED
audit_path:          <path>
audited_file:        <fr_path>
reason:              max_iterations | no_progress
audit_iteration:     <int>
last_open_rule_ids:  [<list>]
recommendation:      Inspect <audit_path>; resolve listed rule_ids manually;
                     either edit the FR and re-run, or accept the audit
                     final state with overall_status: fail.
```
