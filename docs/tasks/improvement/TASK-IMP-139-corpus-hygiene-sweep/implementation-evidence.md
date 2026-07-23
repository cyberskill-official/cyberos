# TASK-IMP-139 implementation evidence — unblocked half (batch/8 parallel wave)

- worker: TASK-IMP-139 unblocked-half subagent · 2026-07-23 · branch `batch/8-audit-hardening`, worktree at `be89966b`
- scope: the MECHANICAL and EVIDENCE halves only. Both operator gates are untouched: no `# UNREVIEWED` marker was edited (Gate 1), no task status was changed (Gate 2). Nothing was committed — the final sequential pass owns commits and full verification.

## Clause coverage

| Spec clause | State after this worker |
|---|---|
| 1.1 / 1.2 (marker fork + sweep) | EVIDENCE ONLY: decision brief + re-derived enumeration at `assets/unreviewed-fork-brief.md`. Zero markers touched. |
| 1.3 (module-case normalization) | DONE: 251 files, each a single `module:` line lowercased. Zero mixed-case and zero folder-mismatch values remain corpus-wide. |
| 1.4 (lint rule + rubric doc) | HALF-DONE: FM-117 added to `tools/install/docs-tools/task-lint.mjs` (error severity, both halves). `modules/skill/task-audit/RUBRIC.md` documentation DEFERRED — outside this worker's file ownership; the lint-and-rubric-ship-together discipline requires the final pass to add it in the same commit as the rule. |
| 1.5 (12 reconcile reports) | EVIDENCE ONLY: 12 dossiers at `assets/reconcile/TASK-*.md`, each embedding the verbatim `reconcile-report@1` plus tree/git analysis and a RECOMMENDED verdict. No verdict applied, no status flipped. |
| 1.6 (`test_corpus_hygiene.sh`) | NOT BUILT (final pass): the census logic to reuse lives in this evidence + the brief (FM-112-equivalent scan — see Deviations #3). |
| 1.7 (CHANGELOG) | NOT WRITTEN (final pass; depends on the Gate-1 branch and Gate-2 tally). |

## What changed and why

1. **251 × `docs/tasks/*/TASK-*/spec.md` — `module:` value lowercased** (e.g. `module: AUTH` → `module: auth`). Case-only, value-only: quotes/whitespace/markers preserved, no file renames (spec edge case: APFS case-folding never exercised). Verified afterwards: every diff is exactly 1 insertion / 1 deletion and every changed line is a `module:` line. Baseline → end state: 251 uppercase-value files → 0; folder-mismatch-after-lowercase was 0 before and after (all 26 distinct uppercase values were straight uppercase of their folder), so lowercasing alone reached the full §1.3 invariant.
2. **`tools/install/docs-tools/task-lint.mjs` — FM-117 added** (next free task@1 FM id; wired inside `checkFrontmatterFields` like its sibling per-field rules; header family range updated). Two independently-firing halves per the audit's ISS-006: value must be lowercase; value must equal the containing `docs/tasks/<module>/` folder. List value → scalar error. Absent stays legal (the rule governs values). Outside the `docs/tasks/<module>/<task>/spec.md` shape or under `_`/`.` trees, only the case half is judged.
3. **12 reconcile dossiers** under `assets/reconcile/` (instrument: `node tools/install/docs-tools/task-reconcile.mjs <ID> --repo <root>`, rungs R1–R4; R5 deliberately not executed — shared tree, and the specs cite Rust test binaries outside R5's sh/py/mjs/js/ts allowlist). Recommended verdicts summarized below.
4. **Gate-1 decision brief** at `assets/unreviewed-fork-brief.md` with the re-derived enumeration (167 files / 333 marker lines), both branches priced, and a recommendation (Branch clear with a one-file carve-out for TASK-EVAL-001).
5. **This evidence file.**

Files created (14): `implementation-evidence.md`, `assets/unreviewed-fork-brief.md`, `assets/reconcile/TASK-{MCP-003,MCP-005,MCP-006,MCP-007,MCP-008,OBS-001,OBS-003,OBS-005,OBS-007,OBS-008,OBS-009,APP-001}.md`. Files modified (252): the 251 specs + `task-lint.mjs`. Nothing else touched; sibling workers' concurrent edits (including `backlog-mutate.mjs` / `memory-append.mjs` in the same docs-tools directory) were left alone.

## Verification (verbatim outputs)

Corpus lint, new rule active, after normalization (`node tools/install/docs-tools/task-lint.mjs docs/tasks`):

```
corpus exit=2                      # pre-existing findings (FM-112 markers etc.), 3187 lines
FM-117 grep exit=1                 # ZERO FM-117 findings corpus-wide
FINDINGS IDENTICAL head-linter vs new-linter   # diff of full outputs: byte-identical
```

The HEAD linter never reads the `module` field (its only "module" occurrence is the word `modules/` in a path comment), so the normalization provably cannot change pre-existing lint results — confirmed empirically by the byte-identical finding sets above.

Fixture tests (temp copies under `/tmp/t139_fixture/docs/tasks/improvement/`, byte-copies of this task's spec with only the `module:` value flipped):

```
--- 990-conformant                 (module: improvement)
exit=0
--- 991-mixed-case                 (module: IMPROVEMENT)
error FM-117 /tmp/t139_fixture/docs/tasks/improvement/TASK-IMP-991-mixed-case/spec.md:6 module must be lowercase (got 'IMPROVEMENT')
exit=2
--- 992-folder-mismatch            (module: auth)
error FM-117 /tmp/t139_fixture/docs/tasks/improvement/TASK-IMP-992-folder-mismatch/spec.md:6 module must equal the containing docs/tasks/<module>/ folder name 'improvement' (got 'auth')
exit=2
--- 993-both                       (module: AUTH)
error FM-117 ...:6 module must be lowercase (got 'AUTH')
error FM-117 ...:6 module must equal the containing docs/tasks/<module>/ folder name 'improvement' (got 'AUTH')
exit=2
```

Existing dedicated suite (temp-dir-confined; runs the modified linter, builds the payload, scratch-installs it):

```
bash tools/install/tests/test_task_lint.sh
  ok t01 .. ok t09_optional_status_reason_enums
test_task_lint: pass=9 fail=0
suite exit=0
```

NOT run, deliberately: `scripts/tests/run_all.sh` and `scripts/tests/test_task_layout.sh` (both regenerate BACKLOG.md — forbidden for this worker; final pass owns regen + full verification).

## Measurements (2026-07-23, re-derived)

- Module case: 572 specs total; 251 with uppercase `module:` values (26 distinct value@folder pairs, all straight uppercase of the folder) → 0 after normalization; folder-mismatch 0 before and after.
- `# UNREVIEWED`, FM-112-equivalent scan (top-level frontmatter lines only): **167 non-draft files / 333 marker lines** (148 done, 12 implementing, 4 ready_to_implement, 2 closed, 1 on_hold). Values at stake: 167× `ai_authorship: generated_then_reviewed`; 166× `not_ai` + 1× `high` (TASK-EVAL-001, on_hold); 167× `client_visible: false`. Draft files with markers: 331 (kept, honest).
- Stuck `implementing`: exactly 12 — TASK-MCP-003/005/006/007/008 (2026-05-17), TASK-OBS-001/003/005/007/008/009 (2026-05-15), TASK-APP-001 (2026-07-14).

## Gate-2 recommended verdicts (operator decides; nothing applied)

| Task | Rungs (R1–R5) | Tool says | Dossier recommends | One-line rationale |
|---|---|---|---|---|
| TASK-MCP-003 | red/red/absent/red/– | route_back | route_back | Validator + CI gate committed under as-built `services/mcp-gateway/`; spec fails machine floor (pre-task@1 `## §N` grammar), never audited → modernize-audit-adopt. |
| TASK-MCP-005 | red/red/absent/red/– | route_back | route_back | PRM shipped inside `oauth/prm.rs`; claimed standalone module + 8 tests don't exist → re-spec-and-adopt, tests are the real delta. |
| TASK-MCP-006 | red/red/absent/red/– | route_back | route_back | Gating consolidated in `gating.rs`+`annotations.rs`; zero of 10 claimed tests → adopt + verify safety ACs. |
| TASK-MCP-007 | red/red/absent/red/– | route_back | route_back | Tasks primitive in `tasks.rs`/`tasks_pg.rs`; widest claim-vs-evidence gap (15 tests, concurrency ACs) → adopt + verify. |
| TASK-MCP-008 | red/red/absent/red/– | route_back | route_back | Elicitation in `elicitation.rs`/`elicitation_pg.rs`; claimed module/tests absent → adopt + verify. |
| TASK-OBS-001 | red/red/absent/red/– | route_back | route_back (spec superseded) | Tree contradicts the spec's architecture (custom collector/proxy, no Loki, no otel config) → re-spec or close; also flags in-tree `.live` token files. |
| TASK-OBS-003 | red/red/absent/red/– | route_back | route_back | SDK crate exists at `services/shared/cyberos-obs-sdk` (wrong claimed workspace); no tests → re-path, adopt, verify. |
| TASK-OBS-005 | red/red/absent/red/– | route_back | route_back | tracecontext/logging/exemplar sources exist; end-to-end correlation unproven; 2 deps themselves stuck → sequence re-entry. |
| TASK-OBS-007 | red/red/absent/red/– | route_back | route_back | Strongest OBS equivalence (14-file router + a test); wiring config + CUO-skill artefact missing → adopt + close gaps. |
| TASK-OBS-008 | red/red/absent/red/– | route_back | route_back | Four regime views + proof exist consolidated; exports/dashboard/tests absent → adopt + verify cross-tenant AC. |
| TASK-OBS-009 | red/red/absent/red/– | route_back | route_back | Manifest+signing+verifier binary committed; format doc/PDF/tests absent → adopt (optionally merge rework with OBS-008). |
| TASK-APP-001 | red/red/absent/absent/– | route_back | **resume** (with spec-hygiene conditions) | Nine days old, `apps/desktop` shipped releases 1.0.9/1.1.0 THIS WEEK; the red is process hygiene, not staleness — route-back would misstate live work. |

Tally if recommendations are accepted: 11 route_back · 1 resume · 0 on_hold.

## Deviations and discoveries (for the HITL reviewer)

1. **RUBRIC.md not edited** (spec 1.4 requires lint + rubric in the same change): outside this worker's ownership; the final pass MUST document FM-117 in `modules/skill/task-audit/RUBRIC.md` in the commit that carries the rule. Note when documenting: `FM-117` is already used by OTHER templates' contracts (`product-requirements-document`, `project-brief`) in their own per-template namespaces — within the task@1 rubric family FM-117 is the next free id.
2. **Census methodology resolved the 167-vs-170 discrepancy**: the audit's 167/148 is correct under the FM-112-equivalent scan; the authoring's 170/336/151 counted three `done` specs (TASK-IMP-084/108/117) that merely QUOTE the marker string in body prose. Five files total quote it without carrying markers (those three + TASK-IMP-139/140). A naive `grep -rl` bulk-clear would corrupt them — `test_corpus_hygiene.sh` (AC 2) must use the FM-112-equivalent scan.
3. **FM-112 fires on drafts too** (993 corpus findings include the 331 draft files), while spec §1.2 says drafts keep their markers legitimately. Whichever Gate-1 branch is chosen, the corpus lint stays red on drafts unless FM-112 gains status-awareness or §1.2's wording adjusts — an implementation decision for the gated half.
4. **BACKLOG.md regen expectation**: not run here (forbidden). Per the spec's guardrail the regenerator groups by folder, so BACKLOG.md SHOULD be byte-stable across the normalization; any consumer that prints the frontmatter `module` value (e.g. the status hub) may legitimately render lowercase where it previously rendered uppercase. The final pass verifies byte-stability (AC 6) after ITS regen.
5. **`deploy/obs/auth/{collector.token.live,tokens.live}`** are tracked in-tree and look like live token material where TASK-OBS-001's spec claimed a `tokens.example` — surfaced in that dossier; out of this task's scope but worth an operator look.
6. **R5 not executed** in any reconcile run: shared working tree (suite execution belongs to the final pass), and the stuck specs cite Rust test binaries that R5's repo-tracked sh/py/mjs/js/ts allowlist would refuse regardless. Cited-path existence was checked without execution; it is recorded per dossier.

## Open items for the final pass / HITL

1. Gate 1: operator selects a branch (brief + enumeration attached); record the dated verdict in this spec's `source_decisions` BEFORE any marker-touching commit (AC 1).
2. Gate 2: operator records 12 per-task verdicts (dossiers attached); apply only through the standard override path (AC 5).
3. RUBRIC.md FM-117 documentation in the same commit as the lint rule (AC 4).
4. `scripts/tests/test_corpus_hygiene.sh` (AC 2/3/4/6) — reuse the FM-112-equivalent census; include the five quote-only files as negative fixtures.
5. CHANGELOG entry (AC 7) — needs the branch choice, the count 251, rule id FM-117, and the Gate-2 tally.
6. Commits: none made by this worker; the 251-file normalization is sitting uncommitted, intended as the spec's "one mechanical commit" (§1.3), separate from the lint-rule commit per the ship-together discipline with RUBRIC.md.
