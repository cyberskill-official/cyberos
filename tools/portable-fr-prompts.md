# Portable FR-driven prompts (project- and agent-agnostic)

Three prompts to drive (1) FR drafting, (2) FR implementation, and (3) FR re-audit
across **any** project + **any** AI coding agent (Claude Code, Cursor, Codex, etc.).

## Placeholders to substitute before running

| Placeholder | What it means | Example |
|---|---|---|
| `{{REPO_ROOT}}` | Absolute path to the project root the agent will work in | `/Users/stephencheng/Projects/X/api-service` |
| `{{BACKLOG_PATH}}` | Path **relative to REPO_ROOT** where the FR backlog table lives | `docs/feature-requests/BACKLOG.md` |
| `{{FR_DIR}}` | Path **relative to REPO_ROOT** where individual FR markdown files live | `docs/feature-requests/{module}/` |
| `{{SOURCE_DOC_PATH}}` | The PRD / SRS / strategy doc the FRs are drafted from | `docs/prd/AUTH-MODULE-PRD.md` |
| `{{MODULE}}` | Module prefix for new FR IDs | `AUTH` (yields `FR-AUTH-001`, `FR-AUTH-002`, …) |
| `{{LANG}}` | Primary implementation language for sanity in workflows | `Rust 1.81`, `TypeScript`, `Python 3.12` |
| `{{TEST_CMD}}` | Test command the agent runs at the coverage gate | `pnpm test`, `cargo test --workspace`, `pytest -q` |
| `{{BUILD_CMD}}` | Build/typecheck the agent runs before declaring an FR shipped | `cargo check`, `pnpm typecheck`, `mypy .` |
| `{{CAP}}` | Optional FR-count cap if you don't want the full backlog drained in one run | `10`, or omit |

If your project has its own house style/rubric document, drop its path in too:
`{{RUBRIC_PATH}}`. Otherwise the prompts carry the rubric inline so the agent
doesn't need an external file.

---

## Prompt 1 — Draft FRs from a source doc

```
Repo: {{REPO_ROOT}}
Source doc: {{SOURCE_DOC_PATH}}
Target backlog: {{BACKLOG_PATH}}
FR file directory: {{FR_DIR}}
Module prefix: {{MODULE}}
Cap (optional): {{CAP}}

TASK
Draft FR-{{MODULE}}-NNN entries from the source doc and append each to the
backlog. One file per FR under {{FR_DIR}}/FR-{{MODULE}}-NNN-<slug>.md.

PROCESS (per FR, no exceptions)
1.  Read the next chunk of the source doc that hasn't been covered by an
    existing FR. Skip already-covered material.
2.  Draft the FR. Required sections:
        §1  Description (BCP-14 normative — MUST / SHOULD / MAY)
        §2  Why this design (rationale prose for humans)
        §3  API contract (types + schemas, language-appropriate)
        §4  Acceptance criteria (numbered, testable)
        §5  Verification (concrete test sketches, language-appropriate)
        §6  Implementation skeleton (real code, not pseudocode)
        §7  Dependencies (other FRs, crates/packages, infra)
        §8  Example payloads (real JSON / HTTP / SQL)
        §9  Open questions (resolved + deferred)
        §10 Failure modes inventory (table: failure | detection | outcome | recovery)
        §11 Notes
3.  Self-audit against this rubric and ITERATE until score is 10/10:
        - Every §1 clause traces to a §4 AC and to at least one §5 test.
        - No §4 AC is uncovered by tests.
        - §3 types compile in your head (no missing imports, no impossible
          trait bounds, no field of unknown type).
        - §10 covers every error path implied by §1.
        - The FR is self-contained — a developer can implement it without
          reading any other FR, slack thread, or doc except the cited
          source pages.
    Do NOT move to the next FR until the current one is 10/10.
4.  Append a one-line row to {{BACKLOG_PATH}} atomically:
        | FR-{{MODULE}}-NNN | <title> | <pri> | draft (10/10) | <deps> | <effort> |

DISCIPLINE
- March autonomously through the source doc. Do not ask the user between
  FRs unless you hit a real decision (e.g. spec contradicts itself; pick
  the path and continue otherwise).
- Every FR file change → atomic commit with message
  `feat(fr): draft FR-{{MODULE}}-NNN <title> (10/10)`.
- Push is left to the human. Never run `git push` automatically.
- If the source doc is ambiguous on a specific normative choice and you
  cannot decide unilaterally, STOP and report the ambiguity. Do not draft
  a guess.

STOP CONDITIONS (any one ends the run)
- Source doc exhausted (no more uncovered material).
- {{CAP}} FRs drafted (if cap supplied).
- Three consecutive FRs hit irreducible ambiguity.

REPORT AT END
- Number of FRs drafted.
- List of FR IDs + titles.
- Any deferred questions for the human.
```

---

## Prompt 2 — Implement the next eligible FR (loop until backlog drained)

```
Repo: {{REPO_ROOT}}
Backlog: {{BACKLOG_PATH}}
FR files: {{FR_DIR}}
Language: {{LANG}}
Test command: {{TEST_CMD}}
Build command: {{BUILD_CMD}}
Cap (optional): {{CAP}}

TASK
Drive each "planned" FR through to "shipped + strict-audited", in
dependency order. Repeat until the backlog is drained.

ELIGIBILITY (pick the next FR each iteration)
- Status = "planned".
- All `depends_on` FRs have status = "shipped" or "shipped + strict-audited".
- Tie-break by priority (MUST > SHOULD > MAY), then by ID order.

PER-FR WORKFLOW (run end-to-end for ONE FR before starting the next)
1.  Read the FR markdown in full. Build a §10-style "audit dossier" section
    inside the FR file (or a sibling FR-NNN.audit.md). Sub-sections:
        §10.1 Verdict
        §10.2 Gap list (one row per spec-vs-code gap, with severity +
              estimated effort + status: OPEN / CLOSED / DEFERRED)
        §10.3 Fix log (one row per gap-closure with ts + change + tests +
              build result + commit hash)
        §10.4 Backlog mutations (ts + line + from → to)
        §10.5 Working notes (anything weird about the codebase the future
              auditor needs to know)
        §10.6 Spec amendments recommended (if any)
        §10.7 Slice plan (which gaps in this commit; what's deferred)
2.  Compare the FR spec to the actual code. For every clause in §1, find
    the matching code path. Any mismatch → entry in §10.2.
3.  Close each OPEN gap in §10.2. Code change → test → run {{BUILD_CMD}}
    then {{TEST_CMD}}. If both pass, mark CLOSED in §10.3 with the commit
    hash you'll attach.
4.  For each DEFERRED gap, write a one-paragraph rationale citing which
    other FR / sprint / system will pick it up. Deferral without rationale
    is forbidden.
5.  Update {{BACKLOG_PATH}}: change the FR's status cell to
    `shipped + strict-audited` (or `shipped` if some gaps deferred). Note
    the deferred count.
6.  Commit. Single atomic commit per FR with message:
        `fix(<scope>): <FR-ID> — close N of M gaps; M-N deferred to <where>`
    Body lists NEW / CHANGED / DEFERRED sections.

CIRCUIT BREAKERS
- If {{BUILD_CMD}} or {{TEST_CMD}} fail 5 consecutive times for ONE gap
  with no progress between failures → revert the in-progress changes for
  that gap, mark the gap `[FAILED: UNRESOLVABLE]` in §10.2 with the last
  error, and move on to the next gap in the same FR.
- If the same FR hits the per-gap circuit breaker on 3 distinct gaps →
  stop work on that FR, mark its backlog status `[BLOCKED: see audit §10]`,
  and continue with the next eligible FR.

DISCIPLINE
- NO partial-ship within an FR. Drive every FR end-to-end (all gaps
  closed or explicitly deferred) before moving to the next. Pause only
  between FRs, never within one.
- Push is the human's job. Never run `git push` automatically.
- One commit per FR. No "fix typo" or "WIP" commits between gap closures
  on the same FR — squash if needed.
- After each FR: emit a 5-line summary in chat — FR-ID, gaps closed,
  gaps deferred, build/test status, commit hash. Then start the next.

STOP CONDITIONS (any one ends the run)
- Backlog drained (no more eligible FRs).
- {{CAP}} FRs shipped (if cap supplied).
- Three consecutive FRs hit the BLOCKED state.

REPORT AT END
- Total FRs shipped, deferred-gap count by destination, BLOCKED FRs.
- One git log line per commit (you don't push, the human does).
```

---

## Prompt 3 — Re-audit FRs against current code

```
Repo: {{REPO_ROOT}}
Backlog: {{BACKLOG_PATH}}
FR files: {{FR_DIR}}
Mode: <draft | shipped>     # pick one
Cap (optional): {{CAP}}

TASK
Re-audit a subset of FRs against the current code, detect drift, raise
new gaps. This is a maintenance pass, not a new-work pass.

MODE A — draft (FRs flagged "draft (<10/10)")
For each draft FR below 10/10:
1.  Re-run the §1→§4→§5 traceability audit (every clause traces to an AC,
    every AC traces to a test).
2.  Re-run the §10 failure-modes inventory check.
3.  Iterate the FR until it scores 10/10. Update its status in
    {{BACKLOG_PATH}} from `draft (X/10)` to `draft (10/10)`.

MODE B — shipped (regression scan)
For each "shipped" or "shipped + strict-audited" FR:
1.  Re-read the FR spec.
2.  Walk the code that implements it. Has it drifted? Has new code been
    added that contradicts a §1 clause? Has a referenced dependency been
    renamed / removed?
3.  Append new gaps to the FR's §10.2 with status OPEN, and set the
    backlog status to `[REGRESSION: see audit §10]`.
4.  Emit a one-line summary per FR: `FR-X-NNN re-audit: clean | N gaps`.

DISCIPLINE
- One commit per re-audited FR. Message:
    `chore(audit): re-audit <FR-ID> — <result>`.
- For each FR re-audited, write the prior score and the new score to a
  changelog row (or your project's equivalent of an audit log) so the
  delta is reviewable.
- Push is the human's job. Never `git push` automatically.

STOP CONDITIONS
- Target subset exhausted, OR
- {{CAP}} FRs re-audited.

REPORT AT END
- Subset processed, regressions found, drafts upgraded to 10/10.
```

---

## How to use these with another AI agent

1. **Pick the prompt** you want for the run (draft, implement, or re-audit).
2. **Fill in the placeholders** — at minimum `{{REPO_ROOT}}`, `{{BACKLOG_PATH}}`,
   and either `{{SOURCE_DOC_PATH}}` (prompt 1) or `{{TEST_CMD}}` + `{{BUILD_CMD}}`
   (prompts 2 + 3).
3. **Paste into the agent's first message**. Add a one-line hint about which
   subdirectory or module to focus on if the repo is large.
4. **Watch the first commit**. If the agent's commit message / file structure
   doesn't match the prompt's "DISCIPLINE" section, stop and tighten the prompt
   before letting it run autonomously.
5. **Push manually** when you're satisfied with a batch. The prompts never
   `git push`; that's intentional — you keep the merge button.

## Differences vs the original cyberos prompts

| Cyberos-specific thing | Replaced with |
|---|---|
| `cuo/chief-technology-officer/implement-backlog-frs` workflow file | The 6-step per-FR procedure inlined in Prompt 2 |
| `AUTHORING_DISCIPLINE.md` rubric file | The 10/10 rubric inlined in Prompt 1's PROCESS step 3 |
| `feature-request-author` + `feature-request-audit` skill names | Generic "draft + self-audit until 10/10" loop |
| `BRAIN audit chain` + `AGENTS.md §14` heartbeat | Atomic commits with structured messages; project's existing changelog |
| `memory feedback_fr_autonomous_march` reference | "March autonomously … do not ask between FRs unless real decision" |
| `memory feedback_no_partial_ship_per_fr` reference | "NO partial-ship within an FR" stated directly in Prompt 2 |
| `re_audit_complete` BRAIN row emission | Changelog row with prior/new score |
| Hard-coded `/Users/stephencheng/.../cyberos` path | `{{REPO_ROOT}}` placeholder |

The receiving agent never needs to know cyberos exists; the rigor is preserved
because it's inlined into the prompt rather than referenced by file path.
