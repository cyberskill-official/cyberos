---
id: TASK-IMP-104
title: Refuse an older payload over a newer .cyberos
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-103]
blocks: []
related_tasks: [TASK-IMP-095, TASK-IMP-096]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 2
service: tools/install
new_files:
  - tools/install/tests/test_install_version_guard.sh
modified_files:
  - tools/install/install.sh
source_pages:
  - "IMPROVEMENT_HANDOFF.md §10 IMP-24"
  - "tools/install/install.sh:21 (avail_ver read), :41 (printed), no comparison anywhere; version.sh + lib/update-check.sh already carry a comparator (verified on main bb231900)"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-104: Refuse an older payload over a newer .cyberos

## Summary

`install.sh` reads the payload's VERSION, prints it, and never compares it against the `.cyberos/VERSION` already in the repo - the vendor step is an unconditional overwrite. An older payload therefore downgrades a consumer silently: skills disappear, doctrine reverts, and the only trace is a line nobody re-reads. Compare before vendoring, refuse a downgrade naming both versions, and allow the operator through with an explicit override.

## Problem

`install.sh:21` sets `avail_ver` from the payload's VERSION and `:41` prints it. No line compares it with the installed `.cyberos/VERSION`. Meanwhile `version.sh` and `lib/update-check.sh` already contain a version comparator - install simply never asks them.

Nothing catches this downstream: workflow-version pins live in the payload being installed, so a downgraded machine reports the older pins as correct. The failure is silent by construction. Today there is exactly one version in the world, which is why it has never bitten - and exactly why it is cheap to fix before 1.0.0 puts a second one out there.

## Proposed Solution

Before the vendor step, read `.cyberos/VERSION` when present and compare it against `avail_ver` using the comparator that already exists - reused, not rewritten. Equal proceeds silently (re-vendor is the documented idempotent path). Newer proceeds. Older refuses with a non-zero exit naming installed version, payload version, and the override. `CYBEROS_ALLOW_DOWNGRADE=1` proceeds with a warning that records both versions, because an operator deliberately pinning an older machine is a real case and the workflow should not lie to them about it.

## Alternatives Considered

- Warn but proceed. Rejected: the defect is silence, and a warning on a step that continues is silence with extra text. The whole point is that a downgrade must be a decision.
- Refuse with no override. Rejected: rolling back a bad release is a legitimate operator action, and a gate with no key gets bypassed by `rm -rf .cyberos` - which loses the operator's config.
- Compare a manifest hash rather than a version. Rejected: overkill for a monotonic version line, and it answers "different" rather than "older", which is not the question.

## Success Metrics

- Primary: installing an older payload over a newer `.cyberos/` exits non-zero with both versions named, and the installed machine is untouched - suite-asserted. Baseline: today it overwrites silently.
- Guardrail: same-version re-install stays silent and idempotent (the documented path), and `CYBEROS_ALLOW_DOWNGRADE=1` completes with both versions recorded in the summary.

## Scope

In scope: the pre-vendor comparison in `install.sh`, the override, the summary line, suite arms.

### Out of scope / Non-Goals

- Any migration of consumer data on upgrade or downgrade - this refuses; it does not transform.
- Rewriting the version comparator (reuse `version.sh`'s).
- Auto-fetching the correct payload on refusal - install vendors what it was handed.

## Dependencies

depends_on TASK-IMP-103: both guard the vendor step in `install.sh`, and the order is normative - the version check MUST run before the lock is acquired, so a refused downgrade never takes a lock it will not use. Shipping 104 first would force 103 to re-open the same lines. Per TASK-IMP-101's depends_on evidence gate, 103's coverage-gate artefact is the evidence.

`version.sh`'s comparator already exists on main.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md IMP-24, verified against install.sh on merged main; implementation under ship-tasks supervision.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 Before the first destructive vendor operation AND before the TASK-IMP-103 lock is acquired, `install.sh` MUST compare `avail_ver` against `.cyberos/VERSION` when that file exists and is parseable. A refused downgrade MUST NOT acquire the lock.
- 1.2 The comparison MUST reuse the existing comparator (`version.sh` / `lib/update-check.sh`); install MUST NOT carry a second implementation.
- 1.3 Payload version older than installed MUST refuse with a non-zero exit naming the installed version, the payload version, and the `CYBEROS_ALLOW_DOWNGRADE=1` override. It MUST NOT vendor.
- 1.4 `CYBEROS_ALLOW_DOWNGRADE=1` MUST proceed and MUST record both versions in the install summary, so the downgrade is legible after the fact.
- 1.5 Equal versions MUST proceed with no new output (idempotent re-vendor is the documented path and MUST NOT become noisy).
- 1.6 An absent, empty, or unparseable `.cyberos/VERSION` MUST proceed, naming the condition - a missing version is a first install or a damaged machine, and neither is a downgrade.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.3) - older payload over newer installed exits non-zero, names both versions and the override, and leaves `.cyberos/` byte-identical - test: `tools/install/tests/test_install_version_guard.sh::t01_downgrade_refused`
- [ ] AC 2 (traces_to: #1.4) - with the override set, the same install completes and the summary records both versions - test: `tools/install/tests/test_install_version_guard.sh::t02_override_records_both`
- [ ] AC 3 (traces_to: #1.5) - same-version re-install completes with no added output vs today - test: `tools/install/tests/test_install_version_guard.sh::t03_equal_is_silent`
- [ ] AC 4 (traces_to: #1.6) - absent and unparseable VERSION both proceed with the condition named - test: `tools/install/tests/test_install_version_guard.sh::t04_missing_version_proceeds`
- [ ] AC 5 (traces_to: #1.2) - no second comparator: install delegates to the existing one - verify: recorded grep in the gate log showing install.sh calls the shared comparator and defines no version-compare function of its own (structural claim; same rationale as TASK-IMP-090 AC 1).

## 3. Edge cases

- Pre-release / suffixed versions (`1.0.0-rc1` vs `1.0.0`): whatever the existing comparator already decides, and the suite pins that behavior rather than inventing a second opinion.
- `.cyberos/VERSION` newer than any released payload (a developer's local build): refuses, and the override is the documented path. Correct - install cannot know it is a dev build.
- Damaged `.cyberos/` where VERSION is absent but the machine is present: proceeds per 1.6 and re-vendors, which is the repair path.
- A payload with no VERSION file at all: `avail_ver` is already `unknown` on main; unknown MUST NOT be treated as older (it is not comparable), so it proceeds with the condition named.
- Security-class: version strings are read from files inside the repo and the payload, are compared, and are never executed or interpolated into a command. No execution surface.
