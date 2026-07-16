---
artefact: edge-case-matrix@1
task_id: TASK-IMP-089
total_rows: 10
created: 2026-07-17
verdict: pass (edge-case-matrix-audit: every category >=1 row, covered-by names real test functions, SECURITY rows point at code+test, DEGRADATION rows carry detection+recovery)
---
# Edge-case matrix - TASK-IMP-089

All test functions live in scripts/tests/test_template_schema.sh; `shape_why` is the shared oracle all three t08 arms call.

| # | category | trigger | expected behavior | covered by |
|---|---|---|---|---|
| 1 | null/empty | TASK-TEMPLATE.md missing entirely | loud fail naming why the file matters (install quickstart cps it), never a vacuous ok - same guard idiom as t06 | t08_single_out_of_scope_home (`[ -f ]` guard) |
| 2 | null/empty | renumber drops the invariants BODY (heading survives, content gone) | `invariants-body-missing` token - "content unchanged" (spec 1.2) has a probe, not just a heading count | t08_single_out_of_scope_home (grep 'must never be made green by weakening') |
| 3 | bounds | duplicate reintroduced at a DIFFERENT number (`## 6. Out of scope`) or unnumbered (`## Out of scope / non-goals`) | oracle regex `^## +([0-9]+\. *)?out of scope` (case-insensitive) flags any H2 respelling; a renumbered duplicate is still a duplicate | shape_why in all three arms |
| 4 | bounds | a heading left or added at `## 5.` (renumber half-done, or a sixth section appended at the old number) | `stray-##5-heading` token - the renumber is complete only when NOTHING sits at 5 | t08_single_out_of_scope_home via shape_why |
| 5 | malformed | fixture: the retired `## 4. Out of scope / non-goals` block re-added verbatim above the invariants (the exact old shape) | shape_why names `duplicate-out-of-scope-H2` specifically; the arm demands THAT token, so an oracle failing for an unrelated reason does not count as detection | t08_duplicate_reintroduction_fails |
| 6 | malformed | invariants heading duplicated, reworded, or drifted (exact-line count != 1) | `invariants-not-at-##4` token (grep -c == 1 against the full literal heading) | shape_why in all three arms |
| 7 | concurrency/order | fixture write and scratch payload build share one TMP across arms; repeated suite runs | `mktemp -d` per run + `trap ... EXIT` cleanup; arms use disjoint subpaths ($TMP/TASK-TEMPLATE.reintroduced.md vs $TMP/payload, which build.sh rm -rf's itself) - reruns never see stale state | suite harness line + t08_payload_carries_shape |
| 8 | SECURITY | template is prompt text handed to every new repo (install.sh:742): a payload copy diverging from source ships consumers a shape the gate never approved (the t06 incident class: `class: product` reached 23 repos) | `cmp -s` byte-parity source<->payload in the scratch build; divergence -> `payload-copy-diverges-from-source` | t08_payload_carries_shape |
| 9 | DEGRADATION | scratch build.sh fails (VERSION missing/invalid, cp error) | detection: exit != 0 -> explicit "scratch build.sh failed", never a skipped ok; recovery: build.sh's own up-front VERSION validation prints the cause on stderr, rerun after fixing | t08_payload_carries_shape (guarded build call) |
| 10 | DEGRADATION | payload builds but carries no cuo/templates/TASK-TEMPLATE.md (vendor line lost) | detection: named-path fail per t07's lesson (a check that matches nothing must be distinguishable from absence); recovery: build.sh:36 `cp -R` line restore + rebuild | t08_payload_carries_shape (`[ -f "$v" ]` arm) |

Documented-by-design (spec §3): existing specs with the old two-home shape stay untouched and rubric-valid - the corpus is never in t08's scope; the rubric itself needs no change because section 4 was never a rule (SEC-006/QA-006 point at the Scope home). The per-type templates carry the PRD half and never had a section 4 (spec Out-of-scope bullet 2) - t02-t05 keep gating them unchanged.
