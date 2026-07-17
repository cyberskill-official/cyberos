---
task_id: TASK-IMP-122
audited: 2026-07-18
verdict: FAIL
score: 4/10
issues_open: 5
issues_resolved: 0
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean (exit 0). TRACE-001 passes STRUCTURALLY - every §1 clause is cited
  by an AC - so this spec dies on judgment, not mechanics.
auditor: independent subagent (reproduced the defect synthetically at /tmp/fpt) + author
  verification of the rules_sha claim against source
---

## §1 - Verdict summary

FAIL at 4/10. The defect is REAL - the auditor reproduced it: mutating one byte of a vendored file
with both VERSIONs equal still yields `verdict=up_to_date`. But the spec's stated REASON is false,
it is unaware of the mechanism living in the two files it proposes to modify, §1.2 is
unimplementable as written, and 5 of 7 ACs are weaker than their clause.

## §2 - Findings (ALL OPEN)

### ISS-001 (CRITICAL) - "Nothing compares content" is FALSE. Verified against source.
TASK-IMP-074 already ships a content fingerprint, in both modified files:
- `version.sh:57`  `# rules_sha - the rule-content fingerprint (TASK-IMP-074 §10)`
- `version.sh:89-93` `elif [ "$inst_sha" != "$pay_sha" ]; then verdict="rules_drift"`
- `update-check.sh:76` `# VERSION is a promise; rules_sha is the evidence. An unchanged VERSION
  shipping changed rules is the exact case the version compare above cannot see.`
- `build.sh:354` computes it.
`update-check.sh:76` states the spec's own thesis verbatim. The spec greps 0 hits for `074` and
0 for `rules_sha`, and nominates TASK-IMP-082 (a status-page renderer) as "the precedent this
reuses" while the true precedent is inside its own modified_files.

### ISS-002 (CRITICAL) - the TRUE defect, which the spec never states
`rules_sha` is computed at BUILD over the payload and COPIED into `.cyberos/manifest.yaml`.
Neither side ever re-hashes the installed tree. So it answers "did these two manifests come from
the same build?" - never "do these bytes still match?". Two manifests from one build always agree
no matter what happens to the files afterward. THAT is why the mutation reported `up_to_date`.
Second, independent, unmentioned defect: the cone is `find cuo plugin mcp cli memory` -
`docs-tools/` is EXCLUDED, and 4 of the 6 artefacts the spec cites as evidence
(batch-select.mjs, render-status-hub.mjs, verify-goals.mjs, workflow-improve.mjs) live there.
Consequence of the false framing: an implementer told "nothing compares content" builds a SECOND
fingerprint beside the broken one - which TASK-IMP-104 §1.2 explicitly forbids ("install MUST NOT
carry a second implementation"). The fix is to REPAIR rules_sha (recompute + widen cone).

### ISS-003 (CRITICAL) - §1.2 is unimplementable; its guardrail claims a protection the design cannot give
"The manifest covers exactly what `build.sh` vendors" presumes a declarative vendor list.
`build.sh` has none: 32 plain `cp`, 9 recursive `cp -R` whole-tree copies, 2 globs, 21 conditional
`[ -f ... ] && cp`. `VENDORED_SKILLS` lists skill NAMES, not paths. Five trees (plugin, ci, mcp,
cli, template) vendor with zero per-path enumeration - a new file vendors with NO build.sh edit,
so AC 6 ("adding a vendored path to build.sh") tests an artefact that does not exist.
Worse, a payload-derived manifest CANNOT catch a vendor-step omission: if build.sh silently skips
a file (`:198`'s `[ -f ... ] && cp` - exactly the `workflow-improve.mjs` ABSENT row), the file is
absent from the output, hence absent from the manifest, hence absent from the install -> manifest
matches -> reports current -> silently blessed. The guardrail is backwards.

### ISS-004 (MAJOR) - 5 of 7 ACs weaker than their clause (TASK-IMP-118 class)
- §1.1 "MUST emit a manifest LISTING EVERY VENDORED PATH WITH A CONTENT HASH" vs AC 1 "a built
  payload CONTAINS a manifest". Presence, not content. An empty manifest passes. Textbook 118.
- §1.2 "MUST cover EXACTLY" (bidirectional) vs AC 6 (tests only absent-from-manifest). A stale
  extra entry passes.
- §1.4 binds `version.sh` AND `update-check.sh`; no AC names which is exercised - the auto-running
  path (`update-check.sh`) can go untested. And "naming EACH differing path" is never tested: both
  ACs use exactly one differing path, so an implementation naming only the first passes.
- §1.5 "a BYTE-IDENTICAL pair reports current" vs AC 4 "a FRESHLY INSTALLED machine" - not the same
  proposition; a fresh install is not byte-identical (install generates gates.env, config.yaml,
  memory/store, .update-check-cache).
- §1.6 "report UNKNOWN" (an OUTPUT claim) vs AC 5 which explicitly FORBIDS asserting on output and
  substitutes an exit code the clause never mentions. The AC's hardening deletes the assertion its
  clause demands. "either side" and "or unreadable" both untested.

### ISS-005 (MAJOR) - AC 7 is false-by-construction against the real code
AC 7 asserts "the installed tree is byte-identical before and after a comparison run".
`update-check.sh:99` writes `printf '%s\n' "$now" > "$cache"` where `cache="$cy/.update-check-cache"`
- INSIDE `.cyberos/`, on every run. Also AC 5 requires exit non-zero on a missing manifest, but
`update-check.sh` is soft by default (`mode="${CYBEROS_UPDATE_CHECK:-soft}"`); a non-zero exit
would break every `.cyberos` invocation. Unreconciled.

### ISS-006 (MODERATE) - the measured baseline no longer reproduces
"Measured on this repo" is present-tense; all six artefacts are byte-identical at HEAD because
this session REPAIRED them (commit e2504cf3, after the d19362ad measurement). The Success Metrics
baseline "reported 'up to date' across six drifted artefacts" is unreproducible. Re-anchor to a
synthetic reproduction (the auditor built one in 3 commands) or mark the evidence historical.

### ISS-007 (MODERATE) - normative MUST NOTs sited in §3, outside TRACE-001's reach
Three BCP-14 clauses live in `## 3. Edge cases` where TRACE-001 (scoped to §1) cannot see them,
none with an AC. The load-bearing one: "gates.env, config.yaml, .update-check-cache, memory/store
MUST NOT appear in the manifest - or every install reports drift against itself immediately".
That is a correctness requirement with no §1 clause and no test. Whether deliberate or not, this
is how a spec dodges the refuse-to-score-10 rule.

## §3 - Verified accurate (credit)
- The 104-vs-122 distinction HOLDS. 104 §1.1-1.6 are entirely ordering, and 104 explicitly
  rejected content hashing: "Compare a manifest hash rather than a version. Rejected: overkill for
  a monotonic version line, and it answers 'different' rather than 'older'". 122's reasoning that
  widening 104 would re-open a passing task is correct and well-argued.
- The tamper-evidence disclaimer is adequate and internally consistent AS PROSE - but normatively
  inert (no §1 clause, no AC), so nothing stops an implementer printing "INTEGRITY CHECK FAILED".
- Alternatives Considered is genuinely strong (QA-005 passes comfortably).
- TRACE-003 passes: all 7 ACs cite `test_payload_fingerprint.sh`, correctly declared in new_files.

## §4 - Required before re-audit
Reframe from "nothing compares content" to "rules_sha is a stored build-identity token, never
recomputed over the installed tree, and its cone excludes docs-tools/". Then make the central
design decision the spec currently cannot even see: does 122 REPAIR rules_sha or REPLACE it?
Re-derive every AC against its clause's verb. Resolve §1.2 (either scope the declarative vendor
list IN, with honest effort, or drop the "exactly" guarantee). Reconcile AC 7 and AC 5 with
update-check.sh's cache write and soft-by-default contract. Re-anchor the baseline.
