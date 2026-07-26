# Status v3 preview

Standalone redesign preview of the CyberOS status page as **one tabless canvas**.
Open `index.html` in a browser (file:// works). Nothing here is wired into CyberOS:
the live page at `docs/status/`, the generator, hooks, and CI are all untouched.

## What it shows

One scrolling page, six bands, one selection state shared by all of them:

| Band | Answers |
| --- | --- |
| 01 Pulse | Where the project stands with zero clicks: shipped ring + burn-up spark, in flight, ready to start, blocked, commit traceability, next release. Attention cards surface computed findings (top blocker, critical path, stale drafts, rule violations). |
| 02 Roadmap | Program lanes (P0-P5, pre-1.0.0, module tracks) on the real calendar, cumulative shipped burn-up, every release plotted on its true date (clustered when same-day), today marker. Red tick under a release = it contains unlinked commits. |
| 03 System map | All 29 modules sized by scope with done/active/draft/hold donut rings, wired by the 82 real cross-module dependency edges derived from task frontmatter. Rail ranks: most open work, most blocked, largest scope. |
| 04 Flow | The task dependency graph, laid out by topological depth. Multi-parent edges are drawn, blocking edges are dashed red, the longest unfinished chain (critical path) is amber. Hover traces a task's full cone; scopes: Focus (auto-curated), Selection, All open. |
| 05 Releases | The rule, stated and measured. Release notes in exactly three types (Features / Fixes / Improvements), per-release commit coverage bars (linked / exempt / unlinked), coverage trend, and every unlinked commit flagged as a violation. |
| 06 Index | Every task, grouped by module, lazy-rendered. Click any row for the full record (drawer with deps, unblocks, citing releases and commits, spec link). |

Interactions: `/` to search (tasks, modules, releases), Esc to clear, hover to trace,
click to pin. Selecting a module, phase, or release updates bands 02, 03, 04, 05, and 06
in place and never scrolls the page; clicking the same item a second time (or the
selection chip in the top bar) jumps to its home band. Selections are deep-linkable
(`#t/TASK-AI-009`, `#m/memory`, `#r/1.10.0`) and deep links do scroll on arrival.
Every commit hash, release pill, PR reference, and the snapshot branch link straight
to GitHub. Theme toggle: brand paper (default) and night ops; `?theme=night` for a
shareable override. A "snapshot Nd old" chip appears when the embedded data is stale.

## Data

Real repo data, snapshotted at build time by `build.mjs` (preview-only script, not
wired into any hook or workflow). Inputs: task frontmatter under `docs/tasks/`,
`CHANGELOG.md`, `git log`, `VERSION`, `modules/manifest.yaml`. Regenerate with:

```sh
node docs/status-v3-preview/build.mjs
```

Release note types are mapped from Keep-a-Changelog headings:
Added -> Features, Fixed -> Fixes, everything else (Changed, Security, Removed, ...) -> Improvements.

Commit classification, in order: `linked` via a canonical TASK id in subject or body;
`linked` via shorthand (`IMP-122`, `TEN-002/004`) when it resolves to a task that
exists; `linked` via the reviewed backfill ledger (`commit-links.yaml` beside this
README — for commits whose link never made it into the message; history is never
rewritten); `exempt` (release plumbing: `chore(release)`, bundle rebuilds, merges,
`[skip ci]`); else `unlinked` = a violation. The exempt list is policy and
deliberately visible on the page.

The repo rolled its version numbers back to 0.1.0 on 2026-07-12, so 1.x version
strings exist in two epochs. The page buckets commits by marker occurrence and folds
the first epoch into one history entry; headline coverage counts the current epoch.

## Enforcement (implemented)

The rule is now mechanical, shared by one script:

1. `scripts/check_task_link.sh` — single implementation. `--msg` for the hook,
   `--range` for CI. Exempt list identical to the page's.
2. `.githooks/commit-msg` — runs it on every commit. Advisory by default (loud
   warning); `CYBEROS_REQUIRE_TASK_LINK=1` or `CYBEROS_STRICT_COMMITS=1` blocks
   locally.
3. `.github/workflows/traceability.yml` — hard gate. Fails any PR or push to main
   containing a non-exempt commit newer than the cutoff
   (`a7e0e212`, 2026-07-26) without a canonical TASK id. Shorthand does not satisfy
   the gate going forward: one grep-able grammar from the cutoff on.

Historical fixes go through the ledger, never `git rebase`.

## If approved (page integration)

1. Fold `build.mjs` extraction into `tools/docs-site/render-status-hub.mjs` as a
   versioned `status-feed` payload (add `depends_on`/`blocks`/`shipped` and commit
   coverage to the corpus; fail the build on dangling task references).
2. Replace the page shell with this one; keep the per-task spec chunks and noscript
   table from the current generator.
3. Freshness: the live page regenerates wherever render-status-hub runs today
   (pre-commit docs trigger, deploy workflow, post-version bump). Extend the
   pre-commit trigger from docs-paths-only to every commit so commit coverage can
   never lag, and move `commit-links.yaml` to `docs/tasks/_state/`.
4. Optional hard stop: block release tagging while unlinked > 0 in the release range.
