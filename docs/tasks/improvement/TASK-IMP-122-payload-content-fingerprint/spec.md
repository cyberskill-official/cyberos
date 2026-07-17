---
id: TASK-IMP-122
title: Payload staleness must be detectable
template: task@1
type: improvement
module: improvement
status: draft
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-18T04:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-104, TASK-IMP-082, TASK-IMP-068]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-18
memory_chain_hash: null
effort_hours: 6
service: tools/install
new_files:
  - tools/install/tests/test_payload_fingerprint.sh
modified_files:
  - tools/install/install.sh
  - tools/install/lib/update-check.sh
  - tools/install/version.sh
  - tools/install/build.sh
source_pages:
  - "2026-07-18 measured on this repo at HEAD d19362ad: .cyberos/VERSION == dist/cyberos/VERSION == 1.0.0 while 6 vendored artefacts differed"
  - "tools/install/install.sh:56-80 (TASK-IMP-104's downgrade guard - compares VERSION, fails closed when its comparator is missing)"
  - "modules/cuo/.../render-status-hub.mjs (TASK-IMP-082's fp- corpus content fingerprint - the precedent this reuses)"
source_decisions:
  - "2026-07-18 Stephen: PLAN gate - author as its own task, distinct from TASK-IMP-104 (which guards ordering, not content)."
---

# TASK-IMP-122: Payload staleness must be detectable

## Summary

An installed `.cyberos/` can differ arbitrarily from the payload it claims to be, and every check we own reports it current. Measured on this repo: `.cyberos/VERSION` and `dist/cyberos/VERSION` both read `1.0.0` while six vendored artefacts differed - including a `batch-select.mjs` whose known parallel-corruption bug had already been fixed and merged. Nothing compares content, so nothing could see it.

## Problem

`version.sh`, `lib/update-check.sh` and TASK-IMP-104's downgrade guard all compare `VERSION` strings. VERSION answers "which release is this", never "are these the bytes of that release". A payload that is stale but same-versioned is therefore indistinguishable from a current one.

Measured drift on this repo, VERSION identical throughout:

| artefact | installed state |
|---|---|
| `cuo/ship-tasks.md` | missing `## 11d. Batch economics` (TASK-IMP-114) |
| `docs-tools/batch-select.mjs` | pre-PR#53: no `declares()`, so an undeclared cone read as EMPTY and joined every batch |
| `docs-tools/render-status-hub.mjs` | knew 3 of 4 status-page inputs; its `fp-` stamp did not cover `docs/batches/` |
| `docs-tools/verify-goals.mjs` | drifted |
| `cuo/skills/workflow-improver` | ABSENT |
| `docs-tools/workflow-improve.mjs` | ABSENT |

The consequences were not theoretical. The stale `batch-select` returned `{106, INV-004, 115}` where the current one returns `{106, 115}` - it admitted a task with an undeclared cone into a parallel swarm batch, which is precisely the failure PR #53 was merged to prevent. The stale renderer produced a committed status page (`d19362ad`) whose `fp-` stamp omitted an input, which the current renderer's own comment names as "a stamp that lies about what determined the page".

TASK-IMP-104 already guards the ORDER of versions and works. This is a different axis: EQUAL version, different bytes. The fix shape is already proven in this repo - TASK-IMP-082 replaced a HEAD-sha stamp with an `fp-` content fingerprint for exactly this reason, on exactly this kind of question.

## Proposed Solution

`build.sh` emits a manifest of per-file hashes into the payload. `install.sh` vendors it. `version.sh` and `update-check.sh` compare the installed manifest against the payload's and report DRIFTED, naming the differing paths, rather than reporting current on a VERSION match alone. The manifest covers exactly what `build.sh` vendors, so a file the vendor list omits is absent from the manifest and cannot be silently blessed.

## Alternatives Considered

- Bump VERSION on every payload change. Rejected: it conflates release identity with build identity, forces a version bump for a comment fix, and still says nothing when a vendor step silently omits a file.
- Compare mtimes. Rejected: mtime is not content, survives no copy or checkout faithfully, and would have reported this exact drift as fine.
- Re-vendor unconditionally on every `.cyberos` use. Rejected: install is not free, and a guard that fixes drift without reporting it teaches nobody that the channel leaked.
- Fold into TASK-IMP-104. Rejected: 104 guards ordering and is correct as shipped; widening it to content would re-open a passing task to add an orthogonal axis.

## Success Metrics

- Primary: a payload/installed pair that differs in any vendored byte is reported DRIFTED with the differing paths named. Baseline: reported "up to date" across six drifted artefacts.
- Guardrail: a byte-identical pair reports current - no false drift, or the check gets ignored.
- Guardrail: the manifest covers every path `build.sh` vendors - a file added to the vendor list without the manifest is a build failure, not a silent gap.

## Scope

In scope: manifest emission in `build.sh`, vendoring in `install.sh`, comparison in `version.sh` and `lib/update-check.sh`, suite.

### Out of scope / Non-Goals

- Auto-repairing drift. Reporting is the deliverable; re-vendoring is the operator's call and `install.sh` already does it.
- TASK-IMP-104's ordering guard. Untouched.
- Signing or tamper-evidence. This detects staleness, not attack; a manifest beside the payload proves neither authorship nor integrity against an adversary, and MUST NOT be described as if it did.

## Dependencies

None. Deliberately independent of TASK-IMP-104: that guard runs before the lock and stays as-is, and this check is additive.

## AI Authorship Disclosure

- **Tools used:** Claude (Opus 4.8) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from measured drift on this repo at HEAD `d19362ad`, verified by running both the installed and source `batch-select.mjs` and diffing their output. Implementation under ship-tasks supervision.
- **Human review:** scope approved at the 2026-07-18 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `build.sh` MUST emit a manifest into the payload listing every vendored path with a content hash.
- 1.2 The manifest MUST cover exactly the set `build.sh` vendors: a vendored path absent from the manifest MUST fail the build.
- 1.3 `install.sh` MUST vendor the manifest into `.cyberos/`.
- 1.4 `version.sh` and `lib/update-check.sh` MUST compare the installed manifest against the payload's and report DRIFTED, naming each differing path, when any hash differs - INCLUDING when the two VERSION strings are equal.
- 1.5 A byte-identical installed/payload pair MUST report current.
- 1.6 A missing or unreadable manifest on either side MUST report UNKNOWN and MUST NOT report current. A check that cannot run is not a pass.
- 1.7 The comparison MUST NOT modify the installed machine.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.3) - a built payload contains a manifest, and installing it puts the manifest in `.cyberos/` - test: `tools/install/tests/test_payload_fingerprint.sh::t01_manifest_emitted_and_vendored`
- [ ] AC 2 (traces_to: #1.4) - mutating ONE byte of ONE vendored file in an installed `.cyberos/`, leaving both VERSION strings equal, is reported DRIFTED and the mutated path is named; the test MUST FAIL if the check reports current - test: `tools/install/tests/test_payload_fingerprint.sh::t02_equal_version_different_bytes_is_drift`
- [ ] AC 3 (traces_to: #1.4) - a file present in the payload and ABSENT from the installed machine is reported DRIFTED and named (the `workflow-improver` shape) - test: `tools/install/tests/test_payload_fingerprint.sh::t03_absent_file_is_drift`
- [ ] AC 4 (traces_to: #1.5) - a freshly installed machine reports current, not drift - test: `tools/install/tests/test_payload_fingerprint.sh::t04_fresh_install_reports_current`
- [ ] AC 5 (traces_to: #1.6) - a removed manifest reports UNKNOWN and exits non-zero; the assertion MUST be on the exit code, not on the printed text - test: `tools/install/tests/test_payload_fingerprint.sh::t05_missing_manifest_fails_closed`
- [ ] AC 6 (traces_to: #1.2) - adding a vendored path to `build.sh` without the manifest fails the build - test: `tools/install/tests/test_payload_fingerprint.sh::t06_vendor_list_and_manifest_agree`
- [ ] AC 7 (traces_to: #1.7) - the installed tree is byte-identical before and after a comparison run - test: `tools/install/tests/test_payload_fingerprint.sh::t07_check_is_read_only`

## 3. Edge cases

- Payload NEWER than installed, VERSION differs: TASK-IMP-104's ordering guard owns that call; this check reports drift and does not duplicate the refusal.
- Installed machine hand-edited by an operator: reported DRIFTED and named - correct. The check reports what differs, never who differed it, and MUST NOT accuse.
- `gates.env`, `config.yaml`, `.update-check-cache`, `memory/store/` are install-generated or operator-owned, not vendored: they MUST NOT appear in the manifest, or every install reports drift against itself immediately.
- A vendored file that is legitimately empty: hashed like any other; empty is a content, not an absence.
- First install (no installed manifest to compare): 1.6's UNKNOWN, not drift and not current.
- Security-class: the manifest is a staleness signal, not an integrity guarantee - anything that can rewrite a vendored file can rewrite the manifest beside it. The output MUST NOT be worded as tamper detection.
