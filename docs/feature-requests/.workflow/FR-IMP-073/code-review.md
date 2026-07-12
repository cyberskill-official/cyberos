# FR-IMP-073 — code-review packet (steps 17–18, code-review@1)

Status: `reviewing`. **HALTED at HITL gate 1 (review acceptance, `reviewing → ready_to_test`).** A recorded human verdict advances or routes back — the agent does not self-cross (EXECUTION-DISCIPLINE §2a).

## Diff under review

- `a6a2f3d` (pre-committed by operator): the 16-file icon copy — exactly the spec §0 `modified_files` list, byte-verified.
- `4da57a4` (this run): release.yml android+ios drift guards, RELEASE.md recopy runbook, phase bundle, status flips.

## §1 clause → evidence map (all 8 clauses)

| §1 clause | Requirement | Evidence / named check | Verdict |
|---|---|---|---|
| 1 | Shells MUST ship the brand icon in every required density/size | §5 verification script (16/16 byte-identical to brand source) — run PASS this session; spot-check hashes in spec §8 | ✅ machine-proven |
| 2 | Root cause confirmed by direct comparison | Pre-fix hash `27ed36…` (48×48 Capacitor template) vs post-fix `bd102a…` recorded in spec §8; `capacitor.config.ts` + package.json greps confirm no icon tooling ever wired | ✅ evidence recorded |
| 3 | Tauri-generated source set identified as correct source | Hash table spec §8; source paths exist and match file-for-file (§3 mapping) | ✅ machine-proven |
| 4 | Fix MUST be a straight filesystem copy, no transformation | Byte-identical hashes are the mechanism — a transform would break equality; §5 PASS | ✅ machine-proven |
| 5 | MUST NOT introduce a new brand asset | Guard compares against the already-shipped desktop set; no image generation anywhere in the diff (binary files in `a6a2f3d` are copies — hash-equal to pre-existing repo files) | ✅ structural |
| 6 | Regression guard SHOULD exist | Option B: two CI assert steps in release.yml (android 15-file, iOS 1-file), run after `cap sync`; Option A: RELEASE.md recopy runbook. Negative path tested (tampered file → DRIFT fires) | ✅ implemented both |
| 7 | Safe-zone padding SHOULD be visually confirmed | **Open — human-only.** Folded into AC #3's checklist; cannot be machine-verified (agent has no image rendering). Reviewer: preview `ic_launcher_foreground.png` under a circular/squircle mask | ⏳ for reviewer |
| 8 | MUST NOT modify adaptive-icon XML wiring | `git show a6a2f3d --stat` + `4da57a4` diff touch zero `.xml` files under `apps/web/android` | ✅ machine-proven |

## Acceptance criteria status

- AC #1 (byte-match): **green** (§5 PASS).
- AC #2 (no unrelated Capacitor files): **green by intent** — `a6a2f3d` touches exactly the 16 icon paths under `apps/web/{android,ios}`; note the commit also carried non-Capacitor docs/spec files (operator's batching), recorded honestly here for the reviewer.
- AC #3 (visual sanity, 4-item checklist): **yours** — the review verdict should cover it (or defer explicitly to the testing phase). Optional contact sheet: spec §6 `montage` snippet.
- AC #4 (real gated CI build): **deferred by design** — `ANDROID_RELEASE`/`IOS_RELEASE` off; exercised on your first real gated run.
- AC #5 (regression guard): **green** (Option A + B in `4da57a4`).
- AC #6 (doc consistency, 4 lists × 16 paths): **green** (checked during authoring; §5 script is the executable form).

## Edge-case matrix closure

All 10 rows in phase-bundle.md map to a guard branch, an explicit human-check delegation (row 5), or a justified-empty category (row 9 security — no runtime surface). No row is unaddressed.

## Reviewer verdict needed

Reply with e.g. **"FR-IMP-073 review: approved"** (→ `ready_to_test`, then testing gates run) or **"FR-IMP-073 review: rejected — <reason>"** (→ routed back to `ready_to_implement`, `routed_back_count` +1).

*End review packet.*
