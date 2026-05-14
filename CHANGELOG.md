# Changelog — CyberOS

All notable changes to the umbrella CyberOS repository, newest-first.

## 2026-05-14 — AUTH module RFC + sign-in mockup

- Added `services/auth/RFC.md` — implementation RFC with 5-slice ship plan, audit-chain integration design, and 5 open questions blocking slice 1.
- Added `services/auth/mockups/sign-in.html` — first AUTH UI mockup applying design-system Part 21 Liquid Glass defaults, Umber + Ochre anchors, Be Vietnam Pro first, passkey-first flow with password fallback, MFA chips, BRAIN audit-chain trust footnote.
- Verification pass against shipped modules:
  - memory: 222 tests pass + 1 skip (numpy + jsonschema needed for full green). Real bug found AND fixed: `check_manifest_validates` was skipping parseability when jsonschema absent → `cyberos state` returned READY on a broken manifest. Patched to always parse `manifest.json` first (regardless of jsonschema availability) and report `False` on `JSONDecodeError`; the optional schema-validation layer still skips cleanly when jsonschema is absent. Verified: all 4 `tests/test_state.py` tests pass, full suite 238 pass / 1 skip / 0 fail. Also verified by simulating absent jsonschema via import hook — good manifest still returns True with "parseability OK, schema skip"; bad manifest returns False with "manifest.json unparseable: ...".
  - skill: 20 SKILL.md bundles structurally verified, 4 crates, 8 inline Rust tests. `cargo build` not run (sandbox-only limitation).
  - cuo: 15/15 pytest + 15/15 routing fixtures pass. Catalog discovers all 20 skills correctly.
- Stale-claim drift surfaced (none are blockers, all are doc-only):
  - Memory tests: bootstrap says 245, README says 255, actual is 238 collected.
  - Doctor invariants: bootstrap says 16, README says 15, actual is 13 on a fresh store.
  - Docs pages: bootstrap says 32, strategy says 31, actual is 33 HTML files (32 user-facing + nav include).
  - Strategy §3 Tier-1 #2 and §5 Session-1 #1 list "wire Pagefind" as a to-do; Pagefind is already built and serving (v1.5.2, 32 pages indexed).
  - DEPLOYMENT.md is at `website/docs/DEPLOYMENT.md` (bootstrap implies it lives at `website/`).
- Docs site deploy-prep findings:
  - 6 real broken internal links to 2 missing architecture pages: `architecture/services.html` (5 refs from LEARN/HR/INV/ESOP/REW) and `architecture/runtime.html` (1 ref from CHAT). These are demand-gen blockers — fix before public deploy or convert the link targets.

## 2026-05-14 — Consolidation pass

Moved all CyberOS-related artifacts into a single umbrella at `cyberos/`:

- `workbench/CyberOS-docs/` → `cyberos/website/docs/`
- `workbench/CYBEROS_STRATEGY.md` → `cyberos/strategy/CYBEROS_STRATEGY.md`
- `workbench/cyberskill-vn-skills/` → `cyberos/public-skills/`
- `/design-system/` → `cyberos/design-system/`
- `/landing-page/` → `cyberos/website/landing/`

This enables clone-and-go for new sessions and keeps strategic + technical + design content co-located.

See per-module CHANGELOG.md files for module-specific history:
- `memory/docs/CHANGELOG.md`
- `skill/docs/CHANGELOG.md`
- `cuo/docs/CHANGELOG.md`
- `design-system/CHANGELOG.md`
- `website/docs/index.html` (the rendered changelog page)
