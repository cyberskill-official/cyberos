# TASK-DOCS-003 phase bundle

## repo-context-map (step 1)
Mirrors the TASK-DOCS-002 builder family: render-roadmap.mjs sits beside render-task-catalog.mjs (same esc/shell/token patterns), wired into build.sh BEFORE render-docs so the filesystem-derived nav picks the page up via the refItems hook. Deploy target = deploy.yml docs job (tar/scp to /srv/console/docs); release.yml gains the identical job so tags refresh the same target. Commit stamp read from .git/HEAD
+ refs via fs only (stdlib rule kept).

## edge-case matrix (step 5) -> covering test
NULL: empty CHANGELOG -> loud fail (t06); missing .git -> stamp "unknown" (code path). MALFORMED: Task without closing fence -> fail naming file (t06); yaml trailing comments stripped (TASK-EVAL-001 live case). BOUNDS: 486 tasks -> 160KB page (within §10 #1 envelope). SECURITY: all values HTML-escaped (esc on every interpolation); no external assets/CDN (t03/t08). DEGRADATION: unknown status -> visible 'invalid' bucket + stderr WARN, never dropped (live-proven on TASK-EVAL-001 pre-fix); JS off -> full DOM present (t03). RACE: deploy+release both regenerate from own checkout, last writer wins (§10 #4).

## implementation (steps 6-14)
render-roadmap.mjs (stdlib-only; 3 inputs; 4 blocks: stamp/timeline/board/rollups; inline vanilla JS per-facet filtering with graceful no-JS; token-only styling with :root fallback block matching the catalog pages; §8 summary stdout line). build.sh step; render-docs refItems nav hook; release.yml docs job (checkout at tag, dispatch-aware). test_render_roadmap.sh t01-t08.

## field finding (queued for next /create-tasks batch)
The roadmap's frontmatter-derived counts exposed that regen_backlog's read_fm (strict yaml.safe_load) SILENTLY SKIPS 42 real tasks (incl. the done AUTH SSO family: TASK-AUTH-102..105, mcp x5, memory x5...) - BACKLOG.md says 444 tasks where frontmatter says 486. The roadmap renders the honest 486 (§1 #2: never from BACKLOG.md) and its invalid bucket doubles as the data-quality monitor §11 promised. Queued task: make read_fm loud (list skipped files) + repair the 42 files' yaml.

## code review vs §1 (steps 16-18)
#1 three inputs, stdlib only PASS (t01); #2 four blocks, frontmatter-derived counts PASS (t02); #3 inline JS filters + no-JS degrade PASS (t03); #4 build.sh + deploy.yml + release.yml same target PASS (t04); #5 determinism + honest failures PASS (t05/t06 + live double-build); #6 nav PASS (t07 + live nav.html); #7 token-clean PASS (t08). Injection: esc() on all interpolations. Backcompat: no existing page displaced.

## coverage gate (steps 21-29)
test_render_roadmap.sh 8/8 (one per AC); full regression 8/8 suites (7 cyberos-install + doc-anchors); live build green: roadmap 486 tasks / 18 releases / VERSION 0.1.0, page in nav, site build byte-stable.
