# TASK-APP-005 — ship-workflow phase bundle (steps 1–12)

Run: 2026-07-13, ship-tasks v2.4.0. `queue: picked TASK-APP-005 (priority=SHOULD, slice 3) — 4th of 5; TASK-IMP-073/003/004 parked at HITL gate 1`.

## Steps 1–2 — repo context map

- **Patterns:** gated dispatch workflow + unconditional lint/anchor job (established this batch); `tauri build --bundles deb` payload reuse (release.yml ubuntu leg confirms the deb pipeline + `libwebkit2gtk-4.1` runtime generation); snapcore/action-build+publish for Store upload.
- **Blast radius:** 4 new files, `modified_files: []` honored (release.yml untouched). Outside-domain: 2 (flathub-manifest/ at repo root — new dir; docs/deploy/) ≤ 3 → **ADR skipped**. **Mocks skipped** — Snap Store upload is credential-gated real infra; Flathub has no service to mock (its infra builds from the manifest).
- **Architectural split honored (spec §1 #2):** Snap = CI-automatable channel; Flathub = manifest + Stephen-gated external PR, deliberately NO CI gate, structurally guarded (AC #6).

## Steps 5–6 — edge-case matrix

| # | Category | Case | Covered by |
|---|---|---|---|
| 1 | null/empty | `SNAP_RELEASE` unset/false | job `if:` skip + anchor job (AC #5) |
| 2 | null/empty | no `.deb` produced | staging step `::error::` + exit 1 |
| 3 | malformed | binary absent/renamed inside the .deb | name-normalization loop (cyberos / cyberos-desktop / CyberOS) → loud failure with tree listing (spec §10 row 3 upgraded to hard CI failure) |
| 4 | malformed | snapcraft.yaml confinement weakened | standing AC #2 lint (proven locally) |
| 5 | security | plug-list scope creep | standing AC #3 exact-set lint (proven locally); future additions need §9 justification (code-review norm per spec — no fabricated prose-parser) |
| 6 | security | Flathub PR automation sneaks in | AC #6 standing grep — **found + fixed a real self-match** (the guard's own pattern; split-pattern idiom, disclosed) |
| 7 | security | store credential outside secret manager | `SNAPCRAFT_STORE_CREDENTIALS` via `${{ secrets.* }}` only |
| 8 | bounds/platform | gnome-extension/base version mismatch | `snapcraft pack` fails at extension resolution (AC #1, first gated run); architectures-form caveat kept in-file |
| 9 | bounds/platform | -dev vs runtime `-N` package confusion | `stage-packages` uses runtime names (`libwebkit2gtk-4.1-0`); smoke-test requirement documented (spec §10) |
| 10 | regression | skeleton's staging-path bug (snap/dist vs project-root dist/) | fixed: extraction to `apps/desktop/src-tauri/dist` where `source: dist/` actually resolves; disclosed |
| 11 | degradation | credentials macaroon expiry | publish-step auth failure → rotate; documented ops note |
| 12 | degradation | snapcraft.yaml `version:` drifts from VERSION | disclosed known gap + manual bump note in answer sheet; stamper follow-up suggested (not silently wired in — out of FR scope) |

## Steps 9–10 — implementation plan (executed)

1. `snap/snapcraft.yaml` — §3 skeleton + lint-invariant header + WORKER caveats preserved in-file. ✅
2. `release-snap.yml` — §3 skeleton + corrections (anchor/lint job with AC #2/#3/#6 standing checks; staging path fix; binary normalization; toolchain trio; artifact upload). ✅
3. `flathub-manifest/os.cyberskill.world.desktop.yml` — §3 skeleton with PROVISIONAL app-id banner + submission-model explainer. ✅
4. `docs/deploy/linux-store-submission.md` — two-section sheet (Snap metadata + Flathub checklist), smoke-test gate, blockers, ops notes. ✅
5. `.desktop` entry — **deliberately NOT authored** (spec §6: depends on the §9 app-id decision; authoring against a provisional id guarantees rework). ✅ (by omission, as specified)

## Steps 11–12 — observability injection

Standing lints print pass/fail lines with `::error::` annotations; staging failure paths name the missing artifact and dump the extracted tree; plug-drift failure prints want/got. Every new error branch emits a diagnostic before exiting.

## Machine verification (this session)

- YAML parses ×3: **PASS** (one real bug caught: unquoted `confinement: strict` scalar → block-scalar fix).
- AC #2 confinement lint: **PASS** (run locally).
- AC #3 exact plug-set lint: **PASS** (same awk logic as CI, run locally).
- AC #6 structural guard: **PASS after a real fix** — the guard initially matched its own pattern; split-pattern self-exclusion applied and re-proven.
- AC #1 `snapcraft pack`: **expected-pending** (snapcraft/snapd unavailable in this container; first `SNAP_RELEASE=true` run + local snap-install smoke test cover it).
- AC #4 `flatpak-builder` validation: **expected-pending** (tool not installed here; required pre-PR step recorded in the answer sheet).
- AC #8: manual secret-reference assert (no scan gate exists yet — TASK-IMP-003 draft); all credentials are `${{ secrets.* }}`.

*End phase bundle.*
