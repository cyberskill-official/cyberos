---
batch: ship/batch-8a-core-locks-notes
members: []
started: 2026-07-23T15:40:00+07:00
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# Batch 8A ship notes — workflow evolution candidates

Branch: `ship/batch-8a-core-locks`  
Shipped: TASK-CUO-302, TASK-CUO-303, TASK-CUO-304 from gate-1 onward  
Date: 2026-07-23  
Closed: gate-2 all-accept 2026-07-23 (`testing → done` for CUO-302/303/304) — evidence `batch-8a-gate2-acceptance.md`

Frontmatter present so `render-status-hub.mjs` can parse every `docs/batches/*.md` (see evolution candidate #1). `members: []` — this is a notes artefact, not a membership ledger; the shipping members live in `batch-8a-gate1-acceptance.md`.

These notes capture friction found while shipping Batch A. Intentional: shipping evolves the workflow.

## Workflow evolution candidates

### 1. Verdict evidence under `docs/batches/` must carry YAML frontmatter

`tools/docs-site/render-status-hub.mjs` scans **every** `docs/batches/*.md` as a batch ledger and requires an opening YAML fence. A prose-only gate-1 acceptance note at `docs/batches/batch-8a-gate1-acceptance.md` made `test_task_layout.sh::t04` fail with `backlog=572 roadmap=` (empty — the hub threw before printing the count).

**Candidate:** document in ship-tasks HITL steps that `--verdict-evidence` paths under `docs/batches/` MUST be ledger-shaped (frontmatter with `batch:` + `members:`), OR put acceptance notes under the task folder (`<task>/gate1-acceptance.md`) which the hub does not treat as ledgers. Batch 8A kept the shared path by promoting the note into a proper incomplete batch ledger for `ship/batch-8a-core-locks`.

### 2. Sub-batch / one-branch-per-batch vs parent batch ledger

Operator asked for one branch per batch (`ship/batch-8a-core-locks`) while the parent ledger `docs/batches/batch-8-audit-hardening.md` still lists all ten members. ship-tasks §11a names `batch/<n>-<theme>`; §11d writes one ledger per batch close. There is no doctrine for **sub-batches** of a larger audit wave (8A/8B/…) that share one parent ledger but ship on separate branches.

**Candidate:** allow nested/sub-batch ledgers (as Batch 8A did), or require renaming so each ship branch owns exactly one ledger and parent membership is an index only. Clarify whether `ended` on the parent waits for every sub-batch gate-2.

### 3. Multi-task HITL: one evidence file, N flips

Operator said "all accept for A". ship-tasks already allows one utterance → N recorded verdicts. Mechanically we used **one** evidence file for three `--verdict-evidence` flips (shared path). That works and appends three `status_overridden` rows pointing at the same reason path. Undocumented whether shared evidence is preferred vs per-task copies.

**Candidate:** ship-tasks HITL section should explicitly sanction shared evidence for batch "approve all" with one file + N flips (what we did), and note that each flip still gets its own audit row.

### 4. Truth-precedes-index is easy to miss mid-flight

`backlog-mutate flip` refuses (exit 6) unless frontmatter already carries `<to>`. The HITL docs emphasize `--verdict-by` / `--verdict-evidence` but the write-frontmatter-first step is easy to forget when resuming at `reviewing`. Bare order of operations for a gated flip:

1. Write evidence file (non-empty)
2. Set frontmatter `status: <to>`
3. `backlog-mutate flip … --verdict-by … --verdict-evidence …`
4. Regenerate `docs/status/` (or rely on pre-commit)

**Candidate:** fold a one-liner ordered checklist into the two HITL step blurbs in ship-tasks.md.

### 5. Status-page regen is easy to skip before gates

ship-tasks §11a says every backlog write rides with a regenerated status page. Running gates before `bash .cyberos/lib/status-page.sh` does not fail on its own once the batch file parses, but the pre-commit hook is the safety net — agent-driven flips outside a commit can leave `docs/status/` stale until commit time. We regenerated explicitly before re-running gates.

**Candidate:** have `backlog-mutate` optionally invoke status-page regen, or have `run-gates.sh` soft-warn when status stamp drifts from backlog totals.

### 6. Live BRAIN is FROZEN but still accepts `status_overridden`

`cyberos doctor` is FAIL (`layout-root-canonical`: stray `adrs/` + `impl-plans/` — MEMORY-303 repair deferred). Gated flips still resolved the store and appended `status_overridden` successfully. That matches CUO-303 (appender does not run doctor). Document for operators: gate-1/gate-2 verdict rows land even while the store is `FROZEN_RECOVERABLE` from layout drift — do not assume "frozen" blocks doc-driven appends.

### 7. Installed `run-gates.sh` lags the Batch A source

Installed `.cyberos/cuo/gates/run-gates.sh` (86 lines) differs from `tools/install/gates/run-gates.sh` (123 lines) — no fail-closed floor, no doctor gate. Machine gates therefore stayed GREEN despite doctor FAIL on the live store. Refresh is intentionally deferred until MEMORY-303 store repair (batch ledger operator item 8). Until then, CI/scratch installs prove CUO-302; this repo's installed floor does not yet enforce it.

**Candidate:** ship-tasks testing phase should note when installed gates ≠ source gates, and whether that is an accepted carve-out or a halt.

### 8. Accidental probe append (session error)

Before the real flips, a dry-run `memory-append.mjs … status_overridden` for `TASK-PROBE` succeeded and wrote **seq 1** on the live chain. Real gate-1 flips are seq 2–4. The probe row is immutable ledger fact; do not purge without §3.6 approval. Treat as operator-visible noise until a cleanup policy exists for mistaken doc-driven appends.

**Candidate:** backlog-mutate / memory-append should support a `--dry-run` that never writes; agents probing exit codes today have no safe probe path against a live store.

### 9. MCP `task_gates` connection dropped mid-ship

`task_gates` MCP call failed with `Connection closed`. Fell back to `bash .cyberos/cuo/gates/run-gates.sh` directly. Not blocking, but the MCP surface is not yet a reliable substitute for the shell gate entry.

### 10. `test_task_layout.sh::t04` side-effect: regenerates BACKLOG

t04 runs `migrate_improvement_to_task.py --backlog`, which **rewrites** BACKLOG.md from frontmatter. Safe when frontmatter is truth (our flips survived), surprising if someone expected only a count. Worth noting for agents who have uncommitted hand-edits to backlog prose.

## Gate-2 close (2026-07-23)

Operator: Gate-2 all-accept for Batch A. Bare `testing → done` refused exit 8; gated flips with `--verdict-by "Stephen Cheng"` + `--verdict-evidence docs/batches/batch-8a-gate2-acceptance.md` succeeded (`status_overridden` seq 5–7). CUO-302/303/304 are **done**. Store still FROZEN until MEMORY-303 repair (next operator item).

## Testing-phase gate results (this run)

| Gate | Result |
|------|--------|
| `bash .cyberos/cuo/gates/run-gates.sh` | **GREEN** — `suites: pass=49 fail=0 skip=1` |
| `modules/cuo/.awh/gate.sh` | **GREEN** — weighted pass@1=100%, no regression |
| `bash scripts/caf_gate.sh cuo` | **CLEAN** — target health PASS; no sealed `.caf/` (health-only floor) |

Transcripts: `batch-8a-gates-transcript.txt`, `batch-8a-awh-caf-transcript.txt`.
