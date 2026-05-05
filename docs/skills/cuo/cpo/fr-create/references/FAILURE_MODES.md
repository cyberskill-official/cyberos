# Failure modes — required handling

> Sourced verbatim from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §14.

## 14.1 Bootstrap-error format

```
BOOTSTRAP_FAILURE
code:        BOOT-NNN
phase:       PLAN | RESUME | WORKER
skill:       cuo/cpo/fr-create
inputs:      <whatever was attempted>
manifest:    <path attempted>
reason:      <one sentence, no chain-of-thought>
remediation: <one sentence, copy-pasteable command>
```

| Code | Reason |
| --- | --- |
| BOOT-001 | A required input file was not found (a `requirements_files` entry). |
| BOOT-002 | An input file was not valid UTF-8 after extraction. |
| BOOT-003 | `manifest.json` exists but JSON parse failed. |
| BOOT-004 | `manifest.json` schema version is not `fr-manifest@2`. |
| BOOT-005 | `output_dir` does not exist and could not be created. |
| BOOT-006 | The runtime cannot reach the chained `fr-audit` skill (only matters when chaining is requested in the input envelope). |
| BOOT-007 | Mode dispatch ambiguous — `fr-create` invoked with `fr_paths` set (those belong to `fr-audit`). |
| BOOT-008 | (reserved — formerly "template_path missing"; obsolete since v0.2.0, the template loads via `depends_on_contracts:` from `cyberos/docs/contracts/feature-request/v1/template.md`). |

Do NOT write a partial manifest on bootstrap failure. Every BOOT failure appends one `genie.action_log` row with `row_kind: notify` (the user is notified, no action taken).

## 14.2 EXHAUSTED termination block

```
LOOP_EXHAUSTED
manifest_path:       <path>
exhausted_fr:        FR-NNN
audited_file:        <path>
reason:              max_iterations | no_progress
audit_iteration:     <int>
last_open_rule_ids:  [<list>]
recommendation:      Inspect <audit_path>; resolve listed rule_ids manually;
                     either edit the FR and re-run, or mark fr.status = ERRORED
                     in the manifest.
```

## 14.3 CONTRACT_DRIFT

```
CONTRACT_DRIFT
this_skill:                   cuo/cpo/fr-create
this_skill_version:           0.1.0
this_prompt_revision:         fr_create@2.0.0
manifest_skill_revision:      <observed value from manifest.skill_revisions.fr_create>
caller_phase_implied:         <PLAN | WORKER | RESUME>
remediation: Either re-run with a skill version matching the manifest, OR
             migrate the manifest forward by re-invoking under the current
             skill version (the WORKER writes a MIGRATE_FORWARD audit row
             before advancing).
             The template (loaded from cyberos/docs/contracts/feature-request/v1/
             via depends_on_contracts:) and the audit rubric (in
             cuo/cpo/fr-audit/RUBRIC.md) advance
             lockstep with the prompt_revision — there is no cross-file
             matrix to consult.
```

The skill body is the single source of truth for `prompt_revision`, `template_version`, and chained skill versions.

## 14.4 INPUTS_CHANGED

```
INPUTS_CHANGED
previous_requirements_hash:  <hex>
current_requirements_hash:   <hex>
plan_status:                 was APPROVED, now INVALIDATED
remediation: Either revert the requirements files (manifest will resume) or
             re-run in PLAN phase. Existing PASS FRs are NOT auto-invalidated.
```

## 14.5 STALE_OVERWRITE

Surfaces as an HITL issue with category `stale_fr_disposition` and rule_id `STALE-001`. See `HITL_PROTOCOL.md`.
