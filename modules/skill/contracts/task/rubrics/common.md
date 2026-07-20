# `common` — rule families that apply to every task, whatever its `type`

Loaded by `task-audit` for all values of `type` (FM-108), then composed with the per-type family:

```
rubrics/common.md   +   rubrics/{type}.md   =   the gate for this task
```

The authoritative rule text lives in `modules/skill/task-audit/RUBRIC.md`. This file is the **composition contract**: it declares which families are universal, which are type-scoped, and what a new type must do to join.

---

## §1  Universal families

| family | what it gates | why it is universal |
|---|---|---|
| `FM-*` | frontmatter shape: `id`, `title`, `module`, `status`, `priority`, `created_at`, `author`, `department`, `template`, `type`, `client_visible`, `ai_authorship`, `eu_ai_act_risk_class` | Every task is a record with the same identity and provenance, regardless of what kind of work it describes. |
| `SEC-*` | required body sections exist and are non-empty | Section *names* vary by type (see §2); the requirement that a declared section have body does not. |
| `SAFE-*` | untrusted-content discipline (AGENTS.md §11) — customer quotes fenced, no nested/unclosed blocks, prompt-injection markers scanned | Anything quoting the outside world is untrusted, whether it is a feature idea from a customer or a stack trace from a user's browser. |
| `QA-*` | prose quality: no fabricated metrics, alternatives considered, success metrics defined | |
| `COND-*` | conditionally-required sections keyed on frontmatter (`client_visible: true` → Customer Quotes; `eu_ai_act_risk_class ∈ {limited, high}` → AI Risk Assessment) | |
| `TRACE-*` | every normative §1 clause names the test that proves it | The clause-to-test link is the spine of the whole system. A bug's clauses are its regression assertions; a feature's are its acceptance criteria. Both must be traceable. |
| `FM-112` | no `# UNREVIEWED` marker survives `draft` | The 2026-07-14 schema migration backfilled two fields it could not derive. Type has nothing to do with it. |

## §2  Type-scoped families

| `type` | adds | relaxes |
|---|---|---|
| `feature` | — | — |
| `bug` | `BUG-*` (§10.1–10.2 of `rubrics/bug.md`), `REGRESSION-*` (§10.3) | edge-case-matrix floor `total_rows >= 8`; skips ADR / spike / SDD unless the fix crosses a module boundary |
| `improvement` | — | — |
| `chore` | — | — |

`improvement` and `chore` share the feature skeleton today. That is deliberate: adding a rule family costs nothing later, and inventing rules for a type nobody has filed yet is how you get a taxonomy nobody fills in correctly.

## §3  Adding a type

The dispatch is data, not code. A new type costs:

1. `templates/<type>.md` — the body skeleton. **Required.** `task-author` HALTS if it is missing rather than falling back to `feature`, because a silent fallback produces an artefact that passes a gate which never knew what to ask.
2. `rubrics/<type>.md` — **optional.** Absent means "common families only".
3. one row in the FM-108 enum in `modules/skill/task-audit/RUBRIC.md`.

No skill code changes. If you find yourself editing `task-author` to add a type, the dispatch has been hardcoded somewhere and that is the bug.

## §4  Scoring

Composed families score into the same 10-point verdict. `task-audit` refuses to pass below 10/10 regardless of type. A bug is not held to a lower bar than a feature — it is held to a *different* one.
