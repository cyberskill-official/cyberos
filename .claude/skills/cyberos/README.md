# CyberOS official skills — improvement-program execution protocol

> **Retired / superseded (2026-07-08).** CyberOS now runs a single implementation workflow, `chief-technology-officer/ship-feature-requests`. Improvement work is a feature-request (`class: improvement`), not a separate track, and HITL is required at the two human-acceptance gates. The two skills here (`cyberos-improve-implement`, `cyberos-improve-review`) are retired tombstones that redirect to the workflow. The rest of this document is kept for historical context. Start at `modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md` (section 1a) and `docs/feature-requests/improvement/README.md`.

This directory holds the **official, reusable CyberOS protocol** for driving an improvement program (a
backlog of enterprise-hardening tasks) from spec to `done`. Two skills, one for each half of the loop:

| Skill | Who runs it | What it does |
|---|---|---|
| `cyberos-improve-implement` | an agent (Claude, Codex, any) | claims the next eligible task, implements it, self-verifies through the gates, ledgers evidence, and moves it to `review` — never `done`, never pushes. |
| `cyberos-improve-review` | a human, optionally agent-assisted | walks the review checklist (or prepares a read-only packet) and moves tasks from `review` to `done`. |

They replace the per-program `PROMPT.md` files that each improvement program used to carry (memory's
`PROMPT.md`, chat's `PROMPT-IMPLEMENT.md`/`PROMPT-REVIEW.md`, the deep-audit `docs/improvement/PROMPT.md`).
Those were ~90% identical; the shared discipline now lives here once, and each program keeps only a small
`program.yaml` adapter.

## How it stays generic

The skills contain zero program-specific paths. Everything that differs between programs — the backlog file
and its status words, the id prefix, the branch, the gate commands, the ledger location, the report, and the
protected invariants — lives in a per-program **`program.yaml`** manifest under `docs/improvement/<program>/`.
The skill reads that manifest and runs the same loop against any program.

## Onboarding a new program (or a new project)

1. Create the backlog and specs under `docs/improvement/<program>/` (a backlog file with a status field, task
   specs, a README, a strategy report).
2. Drop a `program.yaml` next to the backlog. Schema (all keys shown; copy and edit):

```yaml
program: memory                         # short slug
report: docs/strategy/memory-enterprise-grade-and-auto-evolution-plan-2026-07-06.md
report_refs: "R# strengths/findings/recommendations in the report"
backlog:
  file: docs/improvement/memory/backlog.yaml
  format: yaml                          # yaml | markdown-table
  status_field: status
  id_prefix: MEM
statuses:                               # map THIS program's words -> the canonical lifecycle
  ready: ready_to_implement             # canonical: ready | doing | review | done | blocked
  doing: implementing
  review: in_review
  done: done
  blocked: blocked
task_specs: docs/improvement/memory/tasks-phase-{phase}.md
branch:
  mode: one-per-program                 # one-per-program | one-per-task
  name: auto/memory-enterprise          # (one-per-program) OR name_pattern for one-per-task
commit_format: "MEM-0NN: <title>"       # one task = one commit
gates:
  environment: mac-gate                 # mac-gate (sandbox can't build; gate on the operator's Mac) | local
  commands:
    - cargo fmt -p <crate> -- --check
    - cargo clippy -p <crate> --all-targets -- -D warnings
    - cargo test -p <crate>
ledger:
  mode: per-session-file                # per-session-file | single-append
  path: docs/auto-work/{date}-memory-{n}.md
  session_summary: docs/auto-work/{date}-memory-session-summary.md
selection: "lowest phase -> priority (critical>high>medium>low) -> lowest id; deps must be done"
guardrails:
  protected:                            # invariants no task may weaken to pass a gate
    - RLS fail-closed
    - deny-by-default access
    - consent default-deny
    - hash-chained audit (Layer-1 wins)
    - gateway-only model calls
  operator_only:                        # agent prepares, human runs (EXECUTION-DISCIPLINE §2.2)
    - git push / deploy / merge
    - destructive migration on shared data
    - secrets / legal filings
  human_only_ids: []                    # tasks only a human may close (legal, pen-test, decision verdicts)
```

3. Trigger it: in Claude, invoke the `cyberos-improve-implement` skill and name the program; or say
   "implement the `<program>` backlog". For a review pass, `cyberos-improve-review`.

## Using it across different agents

- **Claude / Claude Code / Cowork:** the skills load by name from `.claude/skills/`. Just reference the skill
  and the program.
- **Codex or any other agent:** hand it the skill file path directly, e.g. "Follow
  `.claude/skills/cyberos/cyberos-improve-implement/SKILL.md` for `docs/improvement/memory`." The skill body is
  plain Markdown and self-contained (it embeds the EXECUTION-DISCIPLINE §2 halt rules and references the full
  `modules/cuo/EXECUTION-DISCIPLINE.md` when present), so it works without Claude-specific machinery.
- **Other repos / future projects:** copy `.claude/skills/cyberos/` into the new repo and add a `program.yaml`
  per program. Nothing here hardcodes the cyberos tree except the optional EXECUTION-DISCIPLINE reference. See
  `ADOPTING.md` in this folder for the full ordered procedure (copy, gitignore carve-out, program dir,
  manifest, gate environment, trigger, sign-off).

## Where this sits in the CyberOS workflow surface

- Official registration + persona ownership: `modules/cuo/chief-technology-officer/workflows/run-improvement-program.md`.
- Halt discipline it builds on: `modules/cuo/EXECUTION-DISCIPLINE.md`.
- Distinct from the feature-request lifecycle (`docs/feature-requests/`, driven by
  `chief-technology-officer/ship-feature-requests`) — improvement programs are a separate, lighter track with
  their own id spaces; a task that grows into product scope converts to an FR and is marked
  `superseded by FR-...`.
