# TASK-APP-006 — code-review packet (steps 17–18)

Status: `reviewing`. **HALTED at HITL gate 1 (review acceptance).** Diff under review: the `feat(desktop): TASK-APP-006 ...` phase commit (6 new files, 3 new dirs; `modified_files: []` honored).

## §1 clause → evidence map (all 9 clauses)

| §1 clause | Requirement | Evidence | Verdict |
|---|---|---|---|
| 1 | Two paths pointing at existing GH-Releases artifacts, no new builds | Manifests reference the `desktop` job's `.dmg`/NSIS `.exe` URLs (production URL shape grounded in tauri.conf.json); no build steps anywhere in the new workflow | ✅ |
| 2 | Package managers ≠ stores; no dependency on TASK-APP-003/004 flags | Prep jobs consume the always-on `desktop` job's artifacts only; MAS/MSIX channels never referenced | ✅ |
| 3 | Cask declares version/sha256/url/name/desc/homepage/app + livecheck | All present in `cyberos.rb` (ruby -c verified) | ✅ |
| 4 | winget three-file set per Microsoft's schema | All three authored; ManifestVersion/switches hedges preserved in-file per spec §3's own caveat | ✅ |
| 5 | No PR opened as a consequence of landing | Nothing submits; guard is standing CI, not policy prose | ✅ |
| 6 | Flags gate PREPARATION only; no submission command under any state | AC #3 checks (a)+(b) proven: 0 matches repo-wide including this FR's own workflow (split-pattern idiom; comments deliberately avoid contiguous command strings) | ✅ |
| 7 | Version/hash re-derivable, never hand-maintained | REAL render logic implemented + dry-run proven (fake artifact → correct version/URL/sha in both ecosystems, placeholders eliminated, post-render asserts fire) | ✅ |
| 8 | Answer sheet: quality bars + PAT scopes | `package-manager-submission.md` — 8 Cask rows, 6 winget rows, shared ops rules | ✅ (`pending-human` rows for you) |
| 9 | zap trash: verified before final | Real `brew uninstall --zap` test required + recorded in the answer sheet; candidates flagged in-file as unverified | ✅ (deferred-by-requirement, honestly marked) |

## Implementer-disclosed findings

1. **Spec's own §3 skeleton would have violated its own AC #3:** the skeleton's comment text contained submission-command strings verbatim — my workflow phrases every comment to avoid the contiguous strings, and the guard splits its patterns (else the guard flags itself, as TASK-APP-005's did).
2. **Placeholder echoes upgraded to real render logic** (spec §6 deferred it to WORKER = this run): sed-based re-derivation + post-render asserts, dry-run-proven. NSIS-switch and livecheck open questions are untouched by the render (they live in fields the render never rewrites), so implementing it now contradicts nothing.
3. **Version + locale winget files authored** (spec showed only the installer file): schema-minimum fields, same hedge comments; regenerate from live templates if `winget validate` complains.
4. **AC #4's anchor**: one unconditional guard job anchors both skipped-conclusion assertions (the two prep jobs remain independently gated).
5. AC #1/#2 (`brew audit`, `winget validate`): **expected-pending** — neither tool exists in this container; both are answer-sheet pre-PR requirements against RENDERED drafts (the in-repo placeholders would rightly fail `brew audit`).

## Machine gates

ruby -c OK; YAML ×4 PASS; AC #3 (a)+(b) 0 matches; render dry-run PASS end-to-end; coverage N/A (declared); run-gates floor GREEN.

## Reviewer verdict needed

**"TASK-APP-006 review: approved"** or **"TASK-APP-006 review: rejected — <reason>"**.

*End review packet.*
