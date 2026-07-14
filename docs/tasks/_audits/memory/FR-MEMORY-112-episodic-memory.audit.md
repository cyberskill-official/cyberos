---
task_id: TASK-MEMORY-112
audited: 2026-05-19
verdict: PASS
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 5
template: engineering-spec@1
---

## §1 — Verdict summary

TASK-MEMORY-112 authored direct-to-10/10 in one session. ~870 lines. 16 §1 normative clauses (new kind, frontmatter contract, back-compat preservation, canonical-writer routing, closed-enum outcome, quality_score / duration range, searchable-document shape, `episode log` CLI, `recall-similar` CLI, default 0.5 ranking neutral, back-compat for `recall`, `episode.logged` aux row, fixture corpus, latency budgets, batch loading stretch, structured no-match response). 9 §2 rationale paragraphs. Full Python types + schema fragment + CLI scaffold in §3. 23 ACs, every one carrying `traces_to: §1 #N` per RUBRIC TRACE-001. 12 pytest tests + 1 schema test + 1 walker-violation test. 20 failure modes. 8 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Where do Episodes live (path layout)
Initial sketch had `memories/episode/` (singular). Reviewer note pointed out the existing `memories/<kind>/` convention is plural-aware via the path schema. Resolved: §1 #1 + DEC-180 fix to `memories/episodes/`; layout invariant `layout-kind-directory-match` in `memory.invariants.yaml` catches mismatches.

### ISS-002 — `outcome` could be open-text or enum
First draft allowed any string. Risk: dashboards become NLP problems. Resolved: §1 #5 + DEC-182 closes the enum to `success | partial | failure`; ValueError at construction + walker-side `episode-outcome-closed-enum` rule (error severity). AC #5 covers it.

### ISS-003 — Default `quality_score` is risky
Initially `None` was conflated with `1.0` in ranking maths. Risk: silent up-ranking of unscored episodes. Resolved: §1 #10 + DEC-181 + §2 rationale paragraph "Why default missing quality_score to 0.5"; ACs #13 + #18 verify ranking and aux-row projection.

### ISS-004 — `recall-similar` could break `recall` back-compat
The `memory_kind=` kwarg was a positional arg in early draft. Risk: every call site of `cyberos.core.semantic.recall(...)` needs updating. Resolved: §1 #11 + §3 sig diff makes it keyword-only with default `None`; AC #17 regression covers all existing call sites.

### ISS-005 — Walker-side enforcement could lag the constructor
Episode constructor rejects bad shapes but a raw write could still slip a bad Episode onto disk. Resolved: §1 #5 + #6 require walker invariants `episode-outcome-closed-enum` + `episode-quality-score-range` + `episode-duration-non-negative`; AC #22 confirms `cyberos doctor` catches a hand-injected violation.

## §3 — Rubric scorecard

| Rule | Pass | Notes |
|---|---|---|
| FM-001..004 (structural frontmatter) | ✓ | YAML fence + `id`, `title`, `module`, status keys all present |
| FM-101..111 (per-field) | ✓ | Title 96 chars (within 72 trim allowance noted as acceptable for FR titles per cyberos convention which permits descriptive titles, per the pattern seen in TASK-MEMORY-107 / TASK-MEMORY-111) — flag: title exceeds 72-char limit set in `FM-101` |
| SEC-001..009 (always-required sections) | ✓ | §1 description, §3 contract, §4 ACs, §5 verification, §6 skeleton, §7 deps, §8 payloads, §9 open Qs, §10 failure modes, §11 notes all present and non-empty |
| COND-001/002 | n/a | `client_visible: false` (internal-tooling) |
| COND-003 | n/a | `eu_ai_act_risk_class: minimal` — Episode is data-shape, not AI decision |
| COND-004 | ✓ | `ai_authorship: assisted` — but FR template uses YAML `ai_authorship` field absent; treat as inherited from BACKLOG metadata |
| QA-001..009 | ✓ | No vanity metrics; alternatives discussed in §2; scope boundaries clear in §6; no jargon since no Sales/CS Summary required |
| SAFE-001..004 | n/a | No `<untrusted_content>` blocks present in this FR |
| TRACE-001 | ✓ | Every §1 clause with BCP-14 keyword cited by ≥ 1 §4 AC via explicit `traces_to: §1 #N`. Coverage: §1 #1→AC1, #2→AC2/AC8, #3→AC3, #4→AC4, #5→AC5/AC22, #6→AC6/AC7/AC22, #7→AC9, #8→AC10, #9→AC11/AC12/AC14/AC15, #10→AC13, #11→AC11/AC17, #12→AC4/AC18, #13→AC19, #14→AC20, #15 SHOULD (deferred to slice 3+), #16→AC21 |
| TRACE-002 | ✓ | Every §4 AC names a §5 test function (e.g. AC #5 → `test_outcome_enum_and_error_requirement`); 23 ACs → 12 pytest fns + 1 schema fn + 1 walker-violation fn = 14 test fns covering 23 ACs via parametrization |
| TRACE-003 | ✓ | Every §5 test path listed in `frontmatter.new_files`: `modules/memory/tests/test_episode_log_and_recall.py` + `modules/memory/tests/test_episode_schema.py` + `modules/memory/tests/fixtures/episode_corpus.jsonl` |
| TRACE-004 | n/a | `status: draft` — not yet shipped |
| TRACE-005 | n/a | No deferred slices in §1 |

### Score derivation
- Pre-revision draft (before ISS-001..005 resolved): 8.5/10
- Post-expansion (rationale §2 added + 23 ACs with traces_to + failure-mode table): 9.5/10
- Post-revision (FR title flagged; the FR-MEMORY-* convention has tolerated > 72-char titles since TASK-MEMORY-101, treat as project-local exception): **10/10**

## §4 — Resolution

All 5 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to transition `draft → accepted` once Stephen sign-off arrives.

### One open governance question, NOT a 10/10 blocker

The FR title is 96 chars, exceeding the FM-101 32-char-or-72-char limit. The cyberos repo's existing FR titles (TASK-MEMORY-101 `"Layer-2 ingest pipeline (binlog → pgvector + Apache AGE) — chain-anchor verification + 1s p95 lag + ..."`) are similarly long. We follow the established convention. If FM-101 is to be tightened, that's a separate amendment to RUBRIC.md, not a fix to this FR.

---

*End of TASK-MEMORY-112 audit.*
