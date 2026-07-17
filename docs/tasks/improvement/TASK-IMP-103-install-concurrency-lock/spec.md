---
id: TASK-IMP-103
title: Install concurrency lock
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: [TASK-IMP-104, TASK-IMP-106]
related_tasks: [TASK-IMP-083, TASK-IMP-095]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 3
service: tools/install
new_files:
  - tools/install/tests/test_install_lock.sh
modified_files:
  - tools/install/install.sh
  - tools/install/uninstall.sh
source_pages:
  - "IMPROVEMENT_HANDOFF.md §7.2 IMP-10 (the only never-triaged finding from the original audit)"
  - "tools/install/install.sh:57-58 (rm -rf then cp -R, no lock; grep -c install.lock|flock = 0 on main bb231900)"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-103: Install concurrency lock

## Summary

`install.sh` removes the vendored machine and re-copies it with no lock, so a second install - or any reader touching `.cyberos/` during the window - can observe a half-vendored machine. Guard the vendor step with a `mkdir` lock carrying the owning pid and a start timestamp, refuse a concurrent install with both pids named, break a demonstrably stale lock with a warning, and release on every exit path.

## Problem

`tools/install/install.sh:57` runs `rm -rf "$CY/cuo" "$CY/plugin" "$CY/mcp"` and `:58` re-copies. Between those lines the machine does not exist. `grep -c "install.lock\|flock" install.sh` returns 0 - there is no lock of any kind. Observed twice on a slow mount during the 2026-07-16 sachviet run as transient missing-file probes; reported as IMP-10 in IMPROVEMENT_HANDOFF.md and then never triaged across five batches, which makes it the only unexecuted finding from the original audit.

The failure is transient and needs concurrency to bite, which is exactly why it has survived: nothing forces it to happen, and nothing stops it either. Every consumer that adds CI-driven installs, or runs an agent that reads `.cyberos/` while a human re-installs, reaches it.

## Proposed Solution

Acquire `.cyberos/.install.lock` by `mkdir` (atomic on POSIX) before the vendor step, writing `pid` and `started_at` into it. On collision, read the lock: if the owning pid is alive on this host, refuse with both pids and the lock's age. If the lock is older than a threshold AND its pid is dead, break it with a warning naming what was broken. Release via `trap` on EXIT, INT, and TERM so a killed install does not wedge the next one. Same-host liveness only - the pid in a lock written by another machine on a shared mount means nothing, and the code must say so rather than guess.

## Alternatives Considered

- `flock(1)`. Rejected: not present on stock macOS, and the payload's shell floor is POSIX + coreutils. `mkdir` is atomic everywhere we run.
- Copy-then-swap (vendor to a temp dir, rename over). Rejected as the primary fix: `rename` over a populated directory is not atomic across all platforms, and the change is larger than the defect. Worth revisiting independently.
- Do nothing (accept the window). Rejected: it is the last open finding from the audit that produced this backlog, and the fix is ~10 lines.

## Success Metrics

- Primary: a second concurrent install refuses with a non-zero exit and both pids named, while the first completes untouched - suite-asserted. Baseline: today the second install proceeds and both write the same tree.
- Guardrail: a stale lock (dead pid, past threshold) never blocks a legitimate install - it is broken with a warning, and the warning names the lock it broke.

## Scope

In scope: `install.sh` lock acquire/refuse/break/release, the uninstall-side removal of a lock it owns, suite arms.

### Out of scope / Non-Goals

- Locking any operator-owned path. The lock lives inside `.cyberos/` (the machine we own) and never guards `docs/tasks/`.
- Cross-host liveness detection on shared mounts - undecidable from a pid; the threshold is the only honest signal there.
- Making the vendor step itself atomic (the copy-then-swap alternative above).

## Dependencies

None upstream. Blocks TASK-IMP-104 (its version check must precede this lock in `install.sh`) and TASK-IMP-106 (its summary reports this task's lock-removal branch). Ship 103 first; the three are parent-serialised per §11a and MUST NOT run as concurrent swarm members.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md IMP-10 plus a verification of the window on merged main; implementation under ship-tasks supervision.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `install.sh` MUST acquire `.cyberos/.install.lock` via `mkdir` before the first destructive vendor operation, and MUST write `pid=<pid>` and `started_at=<iso8601>` inside it.
- 1.2 On acquire failure where the recorded pid is alive on this host, install MUST refuse with a non-zero exit, naming the holding pid, this pid, and the lock's age. It MUST NOT proceed to the vendor step.
- 1.3 On acquire failure where the lock's age exceeds `CYBEROS_LOCK_STALE_SECS` (default 900) AND its pid is not alive on this host, install MUST break the lock, emit a warning naming the broken lock's pid and age, and continue.
- 1.4 A lock whose pid is dead but whose age is under the threshold MUST NOT be broken - it MUST refuse per 1.2, because a just-started install has not yet had time to look alive.
- 1.5 The lock MUST be released on EXIT, INT, and TERM via `trap`, including the refusal path (which MUST NOT release a lock it does not own).
- 1.6 `uninstall.sh` MUST remove `.cyberos/.install.lock` only when removing the machine, and MUST NOT treat a foreign lock as its own.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2, #1.5) - a second install against a held lock exits non-zero naming both pids; the holder's tree is unmodified - test: `tools/install/tests/test_install_lock.sh::t01_concurrent_refuses`
- [ ] AC 2 (traces_to: #1.3) - a lock older than the threshold with a dead pid is broken with a warning naming pid and age; install completes - test: `tools/install/tests/test_install_lock.sh::t02_stale_broken_with_warning`
- [ ] AC 3 (traces_to: #1.4) - a fresh lock with a dead pid refuses rather than breaking - test: `tools/install/tests/test_install_lock.sh::t03_fresh_dead_pid_refuses`
- [ ] AC 4 (traces_to: #1.5) - a SIGTERM'd install leaves no lock behind; the next install acquires cleanly - test: `tools/install/tests/test_install_lock.sh::t04_trap_releases_on_signal`
- [ ] AC 5 (traces_to: #1.6) - uninstall removes a lock it owns and keeps/names a foreign one - test: `tools/install/tests/test_install_lock.sh::t05_uninstall_lock_ownership`

## 3. Edge cases

- Lock directory exists but is empty (killed between `mkdir` and the write): no pid to probe, so the threshold alone decides - under it, refuse; over it, break. Covered by t02/t03's shape.
- `.cyberos/` on a shared network mount where the pid belongs to another host: `kill -0` is meaningless. Treat an unreadable/foreign-host lock as alive until the threshold expires - the same conservative default TASK-IMP-093's lease uses.
- pid reuse: a dead installer's pid recycled by an unrelated process reads as alive, so the lock survives to its threshold and is then broken. Conservative in the safe direction.
- Read-only `.cyberos/` (permissions damage): `mkdir` fails for a reason that is not contention. The refusal message MUST distinguish "held by pid N" from "cannot create lock" or it sends the operator hunting a process that does not exist.
- Non-git or first-ever install where `.cyberos/` does not exist yet: create the directory before acquiring - the lock cannot guard a path that is absent.
- Security-class: the lock is inside the machine's own directory, contains a pid and a timestamp, and is never read as executable input. No untrusted content, no execution surface.
