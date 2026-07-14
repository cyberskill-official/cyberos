# Feature request to task: rename impact analysis

Status: draft for decision. Not an ADR yet.
Author: analysis pass, 2026-07-14.
Scope: `feature-request(s)` -> `task(s)` globally, plus a `type` discriminator (feature | bug) with per-type templates, rubrics and gates.

---

## 0. Executive summary

The string swap is the smallest part of this. Three things make it hard:

1. `task` is already a distinct, load-bearing concept in CyberOS. Reusing the word creates a four-way homonym.
2. `FR-<MODULE>-<NNN>` IDs are provenance markers embedded across the entire codebase (29,640 citations, 563 distinct IDs, 3,204 of them outside `docs/`). They are not a docs convention, they are the traceability spine.
3. The FR schema is already drifted three ways (CONTRACT.md vs STATUS-REFERENCE.md vs the specs actually on disk). A rename freezes that drift into the new name unless it is reconciled first.

Recommendation: sequence this as five phases, do not run a codemod as step one.

---

## 1. Measured blast radius

| Signal | Count |
|---|---|
| Files containing `feature[-_ ]request` (any case) | 731 |
| Occurrences of `feature[-_ ]request` | 2,817 |
| Paths with `feature-request` in the filename | 978 |
| `FR-<MOD>-<NNN>` ID citations, whole repo | 29,640 |
| Distinct FR IDs | 563 |
| FR ID citations outside `docs/` (source, tests, migrations, CI) | 3,204 |
| Generated per-FR status data files (`docs/status/data/fr/*.js`) | 507 |
| NFR docs referencing FR IDs | 311 |
| Repo commits (git blame / history surface) | 584 |

Non-doc dirs that cite FR IDs in source comments: `modules/skill` (296), `services/ai-gateway` (122), `services/auth` (104), `modules/cuo` (58), `services/memory` (57), `services/mcp-gateway` (53), `modules/memory` (46), `services/email` (45), `services/chat` (34), `services/proj` (33), `.github/workflows` (21), plus a dozen more.

---

## 2. Blocker A: `task` is already taken, three times

This is the finding that should change your plan.

### 2.1 `task@1` is an existing contract with a different meaning

`modules/skill/contracts/task/` exists today. It defines `task@1` as:

> a comprehensive, addressable, assignable unit of work embedded **inside** a `feature_request@1`

- Task ID format: `FR-NNN-T-MM` (the third task of FR-007 is `FR-007-T-03`).
- Lifecycle: `draft -> ready -> in_progress -> done | blocked`.
- Fields: `sizing`, `assignable_to: [human, ai-agent]`, `agent_profile`, `estimated_tokens`, `acceptance_test`, `parallelisable`.
- Consumed by `modules/skill/runners/fr_with_tasks.py` (skill `cuo/cpo/fr-with-tasks`).
- Referenced by `modules/memory/runtime/migrations/README.md`.

So today: an FR **contains** tasks. If FR becomes "task", a task contains tasks.

### 2.2 FR frontmatter already carries `sub_tasks:`

44 of the 80 FR specs sampled carry a `sub_tasks:` list of hour-estimated work items. Same concept, second spelling.

### 2.3 CAF has its own Task table

`tools/caf/` uses "Task table" with IDs like `L1-T1` (loop 1, task 1), 37 occurrences, embedded in 63 golden eval fixtures under `tools/caf/core/evals/fixtures/*/docs/BACKLOG.md`. These fixtures also contain a file literally named `BACKLOG.md` that has nothing to do with the FR backlog.

### 2.4 awh eval harness uses `tasks:`

`modules/skill/.awh/goldenset.yaml` uses `tasks:` as its top-level key for eval tasks. Repo-wide there are 349 `tasks:` occurrences.

### 2.5 And the agent host itself

Claude Code exposes a Task tool, TaskCreate, TaskUpdate, and subagent tasks. Skill descriptions are matched against user prose. "Task" is the single most common noun in agent prompts.

### Consequence

Naming the backlog atom `task` means the word carries four unrelated meanings inside one system, one of which the agent runtime also owns. Something has to give. Options are in §7.

---

## 3. Blocker B: FR IDs are the traceability spine, not a naming convention

563 distinct FR IDs are cited 29,640 times. 3,204 of those citations are in code: Rust source comments, Python docstrings, SQL migration headers, CI workflow files, shell scripts, test names.

Examples of what breaks if IDs are rewritten:

- `git blame` and `git log -S FR-AUTH-111` stop finding the reason a line exists.
- Every emitted audit artefact (`docs/feature-requests/_audits/`, `.workflow/*/code-review.md`, `.workflow/*/coverage-gate.md`) cites FR IDs as evidence of what was audited. Rewriting them falsifies the record.
- The memory audit chain (AGENTS.md §3.3, §6.5) is append-only and immutable. Rows already on-chain cite `docs/feature-requests/...` paths and FR IDs. Those rows cannot be rewritten. After a path rename, the chain permanently references paths that no longer exist, and any walker invariant that resolves a path could flip the store to `FROZEN_RECOVERABLE`.
- `scripts/awh_goldenset_from_fr.py` builds the eval goldenset from FR specs. `.awh/gate.sh` runs `awh eval ... --max-regression 0.0`. Changing FR identity changes the goldenset, and the gate blocks on any regression.

Conclusion: rewriting the 563 historical IDs is a false economy. Freeze them.

---

## 4. Blocker C: three-way schema drift, today, before any rename

The FR schema disagrees with itself in three places.

`modules/skill/contracts/feature-request/CONTRACT.md` (FM-101..111) declares:

```
title, author, department, status (draft|in_review|approved|in_progress|shipped|closed),
priority (p0|p1|p2|p3), created_at, ai_authorship, feature_type,
eu_ai_act_risk_class, target_release, client_visible, template
```

`modules/skill/contracts/feature-request/STATUS-REFERENCE.md` declares a different, 10-value status enum:

```
draft, ready_to_implement, implementing, ready_to_review, reviewing,
ready_to_test, testing, done, on_hold, closed
```

The FR specs actually on disk carry none of the CONTRACT.md field names. Real frontmatter:

```
id, title, module, priority (MUST), status, verify, phase, milestone, slice,
owner, created, shipped, memory_chain_hash, related_frs, depends_on, blocks,
source_pages, source_decisions, language, service, new_files, modified_files,
allowed_tools, disallowed_tools, sub_tasks, class, refs
```

`class:` (product | improvement) appears on only 30 of 80 sampled specs.

If you rename before reconciling, you carry three incompatible schemas forward under a new name and lose the chance to fix it under cover of a breaking change.

---

## 5. Blocker D: the BACKLOG reader does not match the BACKLOG

`modules/cuo/cuo/core/backlog_reader.py` parses a markdown **table**:

```python
_FR_ROW_RE = re.compile(r"^\|\s*\*{0,2}(?P<fr_id>FR-[A-Z]+-\d+)\*{0,2}\s*\|" ...)
# expects: | FR-ID | Title | Pri | Status | Depends on | Effort |
```

`docs/feature-requests/BACKLOG.md` has **0 table rows and 357 bullet rows**:

```
- [draft] FR-AI-104-vn-provider-integration - AI VN provider integration ...
```

The reader returns zero rows against the live backlog. `backlog-state-update-author`'s `line_number` / `old_line` optimistic-concurrency pre-image is written against the table shape (BSU rule family). This is broken today, independent of the rename, and the rename will be blamed for it if not fixed first.

---

## 6. Risk register

| # | Risk | Evidence | Mitigation |
|---|---|---|---|
| R1 | Classifier precision collapse | `modules/cuo/cuo/trigger_tests.py` + per-skill `acceptance/TRIGGER_TESTS.md` assert the CUO supervisor routes phrasings to skills. `description_format_check.py` and `services/skill-broker/src/frontmatter/description_validator.rs` enforce 80-1024 chars, >=2 quoted trigger phrases, >=1 verb stem, no XML-tag shapes. A `task-author` skill whose description quotes "create a task" will fire on generic agent prose. | Never write bare "task" in a skill description. Always "CyberOS task" or "backlog task". Re-author all TRIGGER_TESTS.md. Add negative triggers for the host Task tool. Re-baseline `.awh/eval-baseline.json` and expect the `--max-regression 0.0` gate to block until it is regenerated. |
| R2 | Public URL break | `tools/docs-site/render-fr-pages.mjs` emits `/frs/<module>/<stem>/index.html` and cross-links `../../${module}/${stem}/index.html`. `docs/status/` has 507 per-FR data files. Deployed via `vercel.json`. | Emit both `/frs/` and `/tasks/` for one release, with 301s from `/frs/`. Keep the FR-ID-keyed data filenames stable (see §7.2). |
| R3 | Stale symlinks in every installed repo | `.gitignore` managed block lists `.claude/skills/ship-feature-requests`, and the same under `.grok`, `.codex`, `.opencode`, `.commandcode`. Five agent-tool dirs, all symlinked to `.cyberos/plugin/skills`. | `cyberos install` must remove the five old symlinks by name before creating new ones, and rewrite the managed gitignore block. Add a test in `tools/cyberos-init/tests/`. |
| R4 | Vendored payload drift | `tools/cyberos-init/build.sh` hardcodes `ship-feature-requests.md`, `contracts/feature-request/STATUS-REFERENCE.md`, and a skill allowlist naming `feature-request-author` / `feature-request-audit`. `.pre-commit-hooks/cyberos-payload-build.sh` has a trigger regex that the comment says is mirrored in `check-version-sync.sh`. `.cyberos/` is fully gitignored (0 tracked files) and regenerated. | Update build.sh, both trigger regexes, and `tools/cyberos-init/lib/fr-migrate.sh` in the same commit. Add a check that fails if the two regexes diverge. |
| R5 | CAF fixture corruption | 63 golden fixtures at `tools/caf/core/evals/fixtures/*/docs/BACKLOG.md`, using CAF's own unrelated BACKLOG + Task-table shape. | Hard-exclude `tools/caf/**` from every codemod pass. Add it to the codemod's deny-list and assert zero diff under `tools/caf/` in CI. |
| R6 | Audit-chain path dangling | Memory protocol §3.3 / §6.5: rows are immutable, append-only, no reordering, no deletion. On-chain rows cite `docs/feature-requests/...`. | Emit one `memory.path_rename_epoch` aux row recording `{old_prefix, new_prefix, at_seq}`, and teach the walker to resolve pre-epoch paths through it. Do not rewrite rows. |
| R7 | Evidence falsification | `docs/feature-requests/_archive/`, `_audits/`, `.workflow/*/`, and `CHANGELOG.md` are records of what happened, not live spec. | Never codemod them. Freeze in place or `git mv` the directory without touching file contents. See D3 in §8. |
| R8 | FR-specific scripts break | `scripts/migrate_fr_layout.py`, `repair_fr_yaml.py`, `rebaseline_fr_status.py`, `migrate_improvement_to_fr.py`, `awh_goldenset_from_fr.py`. Two of these are vendored into the payload as `docs-tools/`. | Rename with the codemod, but re-run each against a fixture repo before shipping the payload. |
| R9 | Path filters in hooks | `.pre-commit-hooks/docs-site-build.sh` regex includes `docs/feature-requests/`. `.pre-commit-hooks/no-real-pii-in-corpus.sh` also matches. | Grep all hook path regexes; they are easy to miss because they are inside single-quoted shell strings. |
| R10 | `proj` module namespace | `services/proj` has its own `issues` and `issue_links` tables. If you later name the bug type "issue", it collides. | Reserve "issue" for the proj module. Use `type: bug` on the task, not "issue". |

---

## 7. Recommendations

### 7.1 Free the word before you use it

`task@1` (the sub-unit inside an FR) must be renamed before `task` can mean the backlog atom. Candidates, in order of preference:

- `work-package@1` — unambiguous, standard PM vocabulary, no collision anywhere in the repo.
- `step@1` — short, but collides with `ship-manifest` step semantics (`steps[].status`).
- `subtask@1` — clear, but keeps "task" as a substring, so grep and classifier ambiguity survive.

Recommendation: `work-package@1`, with `sub_tasks:` in FR frontmatter renamed to `work_packages:` in the same pass. This is ~11 `task@1` references, 1 runner, 1 contract dir, and 44 frontmatter blocks. Small and contained.

Then, and only then, `feature_request@1` -> `task@1`, and the ID sub-format becomes `TASK-NNN-WP-MM`.

### 7.2 Freeze the ID space

Do not rewrite 563 IDs across 29,640 sites. Instead:

- Readers accept `^(FR|TASK)-[A-Z]+-\d+$`.
- Writers emit `TASK-` only, from the rename epoch forward.
- The 563 `FR-*` IDs stay valid forever as a legacy prefix. `related_frs:` / `depends_on:` / `blocks:` values are untouched.
- Rename the frontmatter *field* `related_frs:` -> `related_tasks:` (with a reader alias for one release), but not the values.
- Keep `docs/status/data/fr/FR-XXX.js` filenames stable; add `TASK-XXX.js` alongside. The directory name `fr/` can stay: it is a generated cache, not a contract.

This preserves git blame, the audit chain, every code comment, every NFR cross-reference, and the awh goldenset, at the cost of a dual-prefix regex forever. That trade is correct.

### 7.3 The type discriminator is the real feature

Add to the task contract:

```yaml
type: feature | bug        # required, closed enum, extensible to chore | spike | refactor
class: product | improvement   # existing, orthogonal, keep
```

Structure the contract so a third type is cheap:

```
modules/skill/contracts/task/
  CONTRACT.md
  STATUS-REFERENCE.md
  templates/feature.md
  templates/bug.md
  rubrics/common.md
  rubrics/feature.md
  rubrics/bug.md
```

`task-author` dispatches on `type` and never hardcodes the list.

Keep rubric rule IDs (`FM-*`, `SEC-*`, `COND-*`, `QA-*`, `SAFE-*`, `TRACE-*`, `BSU-*`) unchanged. Renaming them invalidates every `.audit.md` already on disk. Add a new `BUG-*` family instead.

### 7.4 What a bug task must carry that a feature task does not

| Field / section | Why it is checkable |
|---|---|
| Deterministic reproduction steps + environment | A gate can run them. |
| First-bad-commit (when known) | `git bisect` output is evidence. |
| Expected vs observed, stated separately | Forces the author to distinguish them. |
| Severity + blast radius (distinct from `priority`) | Drives the skip-phases decision below. |
| Root-cause statement | Rubric rule: must not be a restatement of the symptom. |
| Regression test that fails at the pre-fix commit and passes after | This is the bug analogue of the edge-case matrix, and it is machine-verifiable: the gate checks out the parent commit and asserts the named test goes red. |
| Link to a postmortem when severity is high | Routes into the existing `postmortem-author` skill. |

Gate changes:

- `coverage-gate-audit` gains a `REGRESSION-*` rule family for `type: bug`: the cited regression test must be red at `HEAD~` and green at `HEAD`.
- `edge-case-matrix-author` relaxes the `total_rows >= 8` floor for bugs, and scopes the matrix to the cause's neighbourhood rather than the whole feature.
- `ship-tasks` skips ADR / architectural-spike / SDD phases for `type: bug` unless `repo-context-map` reports the fix crosses a module boundary.

### 7.5 Status enum additions

The 10-value enum has no home for bug-specific terminal states. Add:

- `cannot_reproduce` (off-ramp, bug only)
- `duplicate` (off-ramp, both types)

`closed` already covers won't-do. Both additions touch `STATUS-REFERENCE.md` §1, `backlog_state_update_rubric`, and the status page legend. Do them in the same change as the type discriminator, not later.

### 7.6 A free win you already half-built

`STATUS-REFERENCE.md` §1.3 already anticipates this:

> Future hook — Issue Request artefact (TBD): when an FR is routed back to `ready_to_implement` from a downstream stage, the system will eventually auto-spawn an Issue Request (a new artefact type, distinct from FR) carrying the failure reason, the failing test name(s), and the reverting commit hash.

That artefact **is** a `type: bug` task. Wire the route-back path to auto-draft one.

Second wire-up: `services/obs-router/src/cuo_triage.rs` and `modules/cuo/cuo/triage_server.py` already route production alerts into CUO triage. An alert that survives triage should emit a `type: bug` task with the reproduction pre-filled from the trace. That is the bug intake path, and it already exists in skeleton.

---

## 8. Proposed sequencing

Do not run a codemod as step one. Five phases, each independently shippable.

### Phase 0 — reconcile, do not rename

- Fix the three-way schema drift (§4). Pick the on-disk schema as truth, rewrite `CONTRACT.md` to match, delete the dead FM-101..111 field set or migrate to it deliberately.
- Fix `backlog_reader.py` vs the bullet-format BACKLOG (§5). Either regenerate BACKLOG as a table, or rewrite the reader for bullets and update the BSU concurrency pre-image rules to match.
- Land both under the existing FR name. No user-visible change.

Ships value even if you abandon the rename.

### Phase 1 — free the word

- `task@1` -> `work-package@1`. Rename `contracts/task/` -> `contracts/work-package/`, `runners/fr_with_tasks.py` -> `runners/fr_with_work_packages.py`, `sub_tasks:` -> `work_packages:`.
- Nothing else changes. `task` is now an unused identifier in CyberOS.

### Phase 2 — introduce the type discriminator, still under the FR name

- Add `type: feature | bug`, default `feature`, backfill all 563 specs.
- Ship `templates/bug.md` + `rubrics/bug.md` + the `REGRESSION-*` gate family.
- Add `cannot_reproduce` and `duplicate` statuses.
- Prove it: file one real bug through the full lifecycle.

At this point you have the actual capability you want. The rename is now cosmetic and reversible.

### Phase 3 — the rename

Codemod, ordered longest-identifier-first so shorter patterns cannot eat longer ones:

1. `feature_request@1` -> `task@1`
2. `feature-request-author` -> `task-author`; `feature-request-audit` -> `task-audit`
3. `ship-feature-requests` -> `ship-tasks`; `create-feature-requests` -> `create-tasks`
4. `contracts/feature-request/` -> `contracts/task/`
5. `docs/feature-requests/` -> `docs/tasks/` (via `git mv`, for rename detection)
6. `fr-manifest@2` -> `task-manifest@1`
7. `audit_rubric@2.0` -> keep the name, bump to `@3.0`
8. prose: `Feature Request` / `feature request` / `FRs` -> `task` / `tasks`
9. bare `FR` abbreviation: **manual pass only**, word-boundary, and never inside an `FR-<MOD>-<NNN>` ID

Codemod deny-list (assert zero diff in CI):

```
tools/caf/**                       # separate BACKLOG/Task vocabulary, golden fixtures
.cyberos/**                        # gitignored, regenerated by build.sh
dist/**                            # build output
docs/status/data/**                # generated cache
CHANGELOG.md                       # historical record
docs/feature-requests/_archive/**  # evidence
docs/feature-requests/_audits/**   # evidence
docs/feature-requests/.workflow/** # evidence
```

Freeze the backlog during the cut: no task may be mid-flight, because BSU optimistic concurrency pre-images are invalidated by any BACKLOG rewrite. Alternatively add a `schema_version` to the BACKLOG frontmatter that the concurrency check reads, and fail loudly on mismatch.

### Phase 4 — ship it

- Alias `/create-feature-requests` and `/ship-feature-requests` to the new commands for one release, printing a deprecation notice. Then delete.
- `cyberos install` detects `docs/feature-requests/`, offers the migration, removes the five stale agent symlinks, rewrites the managed gitignore block.
- Emit the `memory.path_rename_epoch` audit row.
- Regenerate `.awh/eval-baseline.json` and re-author every `TRIGGER_TESTS.md`.
- Emit `/frs/` -> `/tasks/` 301s on the docs site for one release.

Dogfood: run `/create-feature-requests` one last time to author the tasks that retire feature requests.

---

## 9. Decisions taken (2026-07-14)

| # | Decision | Chosen | Rationale given |
|---|---|---|---|
| D1 | `task@1` collision | rename existing `task@1` -> `subtask@1` | parent/child reading is obvious |
| D2 | ID scheme | rewrite all 563 IDs, `FR-*` -> `TASK-*` | clean namespace |
| D3 | History posture | rewrite everything including archives | pre-1.0 |
| D4 | Sequencing | one wave | not released 1.0.0 yet |

The pre-1.0 argument carries D2, D3 and D4. There are no external consumers, so
falsifying a release log or breaking a permalink costs nothing real. Accepted.

It does **not** carry the BRAIN store. See §10.

---

## 10. Carve-out: the BRAIN store cannot be rewritten

This is the one place the "rewrite everything" posture is overridden, and it is
overridden by CyberOS's own protocol, not by preference.

### What is actually there

```
.cyberos/memory/store/          226 MB
  HEAD                          seq = 252,133
  audit/current.binlog          226,883,583 bytes, hash-chained, append-only
  audit/mmr/db.sqlite           Merkle Mountain Range index
  memories/ module/ adrs/ impl-plans/ code-reviews/ audits/ obs-injections/
                                1,334 memory files
```

- 446 memory files carry an FR ID **in the filename** (`impl-plan-fr-mcp-003.md`, `FR-MCP-003-sep986-naming-validator.audit.md`).
- 500 memory files carry an FR ID **in the body**.
- 15 distinct FR IDs are inside the **binlog rows themselves**.

### Why sed breaks it

- AGENTS.md §6.3: `chain = SHA-256(canonical(record_minus_chain) || prev_chain)`. Rows are append-only. Editing any byte of the binlog invalidates every subsequent row's chain hash across 252,133 rows.
- AGENTS.md §5.3: when a sidecar exists, the body's SHA-256 MUST equal `meta.body_hash`. A sed over 500 bodies makes 500 recorded hashes wrong.
- AGENTS.md §6.5: in-place edit of a written row, re-ordering, and deletion are forbidden ledger operations. Recovery is via consolidation, not row mutation.
- AGENTS.md §12: an invariant failure moves the agent to `FROZEN_RECOVERABLE`. Writes are refused until `cyberos doctor --repair`.

A `sed -i` over `.cyberos/memory/store/` does not rename the BRAIN. It bricks it.

### The protocol-legal way

The protocol already has the answer. Express the rename as new operations, not
as edits:

```
for each memory file carrying an FR id:
    move(old_path, new_path)              # 446 rows
    put(new_path, rewritten_body, meta)   # 500 rows
```

The chain then **records** the rename instead of being invalidated by it. Old
rows keep citing old paths, which is correct: that is what happened. Roughly 946
new rows on a 252k-row chain.

`scripts/migrate_fr_to_task.py --emit-brain-ops` emits these as NDJSON, ready to
pipe into the canonical writer. It never writes to the store directly.

Also emit one `memory.path_rename_epoch` aux row recording
`{old_prefix: "FR-", new_prefix: "TASK-", at_seq: <HEAD>}` so the walker can
resolve pre-epoch paths.

Unrelated but visible while measuring: the store is 226 MB / 252k rows against a
consolidation trigger of 5 MB / 5,000 rows (§7.6). It is roughly 45x past the
compaction horizon. Worth a separate task.

---

## 11. Sixth collision found during the codemod: `task_id` is a wire protocol

`fr_id` is the obvious rename target for `task_id`. It is not available.

`task_id` already exists 444 times, and its most important owner is not ours:

```
services/mcp-gateway/migrations/0017_mcp_tasks.sql
    CREATE TABLE mcp_tasks (
        task_id  UUID PRIMARY KEY DEFAULT gen_random_uuid(), ...
services/mcp-gateway/src/tasks.rs
services/mcp-gateway/src/tasks_pg.rs
```

That is the **MCP protocol's** long-running-task concept. It is a wire format.
CyberOS does not get to rename it. `tools/awh/harness` also uses `task_id` for
eval tasks.

Running total of what "task" means inside this repo after the rename:

| # | Meaning | Owner | Renameable? |
|---|---|---|---|
| 1 | backlog atom (was FR) | CyberOS | this change |
| 2 | unit of work inside a task | CyberOS `subtask@1` | renamed by D1 |
| 3 | CAF loop task (`L1-T1`) | `tools/caf` | no, own vocabulary + 63 golden fixtures |
| 4 | awh eval task (`tasks:`, `task_id`) | `tools/awh` | no, harness format |
| 5 | MCP long-running task (`mcp_tasks.task_id`) | MCP spec | **no, wire protocol** |
| 6 | agent runtime Task tool / TaskCreate | Claude Code | no |

Consequence for the codemod: **do not rewrite `fr_id` -> `task_id`.** It would
put two unrelated `task_id` meanings in one repo, and grep is how agents retrieve
context here.

Options:

- `fr_id` -> `task_id`, scoped to `modules/cuo` + `modules/skill` only, with an ADR recording the overload and `scripts/check_sep986_naming.sh` extended to enforce the module boundary. Cheapest, cognitively lossy.
- `fr_id` -> `backlog_task_id`. Unambiguous, verbose, greps clean. [recommended]
- Leave `fr_id` as-is. Honest but leaves the old vocabulary in the hot path.

This is decision **D5**, still open.

---

## 12. Codemod results (dry run, 2026-07-14)

`scripts/migrate_fr_to_task.py` — ordered longest-first passes, word-boundary
rules, deny-list, dry-run by default.

```
in-scope tracked files : 6,876
files with edits       : 2,494
total substitutions    : 26,454
```

| rule | hits | files |
|---|---|---|
| `id:fr-module-num` (`FR-AUTH-111` -> `TASK-AUTH-111`) | 23,141 | 2,280 |
| `skill:fr-audit` | 932 | 295 |
| `field:related-frs` | 516 | 486 |
| `path:docs-dir` | 432 | 174 |
| `free-word:frontmatter-field` (`sub_tasks` -> `subtasks`) | 302 | 295 |
| `cmd:ship` (`ship-feature-requests` -> `ship-tasks`) | 267 | 97 |
| `skill:fr-author` | 258 | 81 |
| `artefact:feature-request-1` (`feature_request@1` -> `task@1`) | 196 | 135 |
| prose + remaining identifiers | 412 | — |

### Residue the rules cannot safely reach

| residue | hits | files | disposition |
|---|---|---|---|
| bare `FR` / `FRs` in prose ("the FR", "FR IDs", "FRs land here") | 4,601 | 1,191 | mechanical, but 2 chars — needs a reviewed pass, not a blind sed |
| `fr_id`, `FrRow`, `_FR_ROW_RE`, `_FR_ID_RE` | 724 | 402 | blocked on D5 (§11) |
| `feature-request` in agent-config files | 13 | 8 | hand-edit: `.cursorrules`, `.windsurfrules`, `.grok/GROK.md`, `.github/copilot-instructions.md`, `.agents/rules/`, `.cursor/rules/`, `.gitignore` managed block, `.githooks/pre-commit`, `AGENTS.md`, `CLAUDE.md`, `CHANGELOG.md` |

The 13 agent-config files matter more than their count suggests. Each one is a
rules file that a different coding agent reads. Miss one and that agent keeps
generating `feature-request` artefacts after the rename.

---

## 13. Additions to the one-wave plan

### 13.1 `.git-blame-ignore-revs`

Rewriting 26,454 lines makes one commit the blame owner for most of the repo.
`git log -S` on history is unaffected (old commits keep old IDs), but `git blame`
becomes useless. Fix, one file:

```
# .git-blame-ignore-revs
<sha-of-the-rename-commit>   # fr -> task codemod, 2026-07-14, no semantic change
```

```
git config blame.ignoreRevsFile .git-blame-ignore-revs
```

Add it to `cyberos install` so downstream repos inherit it.

### 13.2 Rename epoch record

Even with D3 (rewrite archives), write one immutable record of what happened:

`docs/tasks/RENAME-EPOCH.md` — the full 563-row `FR-* -> TASK-*` mapping, the
commit SHA, the date. Without it, a rewritten 2026-05 code-review artefact claims
to have reviewed a `TASK-` that did not exist under that name, and nothing on
disk explains why.

### 13.3 Order of operations, one wave

1. `scripts/migrate_fr_to_task.py --apply` (content + `git mv` of the 15 known paths)
2. `git mv` the 563 per-spec directories `docs/tasks/<mod>/FR-*/` -> `TASK-*/`
3. Hand-edit the 13 agent-config files
4. Resolve D5, then close the `fr_id` residue
5. Reviewed pass over the 4,601 bare-`FR` prose sites
6. Reconcile the three-way schema drift (§4) and the backlog reader (§5) — now unavoidable, since both are in the blast radius
7. Add `type: feature | bug` + `templates/{feature,bug}.md` + `rubrics/bug.md` + the `REGRESSION-*` gate family (§7.3, §7.4)
8. Add `cannot_reproduce` + `duplicate` statuses (§7.5)
9. Re-author every `acceptance/TRIGGER_TESTS.md`; regenerate `.awh/eval-baseline.json` (the `--max-regression 0.0` gate blocks until you do)
10. `tools/cyberos-init/build.sh`: update the hardcoded skill allowlist and the two mirrored trigger regexes
11. `cyberos install`: delete the 5 stale agent symlinks by name, rewrite the managed `.gitignore` block
12. `--emit-brain-ops | python3 -m cyberos.writer apply` (§10)
13. `python3 scripts/migrate_fr_to_task.py --verify` must exit 0
14. CI: assert zero diff under every deny-list prefix

### 13.4 How to run it

Branch `rename/fr-to-task` is created. The codemod is written and dry-run clean.
It has not been applied: it refuses a dirty tree by design, and the two staged
changes (`.gitignore`, `docs/deploy/GO-LIVE-CHECKLIST.md`) are yours.

```bash
rm .wtest                      # sandbox artefact, harmless
git stash                      # park your two staged changes
git checkout rename/fr-to-task

python3 scripts/migrate_fr_to_task.py            # dry run  -> 27,205 subs / 2,498 files
python3 scripts/migrate_fr_to_task.py --apply    # content + git mv + 563 spec dirs
git diff --stat | tail -1

# then, in order:
#   1. hand-edit the 13 agent-config files (--residue lists them)
#   2. reviewed pass over the 4,601 bare-FR prose sites
#   3. echo "<sha>  # fr->task codemod, no semantic change" >> .git-blame-ignore-revs
#   4. python3 scripts/migrate_fr_to_task.py --emit-brain-ops | python3 -m cyberos.writer apply --ndjson -
#   5. python3 scripts/migrate_fr_to_task.py --verify     # must exit 0
```

Two guards exist because the first run in a sandbox died at file ~1,349 of 2,498
on an ACL-restricted file and left a half-renamed tree that `git checkout` could
not undo:

- **Dirty-tree guard** — refuses to apply unless `git status` is clean, so
  `git checkout -- .` is always a complete undo.
- **Two-phase apply** — every edit is planned in memory and every target file is
  checked writable *before* the first byte is written. Nothing is half-applied.

Keep both. A half-renamed repo of this size is not recoverable by inspection.

### 13.5 CI guard against re-entry

After the wave, add a hook that fails on any new `feature[-_ ]request` or
`FR-[A-Z]+-\d+` in tracked files outside the deny-list. Without it, the 5,000+
lines of vendored skill prose will reintroduce the old vocabulary within a month.
