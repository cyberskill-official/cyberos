---
fr_id: FR-PLUGIN-007
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

## §1 — Verdict summary

Multi-runtime adapters — 4 P1 targets (claude-code, cursor, cowork, codex-cli) each consuming the same canonical manifest, producing target-native bundles with per-target Sigstore signatures. 510 lines, 14 §1 clauses, 22 ACs, 5 test files, 15 failure modes, 10 implementation notes. 7 issues resolved (single canonical manifest eliminates per-target drift; per-adapter reproducibility makes Sigstore round-trip work; bridge bundling for non-claude-code targets removes install friction; Cursor skills-omission prevents bloat in a runtime that doesn't render them; target-format conformance avoids proprietary extension lock-in; pack-time targets[] cross-check catches manifest bugs early; per-target tests embed target-specific expectations CI can enforce). **Score = 10/10.**

## §2 — Findings (all resolved)

### ISS-001 — Per-target manifests drift
Maintaining N copies of the same manifest invites drift. Resolved: §1 clause 2 + DEC-2462 — single canonical manifest; AC #17.

### ISS-002 — Sigstore broken by non-reproducible builds
Without per-target reproducibility, Sigstore verify-rebuild round-trip fails. Resolved: §1 clause 3 + DEC-2463 + adapters/common.rs reproducibility helpers; AC #13.

### ISS-003 — Cursor users see broken skill UI
Cursor doesn't render Skills. Shipping skills/ produces dead bytes. Resolved: §1 clause 5 + DEC-2465 — Cursor adapter omits skills; AC #8.

### ISS-004 — Authors ship to targets not declared
Pack succeeds for any target name; targets[] in manifest becomes informational. Resolved: §1 clause 9 — cross-check at pack time; AC #17.

### ISS-005 — Adapters drift toward proprietary extensions
"Just one custom field" is the failure mode. Resolved: §1 clause 12 + DEC-2466 — only target-published formats; failure mode row 14.

### ISS-006 — Install friction for runtimes that need bridge binary
Non-claude-code users have to "also install the bridge" → adoption drops. Resolved: §1 clause 4 + DEC-2464 — bundle binary for cursor + cowork + codex-cli; AC #10.

### ISS-007 — Single-arch binary leaves cross-platform users stranded
Linux-only bundle fails for macOS + Windows users. Resolved: §11.9 — multi-arch default (3 binaries) with single-arch opt-in via --arch flag.

## §3 — Resolution

All 7 ISS findings resolved by extending §1 (clauses 2, 3, 4, 5, 9, 12), defining the Adapter trait and per-target bundle layouts, implementing reproducibility helpers, and writing 5 integration tests including per-target structural validators.

Final score: **10/10.**

*End of FR-PLUGIN-007 audit.*
