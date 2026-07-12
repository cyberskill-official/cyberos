---
fr_id: FR-SKILL-116
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# FR-SKILL-116 audit

## §1 - Verdict summary

Audited for extraction determinism (the checker must parse, not guess), allowlist semantics, and forward-compatibility with FR-CUO-209's expanded set. Two contract holes closed (prose-fuzzy extraction; allowlist covering only one rule kind). Traceability closes over t01-t06 in tools/cyberos-init/tests/test_chain_coverage.sh.

## §2 - Findings (all resolved)

### ISS-001 command-doc extraction was prose-fuzzy
"Every skill named by plugin/commands/*.md" had no grammar - a checker cannot grep intent. Resolved: §1 #2 defines the deterministic backtick token grammar `<name>-(author|audit)` for command docs, `skill:` keys for the chain doc.

### ISS-002 allowlist exempted MISSING but not UNPAIRED
FR-CUO-209 vendors four intentionally single NFR skills; the pair rule would have failed the build the day they land. Resolved: §1 #3 allowlist entries exempt both rule kinds with a reason string; §1 #4 scopes UNPAIRED to non-allowlisted names.

### ISS-003 zero-reference degeneracy
A workflow-doc format change yielding zero extracted skills would pass vacuously. Resolved: §10 #1 makes 0 references exit 2 (structure changed), never a pass.

### ISS-004 empty-dir false positive
A skill directory without SKILL.md counted as present. Resolved: presence = SKILL.md inside the dir (§10 #2).

### ISS-005 side-effect-free guarantee missing
CI reuse (FR-IMP-068) requires read-only behavior; nothing said so. Resolved: §1 #6 + AC 6 hash-tree-before/after assertion.

### ISS-006 build-integration failure path untested
Clause #5 (build fails on violation) had no test. Resolved: AC 3 / t03 run a patched checkout whose set drops the pair and assert build.sh itself fails.

## §3 - Resolution

All six findings addressed as cited. The FR now kills the bug class (hardcoded set drifting from the chain) rather than the single instance. **Score = 10/10.**

*End of FR-SKILL-116 audit.*
