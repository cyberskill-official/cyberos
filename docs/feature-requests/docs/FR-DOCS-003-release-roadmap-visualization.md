---
id: FR-DOCS-003
title: "Release roadmap visualization - a generated roadmap.html rebuilt on every release and deploy from FR frontmatter + CHANGELOG + VERSION"
module: docs
priority: MUST
status: reviewing
class: product
verify: T
phase: Wave C - strengthen the workflows
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: null
memory_chain_hash: null
related_frs: [FR-DOCS-001, FR-DOCS-002, FR-IMP-069]
depends_on: [FR-DOCS-002]
blocks: []
source_pages:
  - tools/docs-site/build.sh
  - docs/feature-requests/BACKLOG.md
  - CHANGELOG.md
  - .github/workflows/deploy.yml
  - .github/workflows/release.yml
source_decisions:
  - "2026-07-12 operator request (plan approval note): 'build a visualize roadmap (html) that trigger on every release/deploy'."
  - "Data sources are the existing records of truth only: FR frontmatter (statuses), CHANGELOG.md (what each version shipped), VERSION (current). No new state is introduced."
  - "Follows FR-DOCS-002's doctrine: generated output under dist/website, deterministic, dependency-free, never committed."
language: node (stdlib only) + bash + GitHub Actions YAML
service: tools/docs-site/
new_files:
  - tools/docs-site/render-roadmap.mjs
  - tools/docs-site/tests/test_render_roadmap.sh
modified_files:
  - tools/docs-site/build.sh
  - .github/workflows/deploy.yml
  - .github/workflows/release.yml
---

# FR-DOCS-003: Release roadmap visualization

## §1 - Description

One generated page that answers "where is the platform and what is coming" from the data the repo already maintains: every FR's status, every version's changelog entries, and the current VERSION - refreshed automatically whenever a release is cut or the site deploys.

Normative clauses:

1. A builder `tools/docs-site/render-roadmap.mjs` MUST generate `roadmap.html` into the site output dir from exactly three inputs: FR frontmatter across `docs/feature-requests/*/FR-*.md` (excluding `_audits/`, `_archive/`, `*.audit.md`), `CHANGELOG.md` version sections, and `VERSION`. It MUST use node stdlib only (no dependencies), matching the FR-DOCS-002 renderer constraint.
2. The page MUST contain four blocks: (a) a header stamp - VERSION + built-from commit; (b) a release timeline - one entry per CHANGELOG version section (version, date, entry lines), newest first; (c) a pipeline board - the 10 lifecycle statuses as columns in STATUS-REFERENCE order, each FR as a row item (id, title, module, class badge) with per-column counts; (d) module rollups - per module: counts by status and by class. Counts MUST be derived from frontmatter, never from BACKLOG.md.
3. Client-side filtering by module, class, and status MUST work with inline vanilla JS (no external assets, no CDN), degrading gracefully to the full unfiltered page when JS is off (content present in the DOM regardless of filter state).
4. `tools/docs-site/build.sh` MUST invoke the roadmap builder on every site build, and BOTH `.github/workflows/deploy.yml` (docs job) and `.github/workflows/release.yml` MUST run the site build step so the published roadmap refreshes on every deploy AND every release tag - the release job publishing to the same docs target the deploy job uses.
5. Determinism and honesty per FR-DOCS-002: same inputs -> byte-identical `roadmap.html` (the header stamp uses VERSION + commit, no wall-clock timestamp); an FR file whose frontmatter fails to parse MUST fail the build non-zero naming the file; a CHANGELOG that yields zero version sections MUST fail likewise (structure changed under the parser).
6. The generated site nav MUST include the roadmap page (via the FR-DOCS-002 filesystem-derived nav or an explicit nav hook, whichever that builder exposes).
7. The page MUST render acceptably in light and dark contexts using the site's existing chrome/tokens - no hardcoded colors outside the site's token set.

## §2 - Why this design

The roadmap is a VIEW, so it introduces no new records: FR frontmatter is already the declared source of truth, CHANGELOG already narrates releases, and VERSION already names "now". Rebuilding on deploy AND release satisfies the trigger requirement with the two hooks that already exist rather than a scheduler. Excluding wall-clock time from the stamp preserves FR-DOCS-002's byte-identical rebuild property, which is what makes "did anything change?" a diff instead of a guess.

## §3 - Contract

```
node tools/docs-site/render-roadmap.mjs <repo-root> <out-dir>
  writes <out-dir>/roadmap.html
  exit 0 ok | exit 1 unparseable FR frontmatter or empty CHANGELOG parse (file named on stderr)
```

Pipeline board column order = STATUS-REFERENCE §1 order: draft, ready_to_implement, implementing, ready_to_review, reviewing, ready_to_test, testing, done, on_hold, closed.

## §4 - Acceptance criteria

1. **Three inputs only, stdlib only** (§1 #1) - the builder reads nothing outside the three sources (strace-free proof: code review + a fixture tree where ONLY those inputs exist); `node -e "require('tools/docs-site/render-roadmap.mjs')"`-style import check shows no third-party requires.
2. **Four blocks render with true counts** (§1 #2) - against a fixture FR tree (known status mix incl. improvement-class rows), the board's per-column counts equal the fixture's frontmatter tally, the timeline lists the fixture CHANGELOG versions newest-first, and the rollups match per module.
3. **Filters work and degrade** (§1 #3) - with JS: selecting module+status hides non-matching rows (DOM assertions via node's parser on the emitted markup + the inline script's data attributes); without JS: all rows present.
4. **Wired into build, deploy, and release** (§1 #4) - build.sh calls the builder; both workflow files contain the site-build step; the release workflow publishes to the same target (structural asserts on the three files).
5. **Deterministic** (§1 #5) - two builds over the same fixture are byte-identical; adding one FR changes the output (sanity inverse).
6. **Honest failure** (§1 #5) - a fixture FR with broken frontmatter fails the build naming the file; an empty CHANGELOG fixture fails.
7. **In the nav** (§1 #6) - the generated site's nav includes the roadmap link (assert on the built fixture site).
8. **Token-clean styling** (§1 #7) - the emitted HTML contains no hex colors outside the site token definitions (grep-level assert), so the page inherits the site theme.

## §5 - Verification

```bash
# tools/docs-site/tests/test_render_roadmap.sh
t01_inputs_and_stdlib_only()     # AC 1
t02_four_blocks_true_counts()    # AC 2
t03_filters_and_nojs_degrade()   # AC 3
t04_wired_build_deploy_release() # AC 4
t05_byte_identical_rebuilds()    # AC 5
t06_honest_failures()            # AC 6
t07_nav_link_present()           # AC 7
t08_token_clean_styles()         # AC 8
```

## §6 - Implementation skeleton

Frontmatter reader: reuse/port the minimal YAML-subset reader pattern from `scripts/migrate_improvement_to_fr.py` in ~60 lines of node (key: value scalars + flow lists suffice for the fields used). CHANGELOG parser: split on `^## ` version headings. Emit: one template literal per block, data attributes (`data-module`, `data-status`, `data-class`) driving a ~30-line inline filter script.

## §7 - Dependencies

Depends on FR-DOCS-002 (site builder home, chrome/tokens, nav derivation - currently `reviewing`, so this FR queues right behind it). Related: FR-IMP-069's release job gives release.yml the natural place for the publish step; FR-DOCS-001's determinism rules are inherited via FR-DOCS-002.

## §8 - Example payloads

```
$ node tools/docs-site/render-roadmap.mjs . dist/website
roadmap: 477 FRs (335 draft, 7 ready_to_implement, 14 implementing, 1 reviewing, 115 done, 1 on_hold), 12 releases, VERSION 1.7.0
```

## §9 - Open questions

None blocking. A per-release "which FRs shipped in vX.Y.Z" join (matching FR ids inside changelog entries) renders when ids appear in the entry text and is omitted otherwise - a data-quality nudge, not a requirement, noted on the page.

## §10 - Failure modes inventory

1. FR volume growth (477 -> thousands) makes the page heavy - rows are plain elements with data attributes (~100 bytes each); at 5k FRs the page stays ~1 MB before gzip. Acceptable; a pagination follow-up is named in the page footer comment when count > 2000.
2. Status vocabulary drift (a file with an invalid status like the fixed FR-EVAL-001 case) - unknown statuses render in an `invalid` bucket at the far right, loudly visible instead of dropped, and the build stderr lists them (warning, not failure - the FR audit pipeline owns enforcement).
3. CHANGELOG format evolution - parser requires `## X.Y.Z` heads; zero matches fails honestly per §1 #5.
4. Deploy and release racing (tag push + main push same minute) - both jobs regenerate from their own checkout; last writer wins with deterministic content from its commit; no shared mutable state.
5. Dark-mode regressions - token-only styling (AC 8) delegates theming to the site chrome; no page-local palette to drift.

## §11 - Implementation notes

Keep the summary stdout line (§8) stable - it doubles as the deploy log's roadmap health line. The invalid-status bucket doubles as a live data-quality monitor for the whole FR corpus; keep it.

*End of FR-DOCS-003.*
