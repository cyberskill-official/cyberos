---
task_id: TASK-DOCS-001
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

TASK-DOCS-001 expanded from 195 lines to ~720. Added 7 §1 clauses (#1 JSON source of truth migration; #6 metadata field enumeration per card type; #7 stable IDs/anchors; #10 watch mode; #11 CI drift gate; #12 build report; expanded #4 with x-cloak mechanism). 8 §2 rationale paragraphs. Full Node.js render script + Handlebars templates + data-extract migration + Alpine init modification + CI workflow in §3. 15 ACs. Mixed verification (bash + Node tests). 17 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Source-of-truth ambiguity (inline JS vs external JSON)
First-pass §6 mentioned "alternative: move data arrays into data/*.json files" without committing. Source-of-truth ambiguity = drift inevitable. Resolved: §1 #1 commits to JSON; data-extract.mjs migrates one-time; Alpine reads JSON.

### ISS-002 — Per-card anchor support unspecified
Crawlers + screenshot tools need `#NFR-PERF-01` URL fragments. Resolved: §1 #7 + AC #7 + Handlebars template includes `id="{{id}}"`.

### ISS-003 — Per-card type metadata fields unspecified
First-pass templates only sketched NFR; RSK + task not fully specified. Resolved: §1 #6 enumerates fields per card type; §3 NFR template shown; analogous patterns for RSK + task.

### ISS-004 — Watch mode for local dev missing
Local iteration requires manual rebuild on every JSON change. Resolved: §1 #10 + `--watch` flag + AC #10.

### ISS-005 — CI drift gate not specified
JSON-HTML drift inevitable without enforcement. Resolved: §1 #11 + docs-prerender-gate.yml workflow + AC #6 + §5 git diff check.

### ISS-006 — Hydration flash mechanism unspecified
First-pass mentioned "hidden via x-cloak or equivalent" without specifics. Resolved: §1 #4 + CSS `[x-cloak] { display: block }` initial; `.hydrated [x-cloak] { display: none }` after Alpine ready; AC #3.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of TASK-DOCS-001 audit.*
