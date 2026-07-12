---
fr_id: FR-IMP-069
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# FR-IMP-069 audit

## §1 - Verdict summary

Audited for spec correctness and for scope honesty against the two adjacent distribution FRs (FR-PLUGIN-008, FR-SKILL-201). Draft under-specified asset naming for `latest/download`, raced release creation, and assumed GNU tar everywhere. All resolved; traceability closes over t01-t09 in tools/cyberos-init/tests/test_release_assets.sh.

## §2 - Findings (all resolved)

### ISS-001 latest/download needs stable asset names
GitHub's latest-download URL requires a constant filename; versioned-only names broke the one canonical URL the whole FR exists to provide. Resolved: §1 #1 stable aliases + versioned twins, AC 2.

### ISS-002 upload raced release creation
`gh release upload` fails when the release object does not exist yet (installer jobs create it concurrently). Resolved: §6 create-or-upload idempotent step (`gh release create --verify-tag || true` before upload).

### ISS-003 determinism flags are GNU-only
`tar --sort/--mtime` and `gzip -n` false-fail on macOS bsdtar. Resolved: §11 pins CI to ubuntu and requires a visible SKIP (not a false pass or fail) for the determinism case on non-GNU hosts.

### ISS-004 supersession ambiguity with the marketplace FRs
Per the operator's "newest wins" conflict rule, the audit checked whether this FR displaces FR-PLUGIN-008/FR-SKILL-201: it does not (different systems). Resolved: source_decisions boundary statement + §2/§7 non-supersession wording, so no old FR is closed by mistake.

### ISS-005 checksums covered only the tarballs
The .plugin assets were outside SHA256SUMS in the first cut, leaving the desktop-install path unverifiable. Resolved: §1 #1 "covering all four", AC 3 round-trip.

### ISS-006 untestable network paths
bootstrap/rollout ACs had no offline test strategy (TRACE-002 risk). Resolved: file:// fixtures specified in §5 (t06-t08), keeping the suite hermetic.

## §3 - Resolution

All six findings addressed as cited; dependency direction (068 -> 069 -> 070) is acyclic and stated on both sides. **Score = 10/10.**

*End of FR-IMP-069 audit.*

## §10 - Post-implementation gates (2026-07-12, ship run)

- §10.4 coverage gate: PASS - t01-t09 green on fresh rerun; FR-IMP-068 (10/10) and FR-SKILL-116
  (6/6) suites green as regression. Report: .workflow/FR-IMP-069/coverage-and-review.md.
- TRACE-004 closure: PASS. awh/caf: N/A (declared); floor = bash -n + 25 green cases.
- HITL gate 1: APPROVED by Stephen Cheng 2026-07-12. HITL gate 2: ACCEPTED same date via explicit
  operator pre-authorization at the review gate; gates stayed green.
- First live proof arrives with the next v* tag (payload job publishes the assets).

*FR-IMP-069 shipped 2026-07-12.*
