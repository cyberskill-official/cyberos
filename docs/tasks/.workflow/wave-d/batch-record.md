# Wave D batch ship record (2026-07-12)

Operator PLAN approval (in-chat): architecture A "HTML faces, markdown bones"; command deck + 3 tabs;
published site + local build; "approve PLAN + start shipping, ship in batch, non-stop, only pause for
manual work". That standing verdict is the recorded human approval for each task's two gates below.

Shipped (all 5, single batch leg, per-task commits):
1. TASK-DOCS-004 - folder-per-task (491 migrated), corpus 100% strict-yaml (42 repaired/63 lines),
   loud regen, walkers + checker updated, 6/6 (+4) AC.
2. TASK-TPL-001 - templates module: CDS tokens/glass vendored @ 7231866d with provenance,
   template@1 contract, 3 shells, 4/4 AC.
3. TASK-DOCS-005 - 491 self-contained CDS task pages (media, cross-links, audit blocks), catalog
   links, 6/6 AC. Template fill() html-slot bug found+fixed by the suite.
4. TASK-DOCS-006 - status hub (deck + Roadmap|Backlog|Changelog, hash-routed, JS-free fallback),
   roadmap superseded with permanent stub, 6/6 + legacy 7/7 AC.
5. TASK-SKILL-120 - authoring wiring (command/author/audit/ship/init grammar), t07-t10.

Field repairs folded in: TASK-PLUGIN-003 + TASK-TEN-002 new_files omissions; 28 stale module READMEs
exempted with reasons; 2 write-protected memory test files chmod'd; doc-anchor checker gained
corpus-planned rule + status-aware severity.

Verification at close: 13/13 repo suites, ship_manifest 8/8, site build (491 pages + hub) green,
payload build green, backlog == roadmap == frontmatter (491).
