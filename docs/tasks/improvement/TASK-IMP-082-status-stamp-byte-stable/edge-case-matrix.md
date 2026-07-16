---
artefact: edge-case-matrix@1
task_id: TASK-IMP-082
total_rows: 10
created: 2026-07-16
verdict: pass (edge-case-matrix-audit: every category >=1 row, covered-by names real test functions, SECURITY rows point at code+test, DEGRADATION rows carry detection+recovery)
---
# Edge-case matrix - TASK-IMP-082

All test functions live in scripts/tests/test_render_stamp.sh unless a suite path is named.

| # | category | trigger | expected behavior | covered by |
|---|---|---|---|---|
| 1 | null/empty | empty corpus: 0 tasks, no CHANGELOG (the lenient fresh-install shape) | render succeeds (lenient), fingerprint over VERSION alone, two renders byte-identical | t02_double_render_stable (empty-corpus half) |
| 2 | null/empty | fully empty input set (no tasks, no CHANGELOG, no VERSION) | stamp is exactly fp-e3b0c44298fc = sha256 of zero bytes - proves no hidden input (path, clock, env) leaks into the hash | t02_double_render_stable (final assert) |
| 3 | null/empty | CYBEROS_COMMIT set to empty string | falls through to the fingerprint (the `\|\|` default, preserved from the old code) | t05_env_pin_wins (second half) |
| 4 | bounds | truncation/shape: stamp must be `fp-` + exactly 12 lowercase hex, and exactly the first 12 of the full sha256 | header stamp equals an independently recomputed `fp-$(sha256 of the ordered inputs \| cut -c1-12)` | t01_fingerprint_on_all_surfaces (oracle recompute) |
| 5 | bounds | very large corpus (512 real specs, 7.6 MB of chunks) | streaming per-file hash updates, no concat buffer; render + double-render stable | recorded ops check (real-corpus double render byte-identical, fp-511e67443359) + inspection render-status-hub.mjs:304 |
| 6 | malformed | spec.md with broken frontmatter | strict: renderer dies loudly naming the file (unchanged contract); lenient: file is still a discovered render input, so the stamp stays deterministic and moves only when bytes move | tools/docs-site/tests/test_render_status_hub.sh::t09_nojs_and_honest_failures (strict) + inspection render-status-hub.mjs:128 (push before parse) |
| 7 | concurrency/order | path sort consulting the runtime locale (LC_ALL=C vs C.UTF-8) would drift the fingerprint between machines | comparator is Buffer.compare - bytewise, locale-blind; whole output tree byte-identical across locales | t02_double_render_stable (cross-locale diff -r) |
| 8 | concurrency/order | render -> commit the page -> render (the HEAD chase, an ordering hazard between renders and commits) | byte-identical: the page's own bytes and repo position are not inputs | t03_commit_chase_ended |
| 9 | SECURITY | CYBEROS_COMMIT carries hostile bytes (env is caller-controlled) into three HTML/JSON surfaces | value flows through esc() on meta/footer (render-status-hub.mjs:454,463) and JSON.stringify + < escaping for cs-data (:415) - pre-existing sinks, unchanged; hash is node:crypto sha256, not homegrown (:302) | t05_env_pin_wins (pin traverses all three escaped surfaces) + inspection |
| 10 | DEGRADATION | rendering outside any git checkout, or on a box where spawning git would fail | detection: tripwire `git` first on PATH logs any invocation and exits 99; recovery: none needed - default path never touches git, render succeeds with an fp- stamp, and a git-checkout copy of the same corpus renders byte-identically | t06_no_git_needed |

Documented-by-design (spec §3): a CRLF or trailing-whitespace edit to a task file changes bytes, so it changes the stamp - correct by definition, content is the contract. t04_corpus_edit_changes_once pins the general form (any byte edit moves the stamp exactly once, then stability returns).
