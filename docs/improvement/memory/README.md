# Memory improvement program

Execution backlog for `docs/strategy/memory-enterprise-grade-and-auto-evolution-plan-2026-07-06.md` (the report). The report holds the why (S1-S12 strengths, F1-F35 findings, R1-R108 recommendations); this folder holds the how: 58 executable tasks, MEM-001 through MEM-058, in four phases.

## Files

| File | Purpose |
|---|---|
| `backlog.yaml` | Machine-readable source of truth for task status. Agents update it; humans audit it. |
| `tasks-phase-0.md` | P0 cards: safety and truth (auth, RLS, recall bugs, eval runner). |
| `tasks-phase-1.md` | P1 cards: a brain that understands content (facts, hybrid recall, PII, emitters). |
| `tasks-phase-2.md` | P2 cards: online-offline sync + compliance (sync protocol, retention, erasure, PDPL). |
| `tasks-phase-3.md` | P3 cards: the auto-evolution loop (evals, dream loop, feedback, self-healing, surfaces). |
| `program.yaml` | The adapter the `cyberos-improve-implement` / `cyberos-improve-review` skills read (branch, gate commands, ledger, guardrails). Replaces the old `PROMPT.md`. |

## Lifecycle

Statuses in `backlog.yaml`, matching the FR convention: `ready_to_implement` | `blocked` | `implementing` | `in_review` | `done` | `superseded`. Rules:

1. A task is eligible when its phase is the lowest phase with open tasks AND every entry in `deps` is `done`. `blocked` means unmet deps; the agent flips it to `ready_to_implement` when deps complete.
2. The agent sets `implementing` when it starts, `in_review` when the gate is green and the card's acceptance criteria all pass. Only the human reviewer sets `done`.
3. A task the agent cannot finish inside the circuit-breaker budget (5 consecutive gate failures) routes back to `ready_to_implement` with a `blocked_note`, per `modules/cuo/EXECUTION-DISCIPLINE.md` §2.3, and the agent moves on.
4. Every status change lands in the same commit as the work it describes.

## Working agreement

Branch: all work on `auto/memory-enterprise` (create from latest `main` if absent). One task per commit, message `MEM-0NN: <title>`. No `git push`, no deploy, no destructive migration on shared data, no secret handling by the agent: those are operator actions (EXECUTION-DISCIPLINE §2.2).

Gates, in order, before a task may enter `in_review`:

```
cargo fmt --check
cargo clippy -p cyberos-memory --all-targets -- -D warnings
cargo test  -p cyberos-memory            # needs services/dev: docker compose up -d
python -m pytest                          # only when modules/memory was touched
```

If the executing environment cannot build (sandbox), route the gate through the Mac-gate loop (author on mount, gate on the operator's machine) and record the gate output in the ledger entry. A task without recorded green gates is not `in_review`.

Evidence: each work session appends a ledger file `docs/auto-work/YYYY-MM-DD-memory-<n>.md` listing tasks touched, gate outputs, decisions taken (ADR-class decisions get a real ADR under `docs/adrs/`), and anything routed back.

## Sequencing summary

P0 (MEM-001..011, ~63 h): closes the critical security holes and recall bugs, stands up the eval runner. Nothing else starts until MEM-001..003 are `in_review`, because everything later assumes a trustworthy recall path.

P1 (MEM-012..031, ~150 h): the fact layer, content-aware ingestion behind consent + PII, hybrid ranking, day-1 emitters, embedding lifecycle.

P2 (MEM-032..044, ~120 h): first-party sync on the chat-core pattern, retention, crypto-shredding erasure, PDPL pack, read audit, chain anchoring.

P3 (MEM-045..058, ~110 h): judge + benchmark, dream loop, feedback ranking, A/B + GEPA, self-healing, poisoning defenses, MCP/memory-tool surfaces, ops tile.

Coverage: the union of `refs` across all tasks equals R1-R108 plus every finding F1-F35, except R42 (Matryoshka adaptive retrieval), parked in the icebox section of `backlog.yaml` until a model migration makes it relevant.

## For the human reviewer

Each card ends with a review checklist: what to inspect beyond the green gate (security posture, data-safety, spend). The review protocol, including the per-phase sign-off gates and the approve/reject mechanics, is the `cyberos-improve-review` skill (`.claude/skills/cyberos/`). The short version: review `in_review` tasks commit by commit, run the named probe commands yourself for security-class tasks, set `done` in `backlog.yaml`, and push when a phase closes.
