# rules_sha: stored-not-recomputed and docs-tools-excluded - accepted by-design - 2026-07-19

A fleet-health finding this run flagged that `rules_sha` (TASK-IMP-074) is stored rather than recomputed, and that its fingerprint cone excludes `docs-tools/`. Both halves are true. At the 2026-07-19 PLAN gate Stephen judged this acceptable by-design (chose not to author a fix). This note records the finding, the confirmation, and the reasoning, so it is not re-surfaced as a bug.

## The finding, confirmed

- `build.sh:376` computes `rules_sha` as a content fingerprint over `cuo plugin mcp cli memory` only, and `build.sh:384` writes it once into the payload `manifest.yaml`.
- Every consumer reads that stored value: `audit-fleet.sh:37`, `version.sh:61,73-74`, `check-version-sync.sh:56`. None recomputes the fingerprint from the live installed tree.
- `docs-tools/` is not in the fingerprint cone.

Two consequences follow, both real:

1. Stored-not-recomputed: comparing stored-to-stored across channels detects that two channels were built from different rule content, but it cannot detect an in-place edit of an installed rule file after the fact - the stored value still "matches" itself.
2. docs-tools-blind: a change to a vendored `docs-tools/` script does not move `rules_sha`, so the fingerprint does not signal docs-tools drift.

## Why by-design (the 2026-07-19 decision)

`rules_sha`'s stated purpose (build.sh comment, TASK-IMP-074) is a content fingerprint of the distributed RULE trees, for cross-channel drift detection - "were these two channels built from the same rules." Compared stored-to-stored across channels, that is exactly what it does. It was never specified as a tamper-detector for post-install in-place edits, and detecting those needs a different mechanism (recompute-at-verify). Excluding `docs-tools/` is consistent with the same intent: `docs-tools/` is tooling, not rules, and several of its scripts are repo-side tools that are not even vendored into every channel (skill-log, cone-audit, fm001-migrate), so folding them into a "rule content" fingerprint would make the fingerprint mean something other than what it says.

## Escape hatch if the posture ever changes

If in-place-edit detection is later wanted, the follow-up is recompute-at-verify: have `version`/`audit-fleet` recompute the fingerprint from the live installed tree and compare to the stored value. TASK-IMP-074 §9 already names client-side comparison in `cyberos update`/plugin/MCP as the designated follow-up. Extending the cone to `docs-tools/` would be a separate, deliberate decision about what the fingerprint is meant to cover.

No code change was made. This is a recorded decision, not a defect.
