# core/improve/ — the self-improvement loop, mapped on one screen

Six things live here. One is a procedure, two are templates/ledgers you append to, one is a register you re-verify, and two are append-only history.

| File / dir | What it is | When you touch it |
|---|---|---|
| [`CRITIC.md`](./CRITIC.md) | **The engine.** The complete one-cycle procedure an agent (or human) runs to change AUDIT.md: gather evidence → critique → ONE minimal change → gate on evals → version → retro → stop decision. | When asked to "run an improvement cycle". It is read and followed, never edited casually — it is itself versioned by ordinary commits. |
| [`RETROSPECTIVE.md`](./RETROSPECTIVE.md) | **A template.** The 10-question, 20-point rubric for scoring any run of AUDIT.md. | Never edited per-run. Copy it into `retros/` and fill the copy. |
| [`FAILURE_LOG.md`](./FAILURE_LOG.md) | **The evidence ledger.** One row per observed letter-vs-intent failure. The Rule of Three lives here: 1st observation = note, 2nd = candidate edit, promotion = protocol change. | Append a row when a run misbehaves. Never delete rows — mark them promoted/deferred. |
| [`BLINDSPOTS.md`](./BLINDSPOTS.md) | **The honesty register.** What the protocol + harness *cannot* see, each row ACCEPTED / MITIGATED / CLOSED. | Re-verify at every campaign start and whenever validate.py changes. Add rows with FAILURE_LOG-grade evidence; never delete — change status. |
| [`retros/`](./retros/) | **Scored history.** One filled rubric per run or critic cycle. `core/evals/scripts/retro-summary.py` aggregates these into per-version trends. | Add a file after every run. Append-only. |
| [`versions/`](./versions/) | **Immutable releases.** A byte-exact snapshot of every released AUDIT.md. CI verifies the current protocol matches its snapshot. | Written once per release by the CRITIC ritual. NEVER edit or delete anything here. |

## The flow in one line

run AUDIT.md somewhere → score it (`retros/`) → log what went wrong (`FAILURE_LOG.md`) → when evidence recurs, run `CRITIC.md` → one change to `AUDIT.md`, gated by `core/evals/`, snapshotted to `versions/`, logged in `CHANGELOG.md` → repeat until a campaign stop rule fires.

## Rules that keep this folder trustworthy

- One protocol change per cycle — attribution is the whole point.
- Everything here is append-only except `BLINDSPOTS.md` statuses and `FAILURE_LOG.md` promotion columns.
- No retro → no edit. Evidence first, wording second.
