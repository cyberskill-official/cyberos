---
id: TASK-IMP-129
title: Uninstall MUST preserve .cyberos/config.yaml, the override home
template: task@1
type: improvement
module: improvement
status: draft
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-20T00:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-126, TASK-IMP-121, TASK-IMP-122, TASK-CUO-207, TASK-IMP-095]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-20
memory_chain_hash: null
effort_hours: 3
service: tools/install
new_files:
  - (none)
modified_files:
  - tools/install/uninstall.sh
  - tools/install/tests/test_install_hygiene.sh
source_pages:
  - "tools/install/install.sh:326 (on regeneration tells the operator 'durable overrides belong in .cyberos/config.yaml')"
  - "tools/install/install.sh:328 (scaffolds .cyberos/config.yaml exactly once, TASK-CUO-207 §1 #3; never clobber)"
  - ".cyberos/cuo/gates/run-gates.sh:24-25 (reads .cyberos/config.yaml as the override layer)"
  - ".cyberos/cuo/gates/run-gates.sh:75 ('Set commands in .cyberos/config.yaml (gates.build/lint/test/coverage)')"
  - "tools/install/uninstall.sh (zero occurrences of config.yaml - grep -c = 0; removed with .cyberos/)"
  - "tools/install/uninstall.sh:4 (preserve list keeps the BRAIN store under CYBEROS_UNINSTALL_KEEP_BRAIN; config.yaml is not in the kept set)"
  - "measured 2026-07-20: uninstall+install on this repo regenerated gates.env with empty TEST_CMD (ecosystem 'unknown'), reverting the same day's non-vacuous-gates configuration"
source_decisions:
  - "2026-07-20 Stephen: PLAN gate - author as three separate tasks (build / CI / config durability); approved."
  - "2026-07-20 authoring: reframed from 'uninstall destroys gates.env' after reading install.sh:324-326 - gates.env is machine-owned by design (TASK-CUO-207) and its regeneration is correct; the defect is that config.yaml, the file the system names as the durable alternative, is itself destroyed."
---

# TASK-IMP-129: Uninstall MUST preserve .cyberos/config.yaml, the override home

## Summary

Three shipped surfaces tell the operator that durable overrides belong in `.cyberos/config.yaml`, and `install.sh` scaffolds it exactly once and never clobbers it. `uninstall.sh` then removes it with the rest of `.cyberos/`. The one file the system promises is durable does not survive the operation most likely to be followed by a reinstall.

## Problem

`gates.env` is machine-owned by design. `install.sh:299` regenerates it on every install from autodetection, and `install.sh:326` says so plainly - when regeneration changes the file it points the operator at the durable alternative: "durable overrides belong in `.cyberos/config.yaml`". That design is coherent and this task does not disturb it.

The durable alternative is not durable. Three surfaces name `config.yaml` as the override home:

- `install.sh:326` directs the operator to it when gates.env is regenerated.
- `install.sh:328` scaffolds it exactly once, explicitly never clobbering (TASK-CUO-207 §1 #3) - a never-clobber promise only means something if the file persists.
- `run-gates.sh:24-25` reads it as the override layer, and `run-gates.sh:75` instructs the operator to "Set commands in `.cyberos/config.yaml` (gates.build/lint/test/coverage)".

`uninstall.sh` contains zero occurrences of `config.yaml`. It removes `.cyberos/` wholesale, keeping only the BRAIN store under `CYBEROS_UNINSTALL_KEEP_BRAIN`. An operator who follows the instruction printed by install and by the gate runner loses their overrides at the next uninstall, and the reinstall scaffolds a fresh commented-out file as though they had never configured anything.

This is the same principle TASK-IMP-126 §1.4 already established - "None of 1.1-1.3 may remove an operator file" - scoped there to `.mcp.json`, unmarked skill dirs, and foreign hook lines. `config.yaml` is operator content by construction: install writes it once and never again, so every subsequent byte in it is the operator's.

Observed on this repo on 2026-07-20: an uninstall/install cycle regenerated `gates.env` with an empty `TEST_CMD`, because autodetect reported ecosystem "unknown" for a repo whose suite is `bash scripts/tests/run_all.sh` rather than an npm script. That silently reverted a same-day fix that had made the machine gates non-vacuous, and the gates would have reported GREEN while running no tests. Node repos were unaffected - autodetect populated build, lint and test correctly there - so the failure is specific to repos whose test entrypoint autodetect cannot name, which is exactly the population that most needs a durable override.

## Proposed Solution

Add `config.yaml` to the set uninstall preserves, alongside the BRAIN store, and report it in the "kept on purpose" banner so the operator can see what survived. Separately, teach autodetect to recognise a shell test entrypoint so a bash-suite repo gets a populated `gates.env` rather than a silent empty one.

## Alternatives Considered

- Preserve `gates.env` instead. Rejected: it contradicts TASK-CUO-207, which makes gates.env machine-owned and regenerated on purpose. Preserving it would carry stale autodetect provenance across installs and make the regeneration message a lie.
- Move `config.yaml` outside `.cyberos/` so uninstall cannot reach it. Rejected: it is per-repo CyberOS configuration and belongs with the machine; the fix is for uninstall to honour the promise, not to relocate the file to route around uninstall.
- Document that operators should back up `config.yaml` before uninstalling. Rejected: a durability promise discharged by asking the operator to do it themselves is not a promise.
- Fix only autodetect and leave uninstall as-is. Rejected: autodetect will always have gaps, which is precisely why an override layer exists. The override layer must survive.

## Success Metrics

- Primary: an operator-edited `config.yaml` survives uninstall and its overrides are in effect after reinstall, with no manual restoration. Baseline today: it is deleted and reinstall scaffolds a fresh commented-out file.
- Guardrail: a repo whose only test entrypoint is a shell script gets a populated `TEST_CMD` from autodetect rather than an empty one, so the vacuous-gate outcome is not reachable by default.

## Scope

In scope: the uninstall preserve set and its banner, the autodetect rule for a shell test entrypoint, and arms in `test_install_hygiene.sh`.

### Out of scope / Non-Goals

- The machine-owned status of `gates.env` (TASK-CUO-207) - unchanged.
- The `config.yaml` schema or the set of overridable keys.
- Re-running install across the fleet after the fix (an operator-gated action).
- The BRAIN store's existing preservation behaviour and its `CYBEROS_UNINSTALL_KEEP_BRAIN` switch.

## Dependencies

None blocking. Extends the operator-file principle established by TASK-IMP-126 §1.4 to a file that task did not enumerate.

**Corroboration from TASK-IMP-122 §1.5.** That task, reasoning about the fingerprint cone rather than about uninstall, independently classifies `config.yaml` as install-generated-or-operator-owned and declares `exempt:config.yaml` so its contents never enter `rules_sha`. Two tasks reaching the same classification from unrelated directions is the strongest available evidence that the file is operator content; this task is the one that makes the classification survive an uninstall.

**Boundary against TASK-IMP-121 (done).** That task made uninstall leave the repo as it found it for `.agents` markers, `.gitignore`, the pre-commit hook, and the five native channel parents. It never reaches inside `.cyberos/`, because everything there was assumed to be machine-owned and disposable. `config.yaml` is the counterexample that assumption missed. This task does not revisit any surface 121 settled.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** the empty-TEST_CMD regression was observed directly during the 2026-07-20 estate sweep, on this repo. The initial framing - that uninstall wrongly destroys operator gate configuration in `gates.env` - was wrong and was corrected before drafting: reading `install.sh:324-326` showed gates.env is machine-owned by design, and the real defect is the destruction of `config.yaml`. Every source_pages line was read at HEAD; the "zero occurrences" claim is a grep count against `uninstall.sh`.
- **Human review:** scope and granularity approved at the 2026-07-20 PLAN gate, where the relationship to TASK-IMP-126 was surfaced before writing; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `uninstall.sh` MUST preserve `.cyberos/config.yaml` when it exists, and MUST report it in the "kept on purpose" banner alongside the other preserved paths.
- 1.2 After uninstall then install, an operator-set override in `config.yaml` MUST still be in effect - `run-gates.sh` MUST resolve the same gate commands it resolved before the cycle.
- 1.3 Gate-command autodetect MUST populate `TEST_CMD` for a repo whose test entrypoint is a shell script at a conventional path, rather than leaving it empty with ecosystem "unknown".
- 1.4 Preserving `config.yaml` MUST NOT keep any machine-owned file: `gates.env` MUST still be regenerated on install, and the rest of `.cyberos/` outside the documented preserve set MUST still be removed.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - a fixture writes an override into `config.yaml`, runs uninstall, and the file is present afterwards and named in the banner output - test: `tools/install/tests/test_install_hygiene.sh::t_config_yaml_preserved`
- [ ] AC 2 (traces_to: #1.2) - after uninstall then install, `run-gates.sh` resolves the gate command set by the operator's `config.yaml`, not the autodetected default - test: `tools/install/tests/test_install_hygiene.sh::t_overrides_survive_reinstall`
- [ ] AC 3 (traces_to: #1.3) - installing into a fixture whose only test entrypoint is a shell script produces a non-empty `TEST_CMD` - test: `tools/install/tests/test_install_hygiene.sh::t_autodetect_shell_suite`
- [ ] AC 4 (traces_to: #1.4) - after the same cycle, `gates.env` is a freshly regenerated file and no other machine-owned path under `.cyberos/` survived uninstall - test: `tools/install/tests/test_install_hygiene.sh::t_machine_files_still_removed`

## 3. Edge cases

- `config.yaml` absent at uninstall (never scaffolded, or deleted by the operator) MUST NOT fail the uninstall, and MUST NOT be reported as kept.
- A `config.yaml` left entirely at defaults (every line commented) is still preserved - uninstall cannot distinguish "unedited" from "deliberately reset to defaults", and guessing would reintroduce the silent-loss failure.
- After preservation, `.cyberos/` is non-empty when the machine is removed; the container-reclaim logic MUST report it kept rather than attempting to remove a non-empty directory.
- A repo with both an npm test script and a shell entrypoint MUST keep the existing precedence rather than switch to the shell one - 1.3 fills a gap and MUST NOT re-rank the cases autodetect already resolves.
- A shell entrypoint that exists but is not executable MUST still be detected; the gate invokes it via `bash`, so the executable bit is not the test.
- Security-class: uninstall reads and removes paths under the repo root. Adding a path to the preserve set narrows what is removed and grants no new capability; the preserved file is data read by `run-gates.sh`, which already treats it as untrusted configuration.
