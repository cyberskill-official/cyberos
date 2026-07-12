# FR-IMP-073 — ship-workflow phase bundle (steps 1–12)

Run: 2026-07-13, ship-feature-requests v2.4.0. Queue echo: `queue: picked FR-IMP-073 (priority=MUST, created=2026-07-13) over 4 other eligible FRs`.

## Steps 1–2 — repo context map (repo-context-map@1)

- **Patterns to follow:** release.yml gated-job convention (`vars.ANDROID_RELEASE`/`IOS_RELEASE`, own gate per platform); loud `::error::` annotations for CI failures; hash comparison via `sha256sum` on ubuntu runners and `shasum -a 256` on macos runners (macos images do not ship sha256sum); docs runbooks live in `docs/deploy/RELEASE.md` with per-feature FR references.
- **State of the fix at run start:** the 16-file icon copy was already committed at `a6a2f3d` and pushed; `git show a6a2f3d --stat -- apps/web/android apps/web/ios` touches exactly the 16 icon paths and nothing else under the Capacitor trees (AC #2 intent satisfied — the commit also carried unrelated docs/spec files, but zero unrelated Capacitor project files, which is the failure mode AC #2 exists to exclude).
- **Blast radius of the remaining work:** 2 files — `.github/workflows/release.yml` (2 inserted steps), `docs/deploy/RELEASE.md` (one section refresh + recopy runbook). Files outside the FR's immediate domain: 0. **ADR condition (steps 3–4): not triggered** (threshold is >3 outside-domain files).
- **Mock condition (steps 7–8): not triggered** — no external dependency (`has_external_dependency` absent/false; pure filesystem + CI change).

## Steps 5–6 — edge-case matrix (edge-case-matrix@1)

Derived from spec §10 (10 rows) and mapped to guards/tests. Categories per rubric:

| # | Category | Case | Covered by |
|---|---|---|---|
| 1 | regression | `npx cap add` re-scaffold resets icons to Capacitor template | Option B guard (both jobs) fails with per-file `::error::` DRIFT; Option A runbook in RELEASE.md tells the operator the recopy commands |
| 2 | regression | desktop icon rebrand without mobile recopy | same guard, opposite direction — hash mismatch is symmetric |
| 3 | null/empty | brand-source file missing entirely (e.g. bad checkout, deleted dir) | explicit `[ -f "$a" ]` branch: `::error::brand source missing`, fail=1 (does not silently pass an absent source) |
| 4 | null/empty | Capacitor-side file missing (partial re-scaffold) | explicit `[ -f "$b" ]` branch: `::error::Capacitor icon missing`, fail=1 |
| 5 | malformed | corrupted-but-present PNG that still hashes equal on both sides | structurally invisible to any hash guard — delegated to AC #3 human visual check (spec §10 row 3); recorded, not claimed |
| 6 | platform | macos runner lacks `sha256sum` | iOS leg uses `shasum -a 256`; android leg (ubuntu) uses `sha256sum` — each leg uses its runner's native tool |
| 7 | platform | case-sensitivity drift macOS→Linux (spec §10 row 8) | guard paths are literal (no globs); a casing drift = missing-file branch fires loudly on the Linux runner |
| 8 | concurrency | `cap sync` rewriting icons mid-job after the copy | guard runs AFTER `npx cap sync` in both jobs, so it asserts the final packaged state, not the pre-sync state |
| 9 | security | none — no auth/tenancy/injection surface (binary asset compare in CI) | n/a; no SECURITY-class row (matrix audit note: category consciously empty with justification, not vacuously skipped) |
| 10 | degradation | guard loops must not abort at first failure (hide later drift) | `fail=1; continue` accumulator pattern — every drifted file reported in one run |

## Steps 9–10 — implementation plan (impl_plan@1)

1. Insert "Assert Android launcher icons match the desktop brand source (FR-IMP-073)" step in the `android` job immediately after `npx cap sync android` — 15-file sha256 loop, accumulator failure, `::error::` per file. ✅
2. Insert "Assert iOS app icon matches the desktop brand source (FR-IMP-073)" step in the `ios` job immediately after `npx cap sync ios` — single-file shasum compare. ✅
3. Refresh `docs/deploy/RELEASE.md` mobile one-time-init section (stale "projects do not exist yet" claim) + add the FR-IMP-073 recopy runbook (Option A) with exact `cp` commands and a pointer to the CI guard. ✅
4. Decision recorded (spec §9 open question #1): **both Option A and Option B** — B is the load-bearing guard (converts silent regressions into loud CI failures at near-zero cost on two off-by-default jobs), A is the co-located operator instruction the guard's error message points at. One-line rationale in lieu of ADR per EXECUTION-DISCIPLINE §2.1 (self-resolvable choice, recorded).

## Steps 11–12 — observability injection (observability-injection@1)

The new code is two CI steps; their observability surface:

- Every failure branch emits a GitHub Actions `::error::` annotation naming the exact file pair and the recovery pointer (`docs/deploy/RELEASE.md`) — surfaced in the PR/run annotations UI, not buried in logs.
- Success emits one summary line per leg ("all 15 Android launcher assets match…" / "iOS AppIcon matches…").
- Error-branch log coverage: 4/4 branches (source-missing, dest-missing, drift, success) = 100 % ≥ the 80 % floor. No tenant/subject IDs apply (no runtime data path).

## Machine verification (run 2026-07-13, this session)

- spec §5 script: **PASS** — all 16 files byte-identical to brand source.
- Android guard logic dry-run (exact CI shell): **PASS** (15 files green).
- Negative path: tampered copy → DRIFT branch fires, fail=1. **PASS**.
- `release.yml` YAML parse: **PASS**.
- `.cyberos/cuo/gates/run-gates.sh`: **GREEN** (floor profile; build/lint/test/coverage unconfigured in gates.env → skipped by config, caf/awh disabled).
- Coverage gate note: touched files are YAML + markdown + binary PNGs — line-coverage tooling does not apply; per workflow §1a the FR "declares awh N/A in its §1 and relies on coverage + caf + the review gate; it does not fabricate" — same declaration here for line coverage, with the §5 hash verification + guard dry-runs as the equivalent machine evidence.

*End phase bundle.*
